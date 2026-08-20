use axum::{
    body::Body,
    http::{header, Request, StatusCode},
};
use serde_json::Value;
use sqlx::PgPool;
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
                name: "Test Key".to_string(),
                expires_at: None,
            },
        )
        .await
        .expect("Failed to create test API key");

    (tenant.id, api_key.raw_key)
}

// 1. Authorized user gets logs
#[tokio::test]
async fn test_authorized_user_gets_audit_logs() {
    let (app, state, _pool) = setup_test_app().await;
    let (tenant_id, raw_key) = create_test_tenant_and_key(&state, "audit-get").await;

    // Create audit log entries
    for i in 1..=3 {
        let _ = state
            .audit_service
            .create_audit_log(
                tenant_id,
                None,
                &format!("action.{i}"),
                Some("destination"),
                Some(Uuid::new_v4()),
                serde_json::json!({"step": i}),
            )
            .await
            .unwrap();
    }

    // Call GET /v1/audit-logs
    let req = Request::builder()
        .uri("/v1/audit-logs")
        .method("GET")
        .header(header::AUTHORIZATION, format!("Bearer {raw_key}"))
        .body(Body::empty())
        .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let body_bytes = axum::body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let json_res: Value = serde_json::from_slice(&body_bytes).unwrap();
    let list = json_res.as_array().expect("Expected JSON array");

    assert_eq!(list.len(), 4); // 1 bootstrap key log + 3 custom logs
    for item in list {
        assert_eq!(item["tenant_id"], tenant_id.to_string());
        assert!(item["action"].is_string());
        assert!(item["created_at"].is_string());
    }

    // Also test /api/v1/audit-logs alias
    let req_api = Request::builder()
        .uri("/api/v1/audit-logs")
        .method("GET")
        .header("X-Api-Key", &raw_key)
        .body(Body::empty())
        .unwrap();

    let res_api = app.oneshot(req_api).await.unwrap();
    assert_eq!(res_api.status(), StatusCode::OK);
}

// 2. Tenant isolation
#[tokio::test]
async fn test_audit_logs_tenant_isolation() {
    let (app, state, _pool) = setup_test_app().await;
    let (tenant_a, key_a) = create_test_tenant_and_key(&state, "audit-iso-a").await;
    let (tenant_b, key_b) = create_test_tenant_and_key(&state, "audit-iso-b").await;

    // Tenant A log
    let log_a = state
        .audit_service
        .create_audit_log(
            tenant_a,
            None,
            "secret_action_a",
            Some("source"),
            None,
            serde_json::json!({"tenant": "a"}),
        )
        .await
        .unwrap();

    // Tenant B log
    let log_b = state
        .audit_service
        .create_audit_log(
            tenant_b,
            None,
            "secret_action_b",
            Some("source"),
            None,
            serde_json::json!({"tenant": "b"}),
        )
        .await
        .unwrap();

    // Query as Tenant A
    let req_a = Request::builder()
        .uri("/v1/audit-logs")
        .method("GET")
        .header(header::AUTHORIZATION, format!("Bearer {key_a}"))
        .body(Body::empty())
        .unwrap();

    let res_a = app.clone().oneshot(req_a).await.unwrap();
    assert_eq!(res_a.status(), StatusCode::OK);
    let list_a: Value = serde_json::from_slice(&axum::body::to_bytes(res_a.into_body(), usize::MAX).await.unwrap()).unwrap();
    let arr_a = list_a.as_array().unwrap();
    assert!(arr_a.iter().all(|l| l["tenant_id"] == tenant_a.to_string()));
    assert!(arr_a.iter().any(|l| l["id"] == log_a.id.to_string()));
    assert!(!arr_a.iter().any(|l| l["id"] == log_b.id.to_string()));

    // Query as Tenant B
    let req_b = Request::builder()
        .uri("/v1/audit-logs")
        .method("GET")
        .header(header::AUTHORIZATION, format!("Bearer {key_b}"))
        .body(Body::empty())
        .unwrap();

    let res_b = app.oneshot(req_b).await.unwrap();
    assert_eq!(res_b.status(), StatusCode::OK);
    let list_b: Value = serde_json::from_slice(&axum::body::to_bytes(res_b.into_body(), usize::MAX).await.unwrap()).unwrap();
    let arr_b = list_b.as_array().unwrap();
    assert!(arr_b.iter().all(|l| l["tenant_id"] == tenant_b.to_string()));
    assert!(arr_b.iter().any(|l| l["id"] == log_b.id.to_string()));
    assert!(!arr_b.iter().any(|l| l["id"] == log_a.id.to_string()));
}

// 3. Unauthorized / unauthenticated request fails
#[tokio::test]
async fn test_audit_logs_unauthenticated() {
    let (app, _state, _pool) = setup_test_app().await;

    // Missing auth header
    let req = Request::builder()
        .uri("/v1/audit-logs")
        .method("GET")
        .body(Body::empty())
        .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);

    // Invalid key
    let req_invalid = Request::builder()
        .uri("/v1/audit-logs")
        .method("GET")
        .header(header::AUTHORIZATION, "Bearer invalid_nonexistent_token_999")
        .body(Body::empty())
        .unwrap();

    let res_invalid = app.oneshot(req_invalid).await.unwrap();
    assert_eq!(res_invalid.status(), StatusCode::UNAUTHORIZED);
}

