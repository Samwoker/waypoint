use axum::{
    body::Body,
    http::{header, Request, StatusCode},
};
use serde_json::Value;
use sqlx::PgPool;
use tower::ServiceExt;
use uuid::Uuid;
use domain::dto::{
    CreateApiKeyInput, CreateDestinationInput, CreateEventInput, CreateSourceInput,
    CreateSubscriptionInput, CreateTenantInput,
};

use api::{create_router, AppState};
use relay_core::config::Config;
use data::pool::{create_pg_pool, run_migrations};
use data::queue::RedisQueue;
use data::repositories::DeliveryRepository;

async fn setup_test_app() -> (axum::Router, AppState, PgPool) {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/webhook_relay".to_string());
    let redis_url = std::env::var("REDIS_URL")
        .unwrap_or_else(|_| "redis://localhost:6379".to_string());
    let pool = create_pg_pool(&database_url).await.expect("DB connect");
    run_migrations(&pool).await.expect("Migrations");
    let queue = RedisQueue::new(&redis_url).await.expect("Redis connect");
    let config = Config {
        database_url, redis_url,
        data_encryption_key: "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f".to_string(),
        jwt_secret: "super-secret-jwt-key-with-at-least-32-chars-length".to_string(),
        api_port: 3000,
        environment: "test".to_string(),
    };
    let state = AppState::new(config, pool.clone(), queue).expect("AppState");
    let router = create_router(state.clone());
    (router, state, pool)
}

async fn create_tenant_and_key(state: &AppState, slug_prefix: &str) -> (Uuid, String) {
    let slug = format!("{slug_prefix}-{}", Uuid::new_v4().simple());
    let tenant = state.tenant_service.create_tenant(CreateTenantInput {
        name: format!("Tenant {slug}"),
        slug,
    }).await.expect("create tenant");
    let key = state.auth_service.create_api_key(tenant.id, CreateApiKeyInput {
        name: "Test Key".to_string(),
        expires_at: None,
    }).await.expect("create api key");
    (tenant.id, key.raw_key)
}

// 1. Stats Overview 24h (Endpoint #52)
#[tokio::test]
async fn test_stats_overview_24h() {
    let (app, state, pool) = setup_test_app().await;
    let (tenant_id, raw_key) = create_tenant_and_key(&state, "stats-ov").await;

    let source = state.source_service.create_source(tenant_id, CreateSourceInput {
        name: "Stats Source".to_string(),
        slug: format!("sts-src-{}", Uuid::new_v4().simple()),
        description: None,
        provider: "generic".to_string(),
        verification_type: "none".to_string(),
        secret: None,
    }).await.unwrap();

    let dest = state.destination_service.create_destination(tenant_id, CreateDestinationInput {
        name: "Stats Dest".to_string(),
        url: "https://example.com/stats".to_string(),
        description: None,
        rate_limit_rps: None,
        timeout_ms: None,
        max_retries: None,
        headers: None,
        secret: None,
    }).await.unwrap();

    let sub = state.subscription_service.create_subscription(tenant_id, CreateSubscriptionInput {
        source_id: source.id,
        destination_id: dest.id,
        event_types: vec!["stats.event".to_string()],
        filter_rules: None,
        transformation_template: None,
    }).await.unwrap();

    let event = state.ingestion_service.create_event(tenant_id, CreateEventInput {
        source_id: Some(source.id),
        event_type: "stats.event".to_string(),
        payload: serde_json::json!({"n": 1}),
        idempotency_key: None,
        headers: None,
    }).await.unwrap();

    let deliv_repo = DeliveryRepository::new(&pool);
    let d = deliv_repo.create(tenant_id, event.id, sub.id, dest.id, 3).await.unwrap();
    deliv_repo.update_status(d.id, "delivered", 1, None).await.unwrap();

    // Test GET /stats/overview?period=24h
    let req = Request::builder()
        .uri("/stats/overview?period=24h")
        .method("GET")
        .header(header::AUTHORIZATION, format!("Bearer {raw_key}"))
        .body(Body::empty())
        .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let body: Value = serde_json::from_slice(&axum::body::to_bytes(res.into_body(), usize::MAX).await.unwrap()).unwrap();
    assert_eq!(body["period"], "24h");
    assert!(body["total_events"].as_i64().unwrap() >= 1);
    assert!(body["total_deliveries"].as_i64().unwrap() >= 1);
    assert!(body["delivered_count"].as_i64().unwrap() >= 1);
    assert!(body["success_rate"].as_f64().unwrap() > 0.0);
}

