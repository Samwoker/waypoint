use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use serde_json::Value;
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

// 1. Healthy service
#[tokio::test]
async fn test_health_check_healthy() {
    let (app, _state, _pool) = setup_test_app().await;

    // Test GET /v1/health
    let req = Request::builder()
        .uri("/v1/health")
        .method("GET")
        .body(Body::empty())
        .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let body_bytes = axum::body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let json_res: Value = serde_json::from_slice(&body_bytes).unwrap();

    assert_eq!(json_res["status"], "ok");
    assert_eq!(json_res["database"], "ok");
    assert_eq!(json_res["queue"], "ok");

    // Test /api/v1/health alias
    let req_api = Request::builder()
        .uri("/api/v1/health")
        .method("GET")
        .body(Body::empty())
        .unwrap();

    let res_api = app.clone().oneshot(req_api).await.unwrap();
    assert_eq!(res_api.status(), StatusCode::OK);

    // Test /health/liveness
    let req_live = Request::builder()
        .uri("/health/liveness")
        .method("GET")
        .body(Body::empty())
        .unwrap();

    let res_live = app.clone().oneshot(req_live).await.unwrap();
    assert_eq!(res_live.status(), StatusCode::OK);

    // Test /health/readiness
    let req_ready = Request::builder()
        .uri("/health/readiness")
        .method("GET")
        .body(Body::empty())
        .unwrap();

    let res_ready = app.oneshot(req_ready).await.unwrap();
    assert_eq!(res_ready.status(), StatusCode::OK);
}

// 2. Dependency failure
#[tokio::test]
async fn test_health_check_database_failure() {
    let redis_url = std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string());
    let queue = RedisQueue::new(&redis_url).await.unwrap();

    let broken_pool = sqlx::postgres::PgPoolOptions::new()
        .acquire_timeout(std::time::Duration::from_millis(10))
        .connect_lazy("postgres://invalid:invalid@localhost:9999/nonexistent")
        .unwrap();

    let config = Config {
        database_url: "postgres://invalid:invalid@localhost:9999/nonexistent".to_string(),
        redis_url,
        data_encryption_key: "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f".to_string(),
        jwt_secret: "super-secret-jwt-key-with-at-least-32-chars-length".to_string(),
        api_port: 3000,
        environment: "test".to_string(),
    };

    let state = AppState::new(config, broken_pool, queue).expect("Failed to create AppState");
    let router = create_router(state);

    let req = Request::builder()
        .uri("/v1/health")
        .method("GET")
        .body(Body::empty())
        .unwrap();

    let res = router.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::SERVICE_UNAVAILABLE);

    let body_bytes = axum::body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let json_res: Value = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(json_res["status"], "degraded");
    assert_eq!(json_res["database"], "unhealthy");
}

// 3. No secrets exposed
#[tokio::test]
async fn test_health_check_no_secrets_exposed() {
    let (app, _state, _pool) = setup_test_app().await;

    let req = Request::builder()
        .uri("/v1/health")
        .method("GET")
        .body(Body::empty())
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    let body_str = String::from_utf8(axum::body::to_bytes(res.into_body(), usize::MAX).await.unwrap().to_vec()).unwrap();

    assert!(!body_str.contains("postgres://"));
    assert!(!body_str.contains("redis://"));
    assert!(!body_str.contains("super-secret"));
    assert!(!body_str.contains("000102030405"));
}
