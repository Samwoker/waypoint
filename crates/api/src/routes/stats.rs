use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

use crate::error::ApiError;
use crate::middleware::auth::AuthenticatedTenant;
use crate::state::AppState;
use data::repositories::StatsRepository;

#[derive(Debug, Deserialize)]
pub struct PeriodQuery {
    pub period: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct TimeseriesQuery {
    pub metric: Option<String>,
    pub bucket: Option<String>,
    pub period: Option<String>,
}

/// GET /stats/overview?period=24h  (Endpoint #52)
pub async fn get_stats_overview(
    tenant: AuthenticatedTenant,
    State(state): State<AppState>,
    Query(query): Query<PeriodQuery>,
) -> Result<impl IntoResponse, ApiError> {
    let period = query.period.as_deref().unwrap_or("24h");
    let interval = StatsRepository::parse_period_interval(period).map_err(ApiError)?;

    let repo = StatsRepository::new(&state.pool);
    let stats = repo
        .get_overview_stats(tenant.tenant_id, interval)
        .await
        .map_err(ApiError)?;

    Ok((StatusCode::OK, Json(json!({
        "period": period,
        "total_events": stats.total_events,
        "total_deliveries": stats.total_deliveries,
        "delivered_count": stats.delivered_count,
        "success_rate": stats.success_rate,
        "p50_latency_ms": stats.p50_latency_ms,
        "p95_latency_ms": stats.p95_latency_ms,
    }))))
}

/// GET /stats/sources/{source_id}?period=7d  (Endpoint #53)
pub async fn get_source_stats(
    tenant: AuthenticatedTenant,
    State(state): State<AppState>,
    Path(source_id): Path<Uuid>,
    Query(query): Query<PeriodQuery>,
) -> Result<impl IntoResponse, ApiError> {
    let period = query.period.as_deref().unwrap_or("7d");
    let interval = StatsRepository::parse_period_interval(period).map_err(ApiError)?;

    let repo = StatsRepository::new(&state.pool);
    let daily = repo
        .get_source_daily_stats(tenant.tenant_id, source_id, interval)
        .await
        .map_err(ApiError)?;

    let days: Vec<serde_json::Value> = daily.iter().map(|d| json!({
        "day": d.day,
        "event_count": d.event_count,
        "signature_valid_count": d.signature_valid_count,
        "signature_invalid_count": d.signature_invalid_count,
    })).collect();

    Ok((StatusCode::OK, Json(json!({
        "source_id": source_id,
        "period": period,
        "daily": days,
    }))))
}

/// GET /stats/destinations/{destination_id}?period=7d  (Endpoint #54)
pub async fn get_destination_stats(
    tenant: AuthenticatedTenant,
    State(state): State<AppState>,
    Path(destination_id): Path<Uuid>,
    Query(query): Query<PeriodQuery>,
) -> Result<impl IntoResponse, ApiError> {
    let period = query.period.as_deref().unwrap_or("7d");
    let interval = StatsRepository::parse_period_interval(period).map_err(ApiError)?;

    let repo = StatsRepository::new(&state.pool);
    let (stats, daily) = repo
        .get_destination_stats(tenant.tenant_id, destination_id, interval)
        .await
        .map_err(ApiError)?;

    let days: Vec<serde_json::Value> = daily.iter().map(|d| json!({
        "day": d.day,
        "total": d.total,
        "delivered": d.delivered,
        "failed": d.failed,
    })).collect();

    Ok((StatusCode::OK, Json(json!({
        "destination_id": stats.destination_id,
        "destination_name": stats.destination_name,
        "destination_url": stats.destination_url,
        "status": stats.status,
        "period": period,
        "total_deliveries": stats.total_deliveries,
        "delivered_count": stats.delivered_count,
        "failed_count": stats.failed_count,
        "success_rate": stats.success_rate,
        "p50_latency_ms": stats.p50_latency_ms,
        "p95_latency_ms": stats.p95_latency_ms,
        "daily": days,
    }))))
}

/// GET /stats/timeseries?metric=volume|success_rate&bucket=1h|1d&period=7d  (Endpoint #55)
pub async fn get_stats_timeseries(
    tenant: AuthenticatedTenant,
    State(state): State<AppState>,
    Query(query): Query<TimeseriesQuery>,
) -> Result<impl IntoResponse, ApiError> {
    let metric = query.metric.as_deref().unwrap_or("volume");
    let bucket = query.bucket.as_deref().unwrap_or("1d");
    let period = query.period.as_deref().unwrap_or("7d");

    // Validate metric — only allow safe known values
    let valid_metric = match metric {
        "volume" | "success_rate" => metric,
        _ => return Err(ApiError(relay_core::error::CoreError::Validation(
            format!("Invalid metric '{metric}'. Valid values: volume, success_rate")
        ))),
    };

    // Map bucket to a safe SQL date_trunc unit — never interpolate user input directly
    let bucket_sql = match bucket {
        "1h" => "hour",
        "1d" => "day",
        _ => return Err(ApiError(relay_core::error::CoreError::Validation(
            format!("Invalid bucket '{bucket}'. Valid values: 1h, 1d")
        ))),
    };

    let interval = StatsRepository::parse_period_interval(period).map_err(ApiError)?;

    let repo = StatsRepository::new(&state.pool);
    let rows = repo
        .get_timeseries(tenant.tenant_id, valid_metric, bucket_sql, interval)
        .await
        .map_err(ApiError)?;

    let series: Vec<serde_json::Value> = rows.iter().map(|r| json!({
        "bucket": r.bucket,
        "value": r.value,
    })).collect();

    Ok((StatusCode::OK, Json(json!({
        "metric": metric,
        "bucket": bucket,
        "period": period,
        "series": series,
    }))))
}

pub async fn get_tenant_stats(
    tenant: AuthenticatedTenant,
    state: State<AppState>,
    query: Query<PeriodQuery>,
) -> Result<impl IntoResponse, ApiError> {
    get_stats_overview(tenant, state, query).await
}

pub async fn get_system_stats(
    tenant: AuthenticatedTenant,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, ApiError> {
    if !tenant.is_admin {
        return Err(ApiError(relay_core::error::CoreError::Forbidden(
            "Admin privileges required to view system stats".to_string(),
        )));
    }

    let repo = StatsRepository::new(&state.pool);
    let stats = repo.get_system_stats().await.map_err(ApiError)?;

    Ok((StatusCode::OK, Json(stats)))
}