// 2. Stats Overview 7d
#[tokio::test]
async fn test_stats_overview_7d() {
    let (app, state, _pool) = setup_test_app().await;
    let (_tenant_id, raw_key) = create_tenant_and_key(&state, "stats-7d").await;

    let req = Request::builder()
        .uri("/stats/overview?period=7d")
        .method("GET")
        .header(header::AUTHORIZATION, format!("Bearer {raw_key}"))
        .body(Body::empty())
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body: Value = serde_json::from_slice(&axum::body::to_bytes(res.into_body(), usize::MAX).await.unwrap()).unwrap();
    assert_eq!(body["period"], "7d");
    assert_eq!(body["total_events"], 0);
    assert_eq!(body["total_deliveries"], 0);
    assert_eq!(body["success_rate"], 0.0);
}

// 3. Invalid period
#[tokio::test]
async fn test_stats_overview_invalid_period() {
    let (app, state, _pool) = setup_test_app().await;
    let (_tenant_id, raw_key) = create_tenant_and_key(&state, "stats-inv").await;

    let req = Request::builder()
        .uri("/stats/overview?period=99x")
        .method("GET")
        .header(header::AUTHORIZATION, format!("Bearer {raw_key}"))
        .body(Body::empty())
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

// 4. Source stats (Endpoint #53)
#[tokio::test]
async fn test_source_stats() {
    let (app, state, _pool) = setup_test_app().await;
    let (tenant_id, raw_key) = create_tenant_and_key(&state, "stats-src").await;

    let source = state.source_service.create_source(tenant_id, CreateSourceInput {
        name: "Source Stats".to_string(),
        slug: format!("src-st-{}", Uuid::new_v4().simple()),
        description: None,
        provider: "generic".to_string(),
        verification_type: "none".to_string(),
        secret: None,
    }).await.unwrap();

    for _ in 0..3 {
        state.ingestion_service.create_event(tenant_id, CreateEventInput {
            source_id: Some(source.id),
            event_type: "test.event".to_string(),
            payload: serde_json::json!({}),
            idempotency_key: None,
            headers: None,
        }).await.unwrap();
    }

    let req = Request::builder()
        .uri(format!("/stats/sources/{}?period=7d", source.id))
        .method("GET")
        .header(header::AUTHORIZATION, format!("Bearer {raw_key}"))
        .body(Body::empty())
        .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body: Value = serde_json::from_slice(&axum::body::to_bytes(res.into_body(), usize::MAX).await.unwrap()).unwrap();
    assert_eq!(body["source_id"], source.id.to_string());
    let daily = body["daily"].as_array().unwrap();
    assert!(!daily.is_empty());
    let total: i64 = daily.iter().map(|d| d["event_count"].as_i64().unwrap_or(0)).sum();
    assert_eq!(total, 3);

    // Cross-tenant source returns 404
    let (_tenant_b, key_b) = create_tenant_and_key(&state, "stats-src-b").await;
    let req_b = Request::builder()
        .uri(format!("/stats/sources/{}", source.id))
        .method("GET")
        .header(header::AUTHORIZATION, format!("Bearer {key_b}"))
        .body(Body::empty())
        .unwrap();

    let res_b = app.oneshot(req_b).await.unwrap();
    assert_eq!(res_b.status(), StatusCode::NOT_FOUND);
}

// 5. Destination stats (Endpoint #54)
#[tokio::test]
async fn test_destination_stats() {
    let (app, state, _pool) = setup_test_app().await;
    let (tenant_id, raw_key) = create_tenant_and_key(&state, "stats-dst").await;

    let dest = state.destination_service.create_destination(tenant_id, CreateDestinationInput {
        name: "Dest Stats".to_string(),
        url: "https://example.com/dst-stats".to_string(),
        description: None,
        rate_limit_rps: None,
        timeout_ms: None,
        max_retries: None,
        headers: None,
        secret: None,
    }).await.unwrap();

    let req = Request::builder()
        .uri(format!("/stats/destinations/{}?period=7d", dest.id))
        .method("GET")
        .header(header::AUTHORIZATION, format!("Bearer {raw_key}"))
        .body(Body::empty())
        .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body: Value = serde_json::from_slice(&axum::body::to_bytes(res.into_body(), usize::MAX).await.unwrap()).unwrap();
    assert_eq!(body["destination_id"], dest.id.to_string());
    assert_eq!(body["destination_name"], "Dest Stats");
    assert_eq!(body["success_rate"], 0.0);
    assert_eq!(body["total_deliveries"], 0);

    // Cross-tenant destination returns 404
    let (_tenant_b, key_b) = create_tenant_and_key(&state, "stats-dst-b").await;
    let req_b = Request::builder()
        .uri(format!("/stats/destinations/{}", dest.id))
        .method("GET")
        .header(header::AUTHORIZATION, format!("Bearer {key_b}"))
        .body(Body::empty())
        .unwrap();

    let res_b = app.oneshot(req_b).await.unwrap();
    assert_eq!(res_b.status(), StatusCode::NOT_FOUND);
}

// 6. Timeseries volume + success_rate (Endpoint #55)
#[tokio::test]
async fn test_stats_timeseries() {
    let (app, state, _pool) = setup_test_app().await;
    let (_tenant_id, raw_key) = create_tenant_and_key(&state, "stats-ts").await;

    // Volume metric
    let req_vol = Request::builder()
        .uri("/stats/timeseries?metric=volume&bucket=1d&period=7d")
        .method("GET")
        .header(header::AUTHORIZATION, format!("Bearer {raw_key}"))
        .body(Body::empty())
        .unwrap();

    let res_vol = app.clone().oneshot(req_vol).await.unwrap();
    assert_eq!(res_vol.status(), StatusCode::OK);
    let body_vol: Value = serde_json::from_slice(&axum::body::to_bytes(res_vol.into_body(), usize::MAX).await.unwrap()).unwrap();
    assert_eq!(body_vol["metric"], "volume");
    assert_eq!(body_vol["bucket"], "1d");
    assert!(body_vol["series"].is_array());

    // Success rate metric
    let req_sr = Request::builder()
        .uri("/stats/timeseries?metric=success_rate&bucket=1h&period=24h")
        .method("GET")
        .header(header::AUTHORIZATION, format!("Bearer {raw_key}"))
        .body(Body::empty())
        .unwrap();

    let res_sr = app.clone().oneshot(req_sr).await.unwrap();
    assert_eq!(res_sr.status(), StatusCode::OK);

    // Invalid metric
    let req_bad_metric = Request::builder()
        .uri("/stats/timeseries?metric=hacker_injection&bucket=1d&period=7d")
        .method("GET")
        .header(header::AUTHORIZATION, format!("Bearer {raw_key}"))
        .body(Body::empty())
        .unwrap();

    let res_bad = app.clone().oneshot(req_bad_metric).await.unwrap();
    assert_eq!(res_bad.status(), StatusCode::UNPROCESSABLE_ENTITY);

    // Invalid bucket
    let req_bad_bucket = Request::builder()
        .uri("/stats/timeseries?metric=volume&bucket=1week&period=7d")
        .method("GET")
        .header(header::AUTHORIZATION, format!("Bearer {raw_key}"))
        .body(Body::empty())
        .unwrap();

    let res_bad_b = app.clone().oneshot(req_bad_bucket).await.unwrap();
    assert_eq!(res_bad_b.status(), StatusCode::UNPROCESSABLE_ENTITY);

    // Invalid period
    let req_bad_period = Request::builder()
        .uri("/stats/timeseries?metric=volume&bucket=1d&period=99years")
        .method("GET")
        .header(header::AUTHORIZATION, format!("Bearer {raw_key}"))
        .body(Body::empty())
        .unwrap();

    let res_bad_p = app.oneshot(req_bad_period).await.unwrap();
    assert_eq!(res_bad_p.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

// 7. GET /healthz (Endpoint #56) - no auth required
#[tokio::test]
async fn test_healthz_no_auth_required() {
    let (app, _state, _pool) = setup_test_app().await;

    let req = Request::builder()
        .uri("/healthz")
        .method("GET")
        .body(Body::empty())
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body: Value = serde_json::from_slice(&axum::body::to_bytes(res.into_body(), usize::MAX).await.unwrap()).unwrap();
    assert_eq!(body["db"], "ok");
    assert_eq!(body["queue"], "ok");
}

// 8. GET /metrics Prometheus format (Endpoint #57) - no auth
#[tokio::test]
async fn test_metrics_prometheus_format() {
    let (app, _state, _pool) = setup_test_app().await;

    let req = Request::builder()
        .uri("/metrics")
        .method("GET")
        .body(Body::empty())
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let content_type = res.headers().get("content-type").unwrap().to_str().unwrap();
    assert!(content_type.contains("text/plain"), "Should be Prometheus text format");

    let body = String::from_utf8(axum::body::to_bytes(res.into_body(), usize::MAX).await.unwrap().to_vec()).unwrap();
    assert!(body.contains("relaycore_events_received_total"), "Must contain event metric");
    assert!(body.contains("relaycore_deliveries_total"), "Must contain delivery metric");
    assert!(body.contains("relaycore_delivery_latency_p50_ms"), "Must contain p50 latency");
    assert!(body.contains("relaycore_delivery_latency_p95_ms"), "Must contain p95 latency");
    assert!(!body.contains("secret"), "Must not expose secrets");
    assert!(!body.contains("postgres://"), "Must not expose DB URL");
}
