use axum::{
    body::Body,
    http::{header, Request, StatusCode},
};
use chrono::{Duration, Utc};
use serde_json::{json, Value};
use sqlx::{PgPool, Row};
use tower::ServiceExt;
use uuid::Uuid;

use api::{create_router, AppState};
use relay_core::config::Config;
use data::pool::{create_pg_pool, run_migrations};
use data::queue::RedisQueue;
use domain::dto::{CreateApiKeyInput, CreateTenantInput};

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
                name: "Bootstrap Key".to_string(),
                expires_at: None,
            },
        )
        .await
        .expect("Failed to create test API key");

    (tenant.id, api_key.raw_key)
}

// 1. Successful key creation
#[tokio::test]
async fn test_successful_api_key_creation() {
    let (app, state, pool) = setup_test_app().await;
    let (_tenant_id, raw_key) = create_test_tenant_and_key(&state, "ak-create").await;

    let payload = json!({
        "name": "Production Service Key",
        "expires_at": null
    });

    let req = Request::builder()
        .uri("/v1/api-keys")
        .method("POST")
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::AUTHORIZATION, format!("Bearer {raw_key}"))
        .body(Body::from(payload.to_string()))
        .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);

    let body_bytes = axum::body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let json_res: Value = serde_json::from_slice(&body_bytes).unwrap();

    assert_eq!(json_res["name"], "Production Service Key");
    assert!(json_res["id"].is_string());
    assert!(json_res["raw_key"].is_string());
    assert!(json_res["key_prefix"].is_string());

    let returned_raw = json_res["raw_key"].as_str().unwrap();
    let returned_prefix = json_res["key_prefix"].as_str().unwrap();
    assert!(returned_raw.starts_with("rc_live_"));
    assert_eq!(&returned_raw[..12], returned_prefix);

    // Verify key_hash stored in DB is SHA-256 and NOT the raw key
    let key_id = Uuid::parse_str(json_res["id"].as_str().unwrap()).unwrap();
    let row = sqlx::query("SELECT key_hash FROM api_keys WHERE id = $1")
        .bind(key_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    let db_hash: String = row.get("key_hash");
    assert_ne!(db_hash, returned_raw);
    assert_eq!(db_hash.len(), 64); // 256-bit SHA-256 hex string

    // Also verify /api/v1/api-keys alias
    let req_api = Request::builder()
        .uri("/api/v1/api-keys")
        .method("POST")
        .header(header::CONTENT_TYPE, "application/json")
        .header("X-Api-Key", &raw_key)
        .body(Body::from(json!({"name": "Secondary Key"}).to_string()))
        .unwrap();

    let res_api = app.oneshot(req_api).await.unwrap();
    assert_eq!(res_api.status(), StatusCode::CREATED);
}

// 2. Authentication failure on create
#[tokio::test]
async fn test_create_api_key_unauthenticated() {
    let (app, _state, _pool) = setup_test_app().await;

    // Missing auth
    let req = Request::builder()
        .uri("/v1/api-keys")
        .method("POST")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(json!({"name": "Key"}).to_string()))
        .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);

    // Invalid token
    let req_invalid = Request::builder()
        .uri("/v1/api-keys")
        .method("POST")
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::AUTHORIZATION, "Bearer invalid_nonexistent_token")
        .body(Body::from(json!({"name": "Key"}).to_string()))
        .unwrap();

    let res_invalid = app.oneshot(req_invalid).await.unwrap();
    assert_eq!(res_invalid.status(), StatusCode::UNAUTHORIZED);
}

// 3. Validation failure (empty name)
#[tokio::test]
async fn test_create_api_key_empty_name() {
    let (app, state, _pool) = setup_test_app().await;
    let (_tenant_id, raw_key) = create_test_tenant_and_key(&state, "ak-val").await;

    let req = Request::builder()
        .uri("/v1/api-keys")
        .method("POST")
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::AUTHORIZATION, format!("Bearer {raw_key}"))
        .body(Body::from(json!({"name": "   "}).to_string()))
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

