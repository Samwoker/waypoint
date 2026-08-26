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
use relay_core::crypto::hash_password;
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

async fn create_test_tenant_and_user(pool: &PgPool, email: &str, password: &str) -> (Uuid, Uuid) {
    let tenant_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    let slug = format!("tenant-{}", Uuid::new_v4().simple());

    sqlx::query(
        "INSERT INTO tenants (id, name, slug, status, plan) VALUES ($1, $2, $3, 'active', 'free')",
    )
    .bind(tenant_id)
    .bind("Auth Test Tenant")
    .bind(slug)
    .execute(pool)
    .await
    .expect("Failed to create tenant");

    let password_hash = hash_password(password).expect("Failed to hash password");

    sqlx::query(
        "INSERT INTO users (id, tenant_id, email, password_hash, role, status) VALUES ($1, $2, $3, $4, 'admin', 'active')",
    )
    .bind(user_id)
    .bind(tenant_id)
    .bind(email)
    .bind(password_hash)
    .execute(pool)
    .await
    .expect("Failed to create user");

    (tenant_id, user_id)
}

#[tokio::test]
async fn test_user_login_success() {
    let (app, _state, pool) = setup_test_app().await;
    let email = format!("user_{}@example.com", Uuid::new_v4().simple());
    let password = "SuperSecretPassword123!";

    let (_tenant_id, _user_id) = create_test_tenant_and_user(&pool, &email, password).await;

    let payload = json!({
        "email": email,
        "password": password
    });

    let req = Request::builder()
        .uri("/api/v1/auth/login")
        .method("POST")
        .header("Content-Type", "application/json")
        .body(Body::from(serde_json::to_vec(&payload).unwrap()))
        .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let body_bytes = axum::body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let body_json: Value = serde_json::from_slice(&body_bytes).unwrap();
    assert!(body_json["access_token"].is_string());
    assert!(body_json["refresh_token"].is_string());
    assert_eq!(body_json["token_type"], "Bearer");
}

#[tokio::test]
async fn test_user_login_invalid_password() {
    let (app, _state, pool) = setup_test_app().await;
    let email = format!("user_{}@example.com", Uuid::new_v4().simple());
    let password = "SuperSecretPassword123!";

    let (_tenant_id, _user_id) = create_test_tenant_and_user(&pool, &email, password).await;

    let payload = json!({
        "email": email,
        "password": "WrongPassword!"
    });

    let req = Request::builder()
        .uri("/api/v1/auth/login")
        .method("POST")
        .header("Content-Type", "application/json")
        .body(Body::from(serde_json::to_vec(&payload).unwrap()))
        .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_auth_refresh_token_and_me() {
    let (app, _state, pool) = setup_test_app().await;
    let email = format!("user_{}@example.com", Uuid::new_v4().simple());
    let password = "SuperSecretPassword123!";

    let (_tenant_id, _user_id) = create_test_tenant_and_user(&pool, &email, password).await;

    // 1. Login
    let login_req = Request::builder()
        .uri("/api/v1/auth/login")
        .method("POST")
        .header("Content-Type", "application/json")
        .body(Body::from(serde_json::to_vec(&json!({ "email": email, "password": password })).unwrap()))
        .unwrap();

    let login_res = app.clone().oneshot(login_req).await.unwrap();
    let login_bytes = axum::body::to_bytes(login_res.into_body(), usize::MAX).await.unwrap();
    let login_json: Value = serde_json::from_slice(&login_bytes).unwrap();
    let access_token = login_json["access_token"].as_str().unwrap();
    let refresh_token = login_json["refresh_token"].as_str().unwrap();

    // 2. Refresh token
    let refresh_req = Request::builder()
        .uri("/api/v1/auth/refresh")
        .method("POST")
        .header("Content-Type", "application/json")
        .body(Body::from(serde_json::to_vec(&json!({ "refresh_token": refresh_token })).unwrap()))
        .unwrap();

    let refresh_res = app.clone().oneshot(refresh_req).await.unwrap();
    assert_eq!(refresh_res.status(), StatusCode::OK);

    // 3. Get /me
    let me_req = Request::builder()
        .uri("/api/v1/auth/me")
        .method("GET")
        .header("Authorization", format!("Bearer {access_token}"))
        .body(Body::empty())
        .unwrap();

    let me_res = app.clone().oneshot(me_req).await.unwrap();
    assert_eq!(me_res.status(), StatusCode::OK);

    let me_bytes = axum::body::to_bytes(me_res.into_body(), usize::MAX).await.unwrap();
    let me_json: Value = serde_json::from_slice(&me_bytes).unwrap();
    assert_eq!(me_json["is_admin"], true);
}
