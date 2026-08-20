use axum::{
    body::Body,
    http::{header, Request, StatusCode},
};
use serde_json::Value;
use sqlx::PgPool;
use std::sync::Arc;
use tower::ServiceExt;
use uuid::Uuid;

use api::{create_router, AppState};
use relay_core::config::Config;
use data::pool::{create_pg_pool, run_migrations};
use data::queue::RedisQueue;
use data::repositories::DeliveryRepository;
use domain::dto::{
    CreateApiKeyInput, CreateDestinationInput, CreateEventInput, CreateSourceInput,
    CreateSubscriptionInput, CreateTenantInput,
};

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

async fn create_test_tenant_and_key(state: &AppState, slug_prefix: &str) -> (Uuid, String) {
    let random_suffix = uuid::Uuid::new_v4().simple().to_string();
    let tenant_slug = format!("{slug_prefix}-{random_suffix}");
    let tenant = state
        .tenant_service
        .create_tenant(CreateTenantInput {
            name: format!("Test Tenant {tenant_slug}"),
            slug: tenant_slug,
        })
        .await
        .expect("Failed to create test tenant");

    let api_key = state
        .auth_service
        .create_api_key(
            tenant.id,
            CreateApiKeyInput {
                name: "Test Key".to_string(),
                expires_at: None,
            },
        )
        .await
        .expect("Failed to create test API key");

    (tenant.id, api_key.raw_key)
}

// 1. Successful lookup
#[tokio::test]
async fn test_successful_delivery_lookup() {
    let (app, state, pool) = setup_test_app().await;
    let (tenant_id, raw_key) = create_test_tenant_and_key(&state, "succ-del").await;

    // Create source
    let source = state
        .source_service
        .create_source(
            tenant_id,
            CreateSourceInput {
                name: "Delivery Source".to_string(),
                slug: format!("src-{}", Uuid::new_v4().simple()),
                description: None,
                provider: "generic".to_string(),
                verification_type: "none".to_string(),
                secret: None,
            },
        )
        .await
        .unwrap();

    // Create destination
    let dest = state
        .destination_service
        .create_destination(
            tenant_id,
            CreateDestinationInput {
                name: "Delivery Dest".to_string(),
                url: "https://example.com/webhooks".to_string(),
                description: None,
                rate_limit_rps: None,
                timeout_ms: None,
                secret: None,
                max_retries: Some(5),
                headers: None,
            },
        )
        .await
        .unwrap();

    // Create subscription
    let sub = state
        .subscription_service
        .create_subscription(
            tenant_id,
            CreateSubscriptionInput {
                source_id: source.id,
                destination_id: dest.id,
                event_types: vec!["payment.succeeded".to_string()],
                filter_rules: None,
                transformation_template: None,
            },
        )
        .await
        .unwrap();

    // Create event
    let event = state
        .ingestion_service
        .create_event(
            tenant_id,
            CreateEventInput {
                source_id: Some(source.id),
                event_type: "payment.succeeded".to_string(),
                payload: serde_json::json!({"amount": 100}),
                idempotency_key: None,
                headers: None,
            },
        )
        .await
        .unwrap();

    // Create delivery in DB
    let delivery_repo = DeliveryRepository::new(&pool);
    let delivery = delivery_repo
        .create(tenant_id, event.id, sub.id, dest.id, 5)
        .await
        .unwrap();

    // Query GET /v1/deliveries/{id}
    let req = Request::builder()
        .uri(format!("/v1/deliveries/{}", delivery.id))
        .method("GET")
        .header(header::AUTHORIZATION, format!("Bearer {raw_key}"))
        .body(Body::empty())
        .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let body_bytes = axum::body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let json_res: Value = serde_json::from_slice(&body_bytes).unwrap();

    assert_eq!(json_res["id"], delivery.id.to_string());
    assert_eq!(json_res["tenant_id"], tenant_id.to_string());
    assert_eq!(json_res["event_id"], event.id.to_string());
    assert_eq!(json_res["destination_id"], dest.id.to_string());
    assert_eq!(json_res["status"], "pending");
    assert_eq!(json_res["attempt_count"], 0);
    assert_eq!(json_res["max_attempts"], 5);

    // Also verify /api/v1/deliveries/{id} alias
    let req_api = Request::builder()
        .uri(format!("/api/v1/deliveries/{}", delivery.id))
        .method("GET")
        .header("X-Api-Key", &raw_key)
        .body(Body::empty())
        .unwrap();

    let res_api = app.oneshot(req_api).await.unwrap();
    assert_eq!(res_api.status(), StatusCode::OK);
}

// 2. Nonexistent delivery
#[tokio::test]
async fn test_get_delivery_nonexistent() {
    let (app, state, _pool) = setup_test_app().await;
    let (_tenant_id, raw_key) = create_test_tenant_and_key(&state, "del-none").await;

    let non_existent_id = Uuid::new_v4();
    let req = Request::builder()
        .uri(format!("/v1/deliveries/{non_existent_id}"))
        .method("GET")
        .header(header::AUTHORIZATION, format!("Bearer {raw_key}"))
        .body(Body::empty())
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}