// 4. Expiration works
#[tokio::test]
async fn test_api_key_expiration() {
    let (app, state, _pool) = setup_test_app().await;
    let (tenant_id, _raw_key) = create_test_tenant_and_key(&state, "ak-exp").await;

    // Create an expired key (expired 1 hour ago)
    let expired_key_resp = state
        .auth_service
        .create_api_key(
            tenant_id,
            CreateApiKeyInput {
                name: "Expired Key".to_string(),
                expires_at: Some(Utc::now() - Duration::hours(1)),
            },
        )
        .await
        .unwrap();

    // Authenticating with expired key must fail
    let req = Request::builder()
        .uri("/v1/api-keys")
        .method("GET")
        .header(header::AUTHORIZATION, format!("Bearer {}", expired_key_resp.raw_key))
        .body(Body::empty())
        .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);

    // Create a valid future key (expires in 10 hours)
    let valid_key_resp = state
        .auth_service
        .create_api_key(
            tenant_id,
            CreateApiKeyInput {
                name: "Future Key".to_string(),
                expires_at: Some(Utc::now() + Duration::hours(10)),
            },
        )
        .await
        .unwrap();

    // Authenticating with valid future key must succeed
    let req_valid = Request::builder()
        .uri("/v1/api-keys")
        .method("GET")
        .header(header::AUTHORIZATION, format!("Bearer {}", valid_key_resp.raw_key))
        .body(Body::empty())
        .unwrap();

    let res_valid = app.oneshot(req_valid).await.unwrap();
    assert_eq!(res_valid.status(), StatusCode::OK);
}

// 5. List API keys & verify no sensitive hash/raw secret exposed
#[tokio::test]
async fn test_list_api_keys_and_no_secrets_exposed() {
    let (app, state, _pool) = setup_test_app().await;
    let (tenant_id, raw_key) = create_test_tenant_and_key(&state, "ak-list").await;

    // Create 2 additional keys
    for i in 1..=2 {
        let _ = state
            .auth_service
            .create_api_key(
                tenant_id,
                CreateApiKeyInput {
                    name: format!("Key {i}"),
                    expires_at: None,
                },
            )
            .await
            .unwrap();
    }

    let req = Request::builder()
        .uri("/v1/api-keys?limit=10&offset=0")
        .method("GET")
        .header(header::AUTHORIZATION, format!("Bearer {raw_key}"))
        .body(Body::empty())
        .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let body_bytes = axum::body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let json_res: Value = serde_json::from_slice(&body_bytes).unwrap();
    let list = json_res.as_array().expect("Expected array");

    assert_eq!(list.len(), 3); // bootstrap key + 2 additional
    for item in list {
        assert_eq!(item["tenant_id"], tenant_id.to_string());
        assert!(item["key_prefix"].is_string());
        assert!(item["name"].is_string());
        // Verify key_hash and raw_key are NEVER returned in list responses
        assert!(item.get("key_hash").is_none());
        assert!(item.get("raw_key").is_none());
    }

    // Verify raw body doesn't contain raw secret prefixes
    let body_str = String::from_utf8(body_bytes.to_vec()).unwrap();
    assert!(!body_str.contains("key_hash"));
}

// 6. Cross-tenant isolation on list
#[tokio::test]
async fn test_api_keys_cross_tenant_isolation() {
    let (app, state, _pool) = setup_test_app().await;
    let (tenant_a, key_a) = create_test_tenant_and_key(&state, "ak-iso-a").await;
    let (tenant_b, key_b) = create_test_tenant_and_key(&state, "ak-iso-b").await;

    // Tenant A queries keys
    let req_a = Request::builder()
        .uri("/v1/api-keys")
        .method("GET")
        .header(header::AUTHORIZATION, format!("Bearer {key_a}"))
        .body(Body::empty())
        .unwrap();

    let res_a = app.clone().oneshot(req_a).await.unwrap();
    assert_eq!(res_a.status(), StatusCode::OK);
    let list_a: Value = serde_json::from_slice(&axum::body::to_bytes(res_a.into_body(), usize::MAX).await.unwrap()).unwrap();
    for item in list_a.as_array().unwrap() {
        assert_eq!(item["tenant_id"], tenant_a.to_string());
    }

    // Tenant B queries keys
    let req_b = Request::builder()
        .uri("/v1/api-keys")
        .method("GET")
        .header(header::AUTHORIZATION, format!("Bearer {key_b}"))
        .body(Body::empty())
        .unwrap();

    let res_b = app.oneshot(req_b).await.unwrap();
    assert_eq!(res_b.status(), StatusCode::OK);
    let list_b: Value = serde_json::from_slice(&axum::body::to_bytes(res_b.into_body(), usize::MAX).await.unwrap()).unwrap();
    for item in list_b.as_array().unwrap() {
        assert_eq!(item["tenant_id"], tenant_b.to_string());
    }
}

