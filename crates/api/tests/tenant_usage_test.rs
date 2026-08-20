use axum::{
    body::Body,
    http::{header, Request, StatusCode},
};
use jsonwebtoken::{encode, EncodingKey, Header};
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

fn create_admin_jwt(state: &AppState, tenant_id: Uuid) -> String {
    #[derive(serde::Serialize)]
    struct AdminClaims {
        sub: String,
        tenant_id: Uuid,
        role: String,
        is_admin: bool,
        exp: usize,
    }

    let claims = AdminClaims {
        sub: "admin-user".to_string(),
        tenant_id,
        role: "admin".to_string(),
        is_admin: true,
        exp: (chrono::Utc::now() + chrono::Duration::hours(2)).timestamp() as usize,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(state.config.jwt_secret.as_bytes()),
    )
    .expect("Failed to encode admin JWT")
}

// 1. Valid tenant usage lookup
#[tokio::test]
async fn test_valid_tenant_usage() {
    let (app, state, pool) = setup_test_app().await;
    let (tenant_id, raw_key) = create_test_tenant_and_key(&state, "usage-ok").await;

    // Create source
    let source = state
        .source_service
        .create_source(
            tenant_id,
            CreateSourceInput {
                name: "Usage Source".to_string(),
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
                name: "Usage Dest".to_string(),
                url: "https://example.com/dest".to_string(),
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

    // Create 2 events
    let mut last_event_id = Uuid::nil();
    for i in 1..=2 {
        let ev = state
            .ingestion_service
            .create_event(
                tenant_id,
                CreateEventInput {
                    source_id: Some(source.id),
                    event_type: "payment.succeeded".to_string(),
                    payload: serde_json::json!({"item": i}),
                    idempotency_key: None,
                    headers: None,
                },
            )
            .await
            .unwrap();
        last_event_id = ev.id;
    }

    // Create delivery & delivery attempt
    let delivery_repo = DeliveryRepository::new(&pool);
    let delivery = delivery_repo
        .create(tenant_id, last_event_id, sub.id, dest.id, 5)
        .await
        .unwrap();

    let _ = delivery_repo
        .record_attempt(
            delivery.id,
            1,
            Some(200),
            None,
            None,
            None,
            None,
            None,
            Some(45),
        )
        .await
        .unwrap();

    // Query GET /tenants/{tenant_id}/usage?period=30d
    let req = Request::builder()
        .uri(format!("/tenants/{tenant_id}/usage?period=30d"))
        .method("GET")
        .header(header::AUTHORIZATION, format!("Bearer {raw_key}"))
        .body(Body::empty())
        .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let body_bytes = axum::body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let json_res: Value = serde_json::from_slice(&body_bytes).unwrap();

    assert_eq!(json_res["tenant_id"], tenant_id.to_string());
    assert_eq!(json_res["period"], "30d");
    assert!(json_res["total_events"].as_i64().unwrap() >= 2);
    assert!(json_res["total_delivery_attempts"].as_i64().unwrap() >= 1);
    assert!(json_res["daily_events"].is_array());

    // Also test /v1/tenants/{tenant_id}/usage and /api/v1/tenants/{tenant_id}/usage
    let req_v1 = Request::builder()
        .uri(format!("/v1/tenants/{tenant_id}/usage?period=30d"))
        .method("GET")
        .header(header::AUTHORIZATION, format!("Bearer {raw_key}"))
        .body(Body::empty())
        .unwrap();

    let res_v1 = app.oneshot(req_v1).await.unwrap();
    assert_eq!(res_v1.status(), StatusCode::OK);
}

// 2. Invalid tenant (nonexistent UUID)
#[tokio::test]
async fn test_nonexistent_tenant_usage() {
    let (app, state, _pool) = setup_test_app().await;
    let (tenant_id, _raw_key) = create_test_tenant_and_key(&state, "usage-none").await;

    let admin_jwt = create_admin_jwt(&state, tenant_id);
    let nonexistent_id = Uuid::new_v4();

    let req = Request::builder()
        .uri(format!("/tenants/{nonexistent_id}/usage"))
        .method("GET")
        .header(header::AUTHORIZATION, format!("Bearer {admin_jwt}"))
        .body(Body::empty())
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}

// 3. Cross-tenant access rejected for normal tenant users
#[tokio::test]
async fn test_cross_tenant_usage_rejected() {
    let (app, state, _pool) = setup_test_app().await;
    let (tenant_a, _key_a) = create_test_tenant_and_key(&state, "usage-iso-a").await;
    let (_tenant_b, key_b) = create_test_tenant_and_key(&state, "usage-iso-b").await;

    // Tenant B attempts to access Tenant A's usage
    let req = Request::builder()
        .uri(format!("/tenants/{tenant_a}/usage"))
        .method("GET")
        .header(header::AUTHORIZATION, format!("Bearer {key_b}"))
        .body(Body::empty())
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::FORBIDDEN);
}

// 4. Platform admin access succeeds for any tenant
#[tokio::test]
async fn test_platform_admin_can_access_other_tenant_usage() {
    let (app, state, _pool) = setup_test_app().await;
    let (tenant_a, _key_a) = create_test_tenant_and_key(&state, "usage-adm-a").await;
    let (tenant_admin, _) = create_test_tenant_and_key(&state, "usage-adm-b").await;

    let admin_jwt = create_admin_jwt(&state, tenant_admin);

    // Admin queries Tenant A's usage
    let req = Request::builder()
        .uri(format!("/tenants/{tenant_a}/usage?period=7d"))
        .method("GET")
        .header(header::AUTHORIZATION, format!("Bearer {admin_jwt}"))
        .body(Body::empty())
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let body_bytes = axum::body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let json_res: Value = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(json_res["tenant_id"], tenant_a.to_string());
    assert_eq!(json_res["period"], "7d");
}

// 5. Period filtering
#[tokio::test]
async fn test_tenant_usage_period_filtering() {
    let (app, state, _pool) = setup_test_app().await;
    let (tenant_id, raw_key) = create_test_tenant_and_key(&state, "usage-periods").await;

    for period in &["24h", "7d", "30d", "90d"] {
        let req = Request::builder()
            .uri(format!("/tenants/{tenant_id}/usage?period={period}"))
            .method("GET")
            .header(header::AUTHORIZATION, format!("Bearer {raw_key}"))
            .body(Body::empty())
            .unwrap();

        let res = app.clone().oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);

        let body_bytes = axum::body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
        let json_res: Value = serde_json::from_slice(&body_bytes).unwrap();
        assert_eq!(json_res["period"], *period);
    }
}

