use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use serde_json::Value;
use sqlx::PgPool;
use tower::ServiceExt;
use uuid::Uuid;

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

fn create_jwt(jwt_secret: &str, tenant_id: Uuid) -> String {
    use jsonwebtoken::{encode, EncodingKey, Header};
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize)]
    struct Claims {
        sub: String,
        tenant_id: Uuid,
        role: Option<String>,
        is_admin: Option<bool>,
        scope: Option<String>,
        exp: usize,
    }

    let claims = Claims {
        sub: "test_user".to_string(),
        tenant_id,
        role: Some("admin".to_string()),
        is_admin: Some(true),
        scope: Some("full".to_string()),
        exp: chrono::Utc::now().timestamp() as usize + 3600,
    };

    encode(&Header::default(), &claims, &EncodingKey::from_secret(jwt_secret.as_bytes())).unwrap()
}

async fn create_test_tenant(pool: &PgPool) -> Uuid {
    let tenant_id = Uuid::new_v4();
    let slug = format!("tenant-{}", Uuid::new_v4().simple());
    sqlx::query(
        "INSERT INTO tenants (id, name, slug, status, plan) VALUES ($1, $2, $3, 'active', 'free') ON CONFLICT DO NOTHING",
    )
    .bind(tenant_id)
    .bind("DLQ Test Tenant")
    .bind(slug)
    .execute(pool)
    .await
    .expect("Failed to create tenant");

    tenant_id
}

#[tokio::test]
async fn test_dlq_retry_all_and_list() {
    let (app, state, pool) = setup_test_app().await;
    let tenant_id = create_test_tenant(&pool).await;
    let token = create_jwt(&state.config.jwt_secret, tenant_id);

    // List DLQ (initially empty)
    let list_req = Request::builder()
        .uri("/api/v1/dlq")
        .method("GET")
        .header("Authorization", format!("Bearer {token}"))
        .body(Body::empty())
        .unwrap();

    let list_res = app.clone().oneshot(list_req).await.unwrap();
    assert_eq!(list_res.status(), StatusCode::OK);

    // Retry all DLQ
    let retry_req = Request::builder()
        .uri("/api/v1/dlq/retry-all")
        .method("POST")
        .header("Authorization", format!("Bearer {token}"))
        .body(Body::empty())
        .unwrap();

    let retry_res = app.clone().oneshot(retry_req).await.unwrap();
    assert_eq!(retry_res.status(), StatusCode::OK);

    let body_bytes = axum::body::to_bytes(retry_res.into_body(), usize::MAX).await.unwrap();
    let body_json: Value = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(body_json["success"], true);
}
