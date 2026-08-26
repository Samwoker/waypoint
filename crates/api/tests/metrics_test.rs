use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use sqlx::PgPool;
use tower::ServiceExt;

use api::{create_router, AppState};
use relay_core::config::Config;
use data::pool::{create_pg_pool, run_migrations};
use data::queue::RedisQueue;

async fn setup_test_app() -> (axum::Router, AppState, PgPool) {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/webhook_relay".to_string());
    let redis_url = std::env::var("REDIS_URL")
        .unwrap_or_else(|_| "redis://localhost:6379".to_string());

    let pool = create_pg_pool(&database_url)
        .await
        .expect("Failed to connect to test Postgres database");

    run_migrations(&pool)
        .await
        .expect("Failed to run migrations");

    let queue = RedisQueue::new(&redis_url)
        .await
        .expect("Failed to connect to test Redis");

    let config = Config {
        database_url,
        redis_url,
        data_encryption_key: "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f".to_string(),
        jwt_secret: "super-secret-jwt-key-with-at-least-32-chars-length".to_string(),
        api_port: 3000,
        environment: "test".to_string(),
    };

    let state = AppState::new(config, pool.clone(), queue).expect("Failed to create AppState");
    let router = create_router(state.clone());

    (router, state, pool)
}

// 1. Endpoint responds
#[tokio::test]
async fn test_metrics_endpoint_responds() {
    let (app, _state, _pool) = setup_test_app().await;

    // Test GET /v1/metrics
    let req1 = Request::builder()
        .uri("/v1/metrics")
        .method("GET")
        .body(Body::empty())
        .unwrap();

    let res1 = app.clone().oneshot(req1).await.unwrap();
    assert_eq!(res1.status(), StatusCode::OK);

    // Test GET /api/v1/metrics
    let req2 = Request::builder()
        .uri("/api/v1/metrics")
        .method("GET")
        .body(Body::empty())
        .unwrap();

    let res2 = app.clone().oneshot(req2).await.unwrap();
    assert_eq!(res2.status(), StatusCode::OK);

    // Test GET /metrics
    let req3 = Request::builder()
        .uri("/metrics")
        .method("GET")
        .body(Body::empty())
        .unwrap();

    let res3 = app.oneshot(req3).await.unwrap();
    assert_eq!(res3.status(), StatusCode::OK);
}

// 2. Metrics are correctly exposed in Prometheus text format
#[tokio::test]
async fn test_metrics_correctly_exposed() {
    let (app, _state, _pool) = setup_test_app().await;

    let req = Request::builder()
        .uri("/v1/metrics")
        .method("GET")
        .body(Body::empty())
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    // Must use Prometheus text content type
    let content_type = res.headers().get("content-type").unwrap().to_str().unwrap();
    assert!(content_type.contains("text/plain"), "Expected Prometheus text/plain content type");

    let body_bytes = axum::body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let body_str = std::str::from_utf8(&body_bytes).unwrap();

    assert!(body_str.contains("relaycore_events_received_total"), "Missing events_received_total metric");
    assert!(body_str.contains("relaycore_deliveries_total"), "Missing deliveries_total metric");
    assert!(body_str.contains("relaycore_deliveries_succeeded_total"), "Missing deliveries_succeeded_total metric");
    assert!(body_str.contains("relaycore_deliveries_failed_total"), "Missing deliveries_failed_total metric");
    assert!(body_str.contains("relaycore_deliveries_pending_total"), "Missing deliveries_pending_total metric");
    assert!(body_str.contains("relaycore_delivery_attempts_total"), "Missing delivery_attempts_total metric");
    assert!(body_str.contains("relaycore_delivery_latency_p50_ms"), "Missing p50 latency metric");
    assert!(body_str.contains("relaycore_delivery_latency_p95_ms"), "Missing p95 latency metric");
}

// 3. Sensitive data is not exposed
#[tokio::test]
async fn test_metrics_no_sensitive_data_exposed() {
    let (app, _state, _pool) = setup_test_app().await;

    let req = Request::builder()
        .uri("/v1/metrics")
        .method("GET")
        .body(Body::empty())
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    let body_str = String::from_utf8(axum::body::to_bytes(res.into_body(), usize::MAX).await.unwrap().to_vec()).unwrap();

    assert!(!body_str.contains("Bearer"));
    assert!(!body_str.contains("secret"));
    assert!(!body_str.contains("password"));
    assert!(!body_str.contains("key_hash"));
}
