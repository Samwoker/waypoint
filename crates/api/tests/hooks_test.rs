use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use serde_json::{json, Value};
use sqlx::PgPool;
use tower::ServiceExt;
use uuid::Uuid;

use api::{create_router, AppState};
use relay_core::config::Config;
use relay_core::crypto::sign_hmac_sha256;
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

async fn create_test_tenant(pool: &PgPool) -> Uuid {
    let tenant_id = Uuid::new_v4();
    let slug = format!("tenant-{}", Uuid::new_v4().simple());
    sqlx::query(
        "INSERT INTO tenants (id, name, slug, status, plan) VALUES ($1, $2, $3, 'active', 'free') ON CONFLICT DO NOTHING",
    )
    .bind(tenant_id)
    .bind("Test Tenant")
    .bind(slug)
    .execute(pool)
    .await
    .expect("Failed to create tenant");

    tenant_id
}

#[tokio::test]
async fn test_public_webhook_ingestion_no_auth_needed() {
    let (app, state, pool) = setup_test_app().await;
    let tenant_id = create_test_tenant(&pool).await;

    let slug = format!("inbound-hook-{}", Uuid::new_v4().simple());
    state
        .source_service
        .create_source(
            tenant_id,
            domain::dto::CreateSourceInput {
                name: "Public Hook Source".to_string(),
                slug: slug.clone(),
                description: None,
                provider: "generic".to_string(),
                verification_type: "none".to_string(),
                secret: None,
            },
        )
        .await
        .expect("Failed to create source");

    let payload = json!({
        "event": "user.created",
        "data": { "id": 12345, "name": "Alice" }
    });

    let req = Request::builder()
        .uri(format!("/hooks/{slug}"))
        .method("POST")
        .header("Content-Type", "application/json")
        .body(Body::from(serde_json::to_vec(&payload).unwrap()))
        .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::ACCEPTED);

    let body_bytes = axum::body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let body_json: Value = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(body_json["status"], "received");
    assert!(body_json["id"].is_string());
}

#[tokio::test]
async fn test_public_webhook_hmac_signature_verification() {
    let (app, state, pool) = setup_test_app().await;
    let tenant_id = create_test_tenant(&pool).await;

    let slug = format!("inbound-hmac-{}", Uuid::new_v4().simple());
    let secret = "my_webhook_secret_key_123";

    state
        .source_service
        .create_source(
            tenant_id,
            domain::dto::CreateSourceInput {
                name: "HMAC Hook Source".to_string(),
                slug: slug.clone(),
                description: None,
                provider: "generic".to_string(),
                verification_type: "hmac_sha256".to_string(),
                secret: Some(secret.to_string()),
            },
        )
        .await
        .expect("Failed to create source");

    let payload = json!({
        "event": "charge.succeeded",
        "amount": 5000
    });
    let payload_bytes = serde_json::to_vec(&payload).unwrap();
    let sig = sign_hmac_sha256(secret.as_bytes(), &payload_bytes).unwrap();

    // 1. Valid signature
    let req = Request::builder()
        .uri(format!("/hooks/{slug}"))
        .method("POST")
        .header("Content-Type", "application/json")
        .header("X-Signature", sig.clone())
        .body(Body::from(payload_bytes.clone()))
        .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::ACCEPTED);

    // 2. Invalid signature
    let req_invalid = Request::builder()
        .uri(format!("/hooks/{slug}"))
        .method("POST")
        .header("Content-Type", "application/json")
        .header("X-Signature", "0000000000000000000000000000000000000000000000000000000000000000")
        .body(Body::from(payload_bytes))
        .unwrap();

    let res_invalid = app.clone().oneshot(req_invalid).await.unwrap();
    assert_eq!(res_invalid.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_public_webhook_nonexistent_slug() {
    let (app, _state, _pool) = setup_test_app().await;

    let req = Request::builder()
        .uri("/hooks/nonexistent-slug-xyz")
        .method("POST")
        .header("Content-Type", "application/json")
        .body(Body::from(r#"{"test": 1}"#))
        .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}