// 4. Filtering
#[tokio::test]
async fn test_audit_logs_filtering() {
    let (app, state, _pool) = setup_test_app().await;
    let (tenant_id, raw_key) = create_test_tenant_and_key(&state, "audit-filt").await;

    let res_id_1 = Uuid::new_v4();
    let res_id_2 = Uuid::new_v4();

    let _ = state
        .audit_service
        .create_audit_log(
            tenant_id,
            None,
            "destination.created",
            Some("destination"),
            Some(res_id_1),
            serde_json::json!({}),
        )
        .await
        .unwrap();

    let _ = state
        .audit_service
        .create_audit_log(
            tenant_id,
            None,
            "source.created",
            Some("source"),
            Some(res_id_2),
            serde_json::json!({}),
        )
        .await
        .unwrap();

    // Filter by action=destination.created
    let req_filter = Request::builder()
        .uri("/v1/audit-logs?action=destination.created")
        .method("GET")
        .header(header::AUTHORIZATION, format!("Bearer {raw_key}"))
        .body(Body::empty())
        .unwrap();

    let res_filter = app.clone().oneshot(req_filter).await.unwrap();
    assert_eq!(res_filter.status(), StatusCode::OK);
    let list: Value = serde_json::from_slice(&axum::body::to_bytes(res_filter.into_body(), usize::MAX).await.unwrap()).unwrap();
    let arr = list.as_array().unwrap();
    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0]["action"], "destination.created");
    assert_eq!(arr[0]["resource_id"], res_id_1.to_string());

    // Filter by resource_type=source
    let req_res = Request::builder()
        .uri("/v1/audit-logs?resource_type=source")
        .method("GET")
        .header(header::AUTHORIZATION, format!("Bearer {raw_key}"))
        .body(Body::empty())
        .unwrap();

    let res_res = app.oneshot(req_res).await.unwrap();
    assert_eq!(res_res.status(), StatusCode::OK);
    let list_res: Value = serde_json::from_slice(&axum::body::to_bytes(res_res.into_body(), usize::MAX).await.unwrap()).unwrap();
    let arr_res = list_res.as_array().unwrap();
    assert_eq!(arr_res.len(), 1);
    assert_eq!(arr_res[0]["action"], "source.created");
    assert_eq!(arr_res[0]["resource_id"], res_id_2.to_string());
}

// 5. Pagination
#[tokio::test]
async fn test_audit_logs_pagination() {
    let (app, state, _pool) = setup_test_app().await;
    let (tenant_id, raw_key) = create_test_tenant_and_key(&state, "audit-pag").await;

    for i in 1..=5 {
        let _ = state
            .audit_service
            .create_audit_log(
                tenant_id,
                None,
                &format!("action.{i}"),
                Some("resource"),
                None,
                serde_json::json!({"index": i}),
            )
            .await
            .unwrap();
    }

    // Page 1: limit 2, offset 0
    let req1 = Request::builder()
        .uri("/v1/audit-logs?limit=2&offset=0")
        .method("GET")
        .header(header::AUTHORIZATION, format!("Bearer {raw_key}"))
        .body(Body::empty())
        .unwrap();

    let res1 = app.clone().oneshot(req1).await.unwrap();
    assert_eq!(res1.status(), StatusCode::OK);
    let list1: Value = serde_json::from_slice(&axum::body::to_bytes(res1.into_body(), usize::MAX).await.unwrap()).unwrap();
    assert_eq!(list1.as_array().unwrap().len(), 2);

    // Page 2: limit 2, offset 2
    let req2 = Request::builder()
        .uri("/v1/audit-logs?limit=2&offset=2")
        .method("GET")
        .header(header::AUTHORIZATION, format!("Bearer {raw_key}"))
        .body(Body::empty())
        .unwrap();

    let res2 = app.oneshot(req2).await.unwrap();
    assert_eq!(res2.status(), StatusCode::OK);
    let list2: Value = serde_json::from_slice(&axum::body::to_bytes(res2.into_body(), usize::MAX).await.unwrap()).unwrap();
    assert_eq!(list2.as_array().unwrap().len(), 2);
}

// 6. No sensitive secrets exposed
#[tokio::test]
async fn test_audit_logs_no_sensitive_secrets_exposed() {
    let (app, state, _pool) = setup_test_app().await;
    let (tenant_id, raw_key) = create_test_tenant_and_key(&state, "audit-sec").await;

    let _ = state
        .audit_service
        .create_audit_log(
            tenant_id,
            None,
            "api_key.created",
            Some("api_key"),
            Some(Uuid::new_v4()),
            serde_json::json!({"key_prefix": "rc_live_abc"}),
        )
        .await
        .unwrap();

    let req = Request::builder()
        .uri("/v1/audit-logs")
        .method("GET")
        .header(header::AUTHORIZATION, format!("Bearer {raw_key}"))
        .body(Body::empty())
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    let body_str = String::from_utf8(axum::body::to_bytes(res.into_body(), usize::MAX).await.unwrap().to_vec()).unwrap();

    assert!(!body_str.contains("secret"));
    assert!(!body_str.contains("password"));
    assert!(!body_str.contains("key_hash"));
    assert!(!body_str.contains("jwt_secret"));
}
