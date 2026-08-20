use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde_json::json;
use sqlx::Row;

use crate::state::AppState;

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

    let failed_deliveries: i64 = sqlx::query("SELECT COUNT(*) FROM deliveries WHERE status = 'failed' OR status = 'dead_letter'")
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

    (
        StatusCode::OK,
        Json(json!({
            "events_received_total": event_count,
            "deliveries_total": delivery_count,
            "deliveries_succeeded_total": successful_deliveries,
            "deliveries_failed_total": failed_deliveries,
            "deliveries_pending_total": pending_deliveries,
            "delivery_attempts_total": attempts_count
        })),
    )
}
