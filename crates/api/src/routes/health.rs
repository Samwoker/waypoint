use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde_json::json;
use sqlx::Row;
use std::time::Duration;

use crate::state::AppState;

/// /healthz (Endpoint #56) — DB + Redis health with 2s timeout, no auth
pub async fn healthz(State(state): State<AppState>) -> impl IntoResponse {
    let db_ok = tokio::time::timeout(
        Duration::from_secs(2),
        sqlx::query("SELECT 1").fetch_one(&*state.pool),
    )
    .await
    .map(|r| r.is_ok())
    .unwrap_or(false);

    let queue_ok = tokio::time::timeout(
        Duration::from_secs(2),
        async {
            let mut q = state.queue.lock().await;
            q.ping().await
        },
    )
    .await
    .map(|r| r.is_ok())
    .unwrap_or(false);

    let overall_ok = db_ok && queue_ok;
    let status_code = if overall_ok {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };

    (
        status_code,
        Json(json!({
            "db": if db_ok { "ok" } else { "unhealthy" },
            "queue": if queue_ok { "ok" } else { "unhealthy" },
        })),
    )
}

pub async fn health_check(State(state): State<AppState>) -> impl IntoResponse {
    let db_ok = sqlx::query("SELECT 1")
        .fetch_one(&*state.pool)
        .await
        .is_ok();

    let queue_ok = {
        let mut q = state.queue.lock().await;
        q.ping().await.is_ok()
    };

    let overall_ok = db_ok && queue_ok;
    let status_code = if overall_ok {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };

    (
        status_code,
        Json(json!({
            "status": if overall_ok { "ok" } else { "degraded" },
            "database": if db_ok { "ok" } else { "unhealthy" },
            "queue": if queue_ok { "ok" } else { "unhealthy" }
        })),
    )
}

pub async fn liveness() -> impl IntoResponse {
    (StatusCode::OK, Json(json!({"status": "ok"})))
}

pub async fn readiness(State(state): State<AppState>) -> impl IntoResponse {
    health_check(State(state)).await
}

/// GET /metrics (Endpoint #57) — Prometheus text format, no auth
pub async fn metrics(State(state): State<AppState>) -> impl IntoResponse {
    let event_count: i64 = sqlx::query("SELECT COUNT(*) FROM events")
        .fetch_one(&*state.pool)
        .await
        .and_then(|r| r.try_get(0))
        .unwrap_or(0);

    let delivery_count: i64 = sqlx::query("SELECT COUNT(*) FROM deliveries")
        .fetch_one(&*state.pool)
        .await
        .and_then(|r| r.try_get(0))
        .unwrap_or(0);

    let successful_deliveries: i64 = sqlx::query("SELECT COUNT(*) FROM deliveries WHERE status = 'delivered'")
        .fetch_one(&*state.pool)
        .await
        .and_then(|r| r.try_get(0))
        .unwrap_or(0);

    let failed_deliveries: i64 = sqlx::query("SELECT COUNT(*) FROM deliveries WHERE status IN ('failed', 'dead_letter', 'dead_lettered')")
        .fetch_one(&*state.pool)
        .await
        .and_then(|r| r.try_get(0))
        .unwrap_or(0);

    let pending_deliveries: i64 = sqlx::query("SELECT COUNT(*) FROM deliveries WHERE status = 'pending'")
        .fetch_one(&*state.pool)
        .await
        .and_then(|r| r.try_get(0))
        .unwrap_or(0);

    let attempts_count: i64 = sqlx::query("SELECT COUNT(*) FROM delivery_attempts")
        .fetch_one(&*state.pool)
        .await
        .and_then(|r| r.try_get(0))
        .unwrap_or(0);

    let p50_latency: Option<f64> = sqlx::query_scalar(
        "SELECT percentile_cont(0.5) WITHIN GROUP (ORDER BY duration_ms) FROM delivery_attempts WHERE duration_ms IS NOT NULL"
    )
    .fetch_one(&*state.pool)
    .await
    .unwrap_or(None);

    let p95_latency: Option<f64> = sqlx::query_scalar(
        "SELECT percentile_cont(0.95) WITHIN GROUP (ORDER BY duration_ms) FROM delivery_attempts WHERE duration_ms IS NOT NULL"
    )
    .fetch_one(&*state.pool)
    .await
    .unwrap_or(None);

    // Prometheus text exposition format
    let body = format!(
        r#"# HELP relaycore_events_received_total Total number of events received
# TYPE relaycore_events_received_total counter
relaycore_events_received_total {event_count}

# HELP relaycore_deliveries_total Total number of deliveries
# TYPE relaycore_deliveries_total counter
relaycore_deliveries_total {delivery_count}

# HELP relaycore_deliveries_succeeded_total Total successful deliveries
# TYPE relaycore_deliveries_succeeded_total counter
relaycore_deliveries_succeeded_total {successful_deliveries}

# HELP relaycore_deliveries_failed_total Total failed deliveries
# TYPE relaycore_deliveries_failed_total counter
relaycore_deliveries_failed_total {failed_deliveries}

# HELP relaycore_deliveries_pending_total Current pending deliveries
# TYPE relaycore_deliveries_pending_total gauge
relaycore_deliveries_pending_total {pending_deliveries}

# HELP relaycore_delivery_attempts_total Total delivery attempts made
# TYPE relaycore_delivery_attempts_total counter
relaycore_delivery_attempts_total {attempts_count}

# HELP relaycore_delivery_latency_p50_ms P50 delivery attempt latency in milliseconds
# TYPE relaycore_delivery_latency_p50_ms gauge
relaycore_delivery_latency_p50_ms {p50}

# HELP relaycore_delivery_latency_p95_ms P95 delivery attempt latency in milliseconds
# TYPE relaycore_delivery_latency_p95_ms gauge
relaycore_delivery_latency_p95_ms {p95}
"#,
        p50 = p50_latency.map(|v| v.to_string()).unwrap_or_else(|| "NaN".to_string()),
        p95 = p95_latency.map(|v| v.to_string()).unwrap_or_else(|| "NaN".to_string()),
    );

    (
        StatusCode::OK,
        [(axum::http::header::CONTENT_TYPE, "text/plain; version=0.0.4; charset=utf-8")],
        body,
    )
}