// 3. Malformed ID
#[tokio::test]
async fn test_get_delivery_malformed_id() {
    let (app, state, _pool) = setup_test_app().await;
    let (_tenant_id, raw_key) = create_test_tenant_and_key(&state, "del-mal").await;

    let req = Request::builder()
        .uri("/v1/deliveries/not-a-valid-uuid-12345")
        .method("GET")
        .header(header::AUTHORIZATION, format!("Bearer {raw_key}"))
        .body(Body::empty())
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert!(res.status() == StatusCode::BAD_REQUEST || res.status() == StatusCode::NOT_FOUND || res.status().is_client_error());
}

// 4. Cross-tenant delivery
#[tokio::test]
async fn test_get_delivery_cross_tenant_rejected() {
    let (app, state, pool) = setup_test_app().await;
    let (tenant_a, _key_a) = create_test_tenant_and_key(&state, "del-cross-a").await;
    let (_tenant_b, key_b) = create_test_tenant_and_key(&state, "del-cross-b").await;

    // Tenant A resources
    let source_a = state
        .source_service
        .create_source(
            tenant_a,
            CreateSourceInput {
                name: "Src A".to_string(),
                slug: format!("src-a-{}", Uuid::new_v4().simple()),
                description: None,
                provider: "generic".to_string(),
                verification_type: "none".to_string(),
                secret: None,
            },
        )
        .await
        .unwrap();

    let dest_a = state
        .destination_service
        .create_destination(
            tenant_a,
            CreateDestinationInput {
                name: "Dest A".to_string(),
                url: "https://example.com/dest-a".to_string(),
                description: None,
                rate_limit_rps: None,
                timeout_ms: None,
                secret: None,
                max_retries: None,
                headers: None,
            },
        )
        .await
        .unwrap();

    let sub_a = state
        .subscription_service
        .create_subscription(
            tenant_a,
            CreateSubscriptionInput {
                source_id: source_a.id,
                destination_id: dest_a.id,
                event_types: vec!["test.event".to_string()],
                filter_rules: None,
                transformation_template: None,
            },
        )
        .await
        .unwrap();

    let event_a = state
        .ingestion_service
        .create_event(
            tenant_a,
            CreateEventInput {
                source_id: Some(source_a.id),
                event_type: "test.event".to_string(),
                payload: serde_json::json!({"secret": "data"}),
                idempotency_key: None,
                headers: None,
            },
        )
        .await
        .unwrap();

    let delivery_repo = DeliveryRepository::new(&pool);
    let delivery_a = delivery_repo
        .create(tenant_a, event_a.id, sub_a.id, dest_a.id, 3)
        .await
        .unwrap();

    // Tenant B queries Tenant A's delivery -> 404 Not Found (strict multi-tenant isolation)
    let req = Request::builder()
        .uri(format!("/v1/deliveries/{}", delivery_a.id))
        .method("GET")
        .header(header::AUTHORIZATION, format!("Bearer {key_b}"))
        .body(Body::empty())
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}

// 5. Authentication failure
#[tokio::test]
async fn test_get_delivery_unauthenticated() {
    let (app, _state, _pool) = setup_test_app().await;
    let delivery_id = Uuid::new_v4();

    // Missing auth header
    let req = Request::builder()
        .uri(format!("/v1/deliveries/{delivery_id}"))
        .method("GET")
        .body(Body::empty())
        .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);

    // Invalid key
    let req_invalid = Request::builder()
        .uri(format!("/v1/deliveries/{delivery_id}"))
        .method("GET")
        .header(header::AUTHORIZATION, "Bearer invalid_nonexistent_key_123")
        .body(Body::empty())
        .unwrap();

    let res_invalid = app.oneshot(req_invalid).await.unwrap();
    assert_eq!(res_invalid.status(), StatusCode::UNAUTHORIZED);
}

// 6. Database failure
#[tokio::test]
async fn test_get_delivery_database_failure() {
    let (_app, state, _pool) = setup_test_app().await;
    let (tenant_id, _) = create_test_tenant_and_key(&state, "get-del-db-fail").await;

    let invalid_pool = Arc::new(
        sqlx::postgres::PgPoolOptions::new()
            .acquire_timeout(std::time::Duration::from_millis(10))
            .connect_lazy("postgres://invalid:invalid@localhost:9999/nonexistent")
            .unwrap(),
    );
    let broken_delivery_service = domain::services::DeliveryService::new(invalid_pool, reqwest::Client::new());

    let res = broken_delivery_service.get_delivery(tenant_id, Uuid::new_v4()).await;
    assert!(res.is_err());
    match res.unwrap_err() {
        relay_core::error::CoreError::Internal(msg) => {
            assert!(msg.contains("Database error") || msg.contains("connection"));
        }
        err => panic!("Expected CoreError::Internal, got {:?}", err),
    }
}