// 6. Empty period defaults to 30d
#[tokio::test]
async fn test_empty_period_defaults_to_30d() {
    let (app, state, _pool) = setup_test_app().await;
    let (tenant_id, raw_key) = create_test_tenant_and_key(&state, "usage-empty-p").await;

    let req = Request::builder()
        .uri(format!("/tenants/{tenant_id}/usage"))
        .method("GET")
        .header(header::AUTHORIZATION, format!("Bearer {raw_key}"))
        .body(Body::empty())
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let body_bytes = axum::body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let json_res: Value = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(json_res["period"], "30d");
}

// 7. Database failure handling
#[tokio::test]
async fn test_tenant_usage_database_failure() {
    let (_app, state, _pool) = setup_test_app().await;
    let (tenant_id, _) = create_test_tenant_and_key(&state, "usage-db-fail").await;

    let invalid_pool = Arc::new(
        sqlx::postgres::PgPoolOptions::new()
            .acquire_timeout(std::time::Duration::from_millis(10))
            .connect_lazy("postgres://invalid:invalid@localhost:9999/nonexistent")
            .unwrap(),
    );
    let broken_tenant_service = domain::services::TenantService::new(invalid_pool);

    let res = broken_tenant_service
        .get_tenant_usage(tenant_id, false, tenant_id, Some("30d"))
        .await;

    assert!(res.is_err());
    match res.unwrap_err() {
        relay_core::error::CoreError::Internal(msg) => {
            assert!(msg.contains("Database error") || msg.contains("connection"));
        }
        err => panic!("Expected CoreError::Internal, got {:?}", err),
    }
}

// 8. Authentication failure
#[tokio::test]
async fn test_tenant_usage_unauthenticated() {
    let (app, _state, _pool) = setup_test_app().await;
    let tenant_id = Uuid::new_v4();

    // Missing auth header
    let req = Request::builder()
        .uri(format!("/tenants/{tenant_id}/usage"))
        .method("GET")
        .body(Body::empty())
        .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);

    // Invalid key
    let req_invalid = Request::builder()
        .uri(format!("/tenants/{tenant_id}/usage"))
        .method("GET")
        .header(header::AUTHORIZATION, "Bearer invalid_nonexistent_token_999")
        .body(Body::empty())
        .unwrap();

    let res_invalid = app.oneshot(req_invalid).await.unwrap();
    assert_eq!(res_invalid.status(), StatusCode::UNAUTHORIZED);
}