// 7. Successful revocation and audit log creation
#[tokio::test]
async fn test_successful_api_key_revocation_and_audit() {
    let (app, state, _pool) = setup_test_app().await;
    let (tenant_id, master_key) = create_test_tenant_and_key(&state, "ak-rev").await;

    let target_key = state
        .auth_service
        .create_api_key(
            tenant_id,
            CreateApiKeyInput {
                name: "Revocable Key".to_string(),
                expires_at: None,
            },
        )
        .await
        .unwrap();

    // Revoke target key
    let req_del = Request::builder()
        .uri(format!("/v1/api-keys/{}", target_key.id))
        .method("DELETE")
        .header(header::AUTHORIZATION, format!("Bearer {master_key}"))
        .body(Body::empty())
        .unwrap();

    let res_del = app.clone().oneshot(req_del).await.unwrap();
    assert_eq!(res_del.status(), StatusCode::NO_CONTENT);

    // Using target key now fails with 401 Unauthorized
    let req_use = Request::builder()
        .uri("/v1/api-keys")
        .method("GET")
        .header(header::AUTHORIZATION, format!("Bearer {}", target_key.raw_key))
        .body(Body::empty())
        .unwrap();

    let res_use = app.clone().oneshot(req_use).await.unwrap();
    assert_eq!(res_use.status(), StatusCode::UNAUTHORIZED);

    // Revoking again returns 409 Conflict (already revoked)
    let req_del_again = Request::builder()
        .uri(format!("/v1/api-keys/{}", target_key.id))
        .method("DELETE")
        .header(header::AUTHORIZATION, format!("Bearer {master_key}"))
        .body(Body::empty())
        .unwrap();

    let res_del_again = app.clone().oneshot(req_del_again).await.unwrap();
    assert_eq!(res_del_again.status(), StatusCode::CONFLICT);

    // Verify audit log entry was created
    let req_audit = Request::builder()
        .uri("/v1/audit-logs?action=api_key.revoked")
        .method("GET")
        .header(header::AUTHORIZATION, format!("Bearer {master_key}"))
        .body(Body::empty())
        .unwrap();

    let res_audit = app.oneshot(req_audit).await.unwrap();
    assert_eq!(res_audit.status(), StatusCode::OK);
    let audit_list: Value = serde_json::from_slice(&axum::body::to_bytes(res_audit.into_body(), usize::MAX).await.unwrap()).unwrap();
    let arr = audit_list.as_array().unwrap();
    assert!(arr.iter().any(|item| item["resource_id"] == target_key.id.to_string()));
}

// 8. Cross-tenant key revocation rejected
#[tokio::test]
async fn test_cross_tenant_api_key_revocation_rejected() {
    let (app, state, _pool) = setup_test_app().await;
    let (tenant_a, _key_a) = create_test_tenant_and_key(&state, "ak-cross-a").await;
    let (_tenant_b, key_b) = create_test_tenant_and_key(&state, "ak-cross-b").await;

    let key_a_item = state
        .auth_service
        .create_api_key(
            tenant_a,
            CreateApiKeyInput {
                name: "Tenant A Secret Key".to_string(),
                expires_at: None,
            },
        )
        .await
        .unwrap();

    // Tenant B attempts to revoke Tenant A's key -> 404 Not Found
    let req = Request::builder()
        .uri(format!("/v1/api-keys/{}", key_a_item.id))
        .method("DELETE")
        .header(header::AUTHORIZATION, format!("Bearer {key_b}"))
        .body(Body::empty())
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}

// 9. Nonexistent key revocation
#[tokio::test]
async fn test_nonexistent_api_key_revocation() {
    let (app, state, _pool) = setup_test_app().await;
    let (_tenant_id, raw_key) = create_test_tenant_and_key(&state, "ak-none").await;

    let req = Request::builder()
        .uri(format!("/v1/api-keys/{}", Uuid::new_v4()))
        .method("DELETE")
        .header(header::AUTHORIZATION, format!("Bearer {raw_key}"))
        .body(Body::empty())
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}
