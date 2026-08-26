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

fn create_jwt(jwt_secret: &str, tenant_id: Uuid, is_admin: bool) -> String {
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
        role: if is_admin { Some("admin".to_string()) } else { Some("member".to_string()) },
        is_admin: Some(is_admin),
        scope: Some("full".to_string()),
        exp: chrono::Utc::now().timestamp() as usize + 3600,
    };

    encode(&Header::default(), &claims, &EncodingKey::from_secret(jwt_secret.as_bytes())).unwrap()
}

#[tokio::test]
async fn test_tenant_crud_lifecycle() {
    let (app, state, _pool) = setup_test_app().await;

    let slug = format!("org-{}", Uuid::new_v4().simple());
    let create_payload = json!({
        "name": "New Organization Corp",
        "slug": slug
    });

    // 1. Create tenant
    let req = Request::builder()
        .uri("/api/v1/tenants")
        .method("POST")
        .header("Content-Type", "application/json")
        .body(Body::from(serde_json::to_vec(&create_payload).unwrap()))
        .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);

    let body_bytes = axum::body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let body_json: Value = serde_json::from_slice(&body_bytes).unwrap();
    let tenant_id = Uuid::parse_str(body_json["id"].as_str().unwrap()).unwrap();

    let admin_token = create_jwt(&state.config.jwt_secret, tenant_id, true);
    let member_token = create_jwt(&state.config.jwt_secret, tenant_id, false);

    // 2. Get tenant (as member)
    let get_req = Request::builder()
        .uri(format!("/api/v1/tenants/{tenant_id}"))
        .method("GET")
        .header("Authorization", format!("Bearer {member_token}"))
        .body(Body::empty())
        .unwrap();

    let get_res = app.clone().oneshot(get_req).await.unwrap();
    assert_eq!(get_res.status(), StatusCode::OK);

    // 3. Update tenant
    let update_req = Request::builder()
        .uri(format!("/api/v1/tenants/{tenant_id}"))
        .method("PUT")
        .header("Authorization", format!("Bearer {member_token}"))
        .header("Content-Type", "application/json")
        .body(Body::from(serde_json::to_vec(&json!({ "name": "Updated Org Name" })).unwrap()))
        .unwrap();

    let update_res = app.clone().oneshot(update_req).await.unwrap();
    assert_eq!(update_res.status(), StatusCode::OK);

    // 4. Cross-tenant access rejected
    let other_tenant_id = Uuid::new_v4();
    let cross_token = create_jwt(&state.config.jwt_secret, other_tenant_id, false);
    let cross_req = Request::builder()
        .uri(format!("/api/v1/tenants/{tenant_id}"))
        .method("GET")
        .header("Authorization", format!("Bearer {cross_token}"))
        .body(Body::empty())
        .unwrap();

    let cross_res = app.clone().oneshot(cross_req).await.unwrap();
    assert_eq!(cross_res.status(), StatusCode::FORBIDDEN);

    // 5. Delete tenant (as admin)
    let delete_req = Request::builder()
        .uri(format!("/api/v1/tenants/{tenant_id}"))
        .method("DELETE")
        .header("Authorization", format!("Bearer {admin_token}"))
        .body(Body::empty())
        .unwrap();

    let delete_res = app.clone().oneshot(delete_req).await.unwrap();
    assert_eq!(delete_res.status(), StatusCode::NO_CONTENT);
}
