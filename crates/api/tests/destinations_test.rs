use axum::{
    body::Body,
    http::{header, Request, StatusCode},
};
use serde_json::{json, Value};
use sqlx::PgPool;
use std::sync::Arc;
use tower::ServiceExt;
use uuid::Uuid;

use api::{create_router, AppState};
use relay_core::config::Config;
use data::pool::{create_pg_pool, run_migrations};
use data::queue::RedisQueue;
use domain::dto::{CreateApiKeyInput, CreateDestinationInput, CreateTenantInput};

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

// 1. Successful destination creation test
#[tokio::test]
async fn test_successful_destination_creation() {
    let (app, state, _pool) = setup_test_app().await;
    let (tenant_id, raw_key) = create_test_tenant_and_key(&state, "succ-dest").await;

    let payload = json!({
        "name": "Customer Webhook Endpoint",
        "url": "https://api.customer.com/webhooks/v1",
        "description": "Production webhook receiver",
        "rate_limit_rps": 50,
        "timeout_ms": 5000,
        "max_retries": 5,
        "headers": {
            "X-Custom-Auth": "custom-token-header"
        },
        "secret": "whsec_client_secret_xyz123"
    });

    let req = Request::builder()
        .uri("/v1/destinations")
        .method("POST")
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::AUTHORIZATION, format!("Bearer {raw_key}"))
        .body(Body::from(payload.to_string()))
        .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);

    let body_bytes = axum::body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let json_res: Value = serde_json::from_slice(&body_bytes).unwrap();

    assert_eq!(json_res["name"], "Customer Webhook Endpoint");
    assert_eq!(json_res["url"], "https://api.customer.com/webhooks/v1");
    assert_eq!(json_res["tenant_id"], tenant_id.to_string());
    assert_eq!(json_res["rate_limit_rps"], 50);
    assert_eq!(json_res["timeout_ms"], 5000);
    assert_eq!(json_res["is_active"], true);
    assert!(json_res["id"].is_string());
    assert!(json_res["created_at"].is_string());
    assert!(json_res["updated_at"].is_string());

    // 5. Secret is never returned
    assert!(json_res.get("secret").is_none());
    assert!(json_res.get("secret_encrypted").is_none());
    assert!(json_res.get("encrypted_secret").is_none());

    // Verify /api/v1/destinations alias works
    let payload2 = json!({
        "name": "Second Destination",
        "url": "https://hooks.slack.com/services/T00/B00/X00"
    });

    let req2 = Request::builder()
        .uri("/api/v1/destinations")
        .method("POST")
        .header(header::CONTENT_TYPE, "application/json")
        .header("X-Api-Key", &raw_key)
        .body(Body::from(payload2.to_string()))
        .unwrap();

    let res2 = app.oneshot(req2).await.unwrap();
    assert_eq!(res2.status(), StatusCode::CREATED);
}

// 2. Invalid URL test
#[tokio::test]
async fn test_invalid_destination_url() {
    let (app, state, _pool) = setup_test_app().await;
    let (_tenant_id, raw_key) = create_test_tenant_and_key(&state, "inv-url").await;

    let invalid_urls = vec![
        "not-a-valid-url",
        "ftp://example.com/webhooks",
        "javascript:alert(1)",
        "file:///etc/passwd",
        "http://",
        "https://",
        "http://169.254.169.254/latest/meta-data/", // SSRF cloud metadata
        "https://metadata.google.internal/computeMetadata/v1/",
    ];

    for url in invalid_urls {
        let payload = json!({
            "name": "Invalid URL Destination",
            "url": url
        });

        let req = Request::builder()
            .uri("/v1/destinations")
            .method("POST")
            .header(header::CONTENT_TYPE, "application/json")
            .header(header::AUTHORIZATION, format!("Bearer {raw_key}"))
            .body(Body::from(payload.to_string()))
            .unwrap();

        let res = app.clone().oneshot(req).await.unwrap();
        assert_eq!(
            res.status(),
            StatusCode::UNPROCESSABLE_ENTITY,
            "Expected 422 for invalid URL '{url}'"
        );
    }
}

// 3. Missing URL test
#[tokio::test]
async fn test_missing_destination_url() {
    let (app, state, _pool) = setup_test_app().await;
    let (_tenant_id, raw_key) = create_test_tenant_and_key(&state, "miss-url").await;

    // Empty string URL
    let payload1 = json!({
        "name": "Valid Name",
        "url": ""
    });

    let req1 = Request::builder()
        .uri("/v1/destinations")
        .method("POST")
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::AUTHORIZATION, format!("Bearer {raw_key}"))
        .body(Body::from(payload1.to_string()))
        .unwrap();

    let res1 = app.clone().oneshot(req1).await.unwrap();
    assert_eq!(res1.status(), StatusCode::UNPROCESSABLE_ENTITY);

    // Whitespace only URL
    let payload2 = json!({
        "name": "Valid Name",
        "url": "    "
    });

    let req2 = Request::builder()
        .uri("/v1/destinations")
        .method("POST")
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::AUTHORIZATION, format!("Bearer {raw_key}"))
        .body(Body::from(payload2.to_string()))
        .unwrap();

    let res2 = app.clone().oneshot(req2).await.unwrap();
    assert_eq!(res2.status(), StatusCode::UNPROCESSABLE_ENTITY);

    // Completely missing URL field in JSON
    let payload3 = json!({
        "name": "Missing URL field"
    });

    let req3 = Request::builder()
        .uri("/v1/destinations")
        .method("POST")
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::AUTHORIZATION, format!("Bearer {raw_key}"))
        .body(Body::from(payload3.to_string()))
        .unwrap();

    let res3 = app.oneshot(req3).await.unwrap();
    assert!(res3.status().is_client_error());
}

// 4. Unauthenticated request test
#[tokio::test]
async fn test_unauthenticated_destination_request() {
    let (app, _state, _pool) = setup_test_app().await;

    let payload = json!({
        "name": "Test Destination",
        "url": "https://example.com/webhooks"
    });

    // Missing auth header
    let req = Request::builder()
        .uri("/v1/destinations")
        .method("POST")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(payload.to_string()))
        .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);

    // Invalid API key
    let req_invalid = Request::builder()
        .uri("/v1/destinations")
        .method("POST")
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::AUTHORIZATION, "Bearer invalid_nonexistent_key")
        .body(Body::from(payload.to_string()))
        .unwrap();

    let res_invalid = app.oneshot(req_invalid).await.unwrap();
    assert_eq!(res_invalid.status(), StatusCode::UNAUTHORIZED);
}

// 6. Tenant isolation test
#[tokio::test]
async fn test_destination_tenant_isolation() {
    let (app, state, _pool) = setup_test_app().await;
    let (tenant_a, key_a) = create_test_tenant_and_key(&state, "dest-iso-a").await;
    let (tenant_b, key_b) = create_test_tenant_and_key(&state, "dest-iso-b").await;

    // Tenant A attempts to spoof tenant_id by injecting tenant B's ID in request body
    let req = Request::builder()
        .uri("/v1/destinations")
        .method("POST")
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::AUTHORIZATION, format!("Bearer {key_a}"))
        .body(Body::from(
            json!({
                "name": "Spoofed Destination",
                "url": "https://tenant-a.com/webhooks",
                "tenant_id": tenant_b.to_string()
            })
            .to_string(),
        ))
        .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);

    let body: Value = serde_json::from_slice(&axum::body::to_bytes(res.into_body(), usize::MAX).await.unwrap()).unwrap();
    // Security check: tenant_id MUST strictly be Tenant A (from auth token), not Tenant B!
    assert_eq!(body["tenant_id"], tenant_a.to_string());
    assert_ne!(body["tenant_id"], tenant_b.to_string());

    // Verify Tenant B listing destinations does NOT see Tenant A's destination
    let list_req = Request::builder()
        .uri("/v1/destinations")
        .method("GET")
        .header(header::AUTHORIZATION, format!("Bearer {key_b}"))
        .body(Body::empty())
        .unwrap();

    let list_res = app.oneshot(list_req).await.unwrap();
    assert_eq!(list_res.status(), StatusCode::OK);

    let list_body: Value = serde_json::from_slice(&axum::body::to_bytes(list_res.into_body(), usize::MAX).await.unwrap()).unwrap();
    let dests_list = list_body.as_array().unwrap();
    for d in dests_list {
        assert_eq!(d["tenant_id"], tenant_b.to_string());
        assert_ne!(d["name"], "Spoofed Destination");
    }
}

// 7. Database failure test
#[tokio::test]
async fn test_destination_database_failure_handling() {
    let (_, state, _pool) = setup_test_app().await;
    let (tenant_id, _) = create_test_tenant_and_key(&state, "dest-db-fail").await;

    let invalid_pool = Arc::new(
        sqlx::postgres::PgPoolOptions::new()
            .acquire_timeout(std::time::Duration::from_millis(10))
            .connect_lazy("postgres://invalid:invalid@localhost:9999/nonexistent")
            .unwrap(),
    );
    let broken_destination_service = domain::services::DestinationService::new(invalid_pool, [0u8; 32]);

    let res = broken_destination_service
        .create_destination(
            tenant_id,
            CreateDestinationInput {
                name: "Test Destination".to_string(),
                url: "https://example.com/webhooks".to_string(),
                description: None,
                rate_limit_rps: None,
                timeout_ms: None,
                secret: None,
                max_retries: None,
                headers: None,
            },
        )
        .await;

    assert!(res.is_err());
    match res.unwrap_err() {
        relay_core::error::CoreError::Internal(msg) => {
            assert!(msg.contains("Database error") || msg.contains("connection"));
        }
        err => panic!("Expected CoreError::Internal, got {:?}", err),
    }
}

// --- GET /v1/destinations Tests ---

// 1. Tenant receives its destinations
#[tokio::test]
async fn test_get_destinations_tenant_receives_its_destinations() {
    let (app, state, _pool) = setup_test_app().await;
    let (tenant_id, raw_key) = create_test_tenant_and_key(&state, "get-dest").await;

    // Create 3 destinations
    for i in 1..=3 {
        let _ = state
            .destination_service
            .create_destination(
                tenant_id,
                CreateDestinationInput {
                    name: format!("Destination {i}"),
                    url: format!("https://api.example.com/dest-{i}"),
                    description: Some(format!("Description {i}")),
                    rate_limit_rps: Some(25 * i),
                    timeout_ms: Some(5000 + i * 1000),
                    secret: Some(format!("secret-{i}")),
                    max_retries: Some(5),
                    headers: Some(json!({"X-Header": format!("Val-{i}")})),
                },
            )
            .await
            .unwrap();
    }

    // Call GET /v1/destinations
    let req = Request::builder()
        .uri("/v1/destinations?limit=10&offset=0")
        .method("GET")
        .header(header::AUTHORIZATION, format!("Bearer {raw_key}"))
        .body(Body::empty())
        .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let body_bytes = axum::body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let json_res: Value = serde_json::from_slice(&body_bytes).unwrap();
    let list = json_res.as_array().expect("Expected JSON array");

    assert_eq!(list.len(), 3);
    for item in list {
        assert_eq!(item["tenant_id"], tenant_id.to_string());
        assert!(item["id"].is_string());
        assert!(item["name"].is_string());
        assert!(item["url"].is_string());
        assert_eq!(item["is_active"], true);
    }

    // Also verify /api/v1/destinations route
    let req_api = Request::builder()
        .uri("/api/v1/destinations")
        .method("GET")
        .header("X-Api-Key", &raw_key)
        .body(Body::empty())
        .unwrap();

    let res_api = app.oneshot(req_api).await.unwrap();
    assert_eq!(res_api.status(), StatusCode::OK);
    let list_api: Value = serde_json::from_slice(&axum::body::to_bytes(res_api.into_body(), usize::MAX).await.unwrap()).unwrap();
    assert_eq!(list_api.as_array().unwrap().len(), 3);
}

// 2. Empty tenant returns empty collection
#[tokio::test]
async fn test_get_destinations_empty_tenant_returns_empty() {
    let (app, state, _pool) = setup_test_app().await;
    let (_tenant_id, raw_key) = create_test_tenant_and_key(&state, "get-dest-empty").await;

    let req = Request::builder()
        .uri("/v1/destinations")
        .method("GET")
        .header(header::AUTHORIZATION, format!("Bearer {raw_key}"))
        .body(Body::empty())
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let body_bytes = axum::body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let json_res: Value = serde_json::from_slice(&body_bytes).unwrap();
    let list = json_res.as_array().expect("Expected JSON array");

    assert_eq!(list.len(), 0);
}

// 3. Another tenant's destinations are excluded (multi-tenant isolation)
#[tokio::test]
async fn test_get_destinations_cross_tenant_isolation() {
    let (app, state, _pool) = setup_test_app().await;
    let (tenant_a, key_a) = create_test_tenant_and_key(&state, "dest-cross-a").await;
    let (tenant_b, key_b) = create_test_tenant_and_key(&state, "dest-cross-b").await;

    // Create 2 destinations for Tenant A
    let _ = state.destination_service.create_destination(tenant_a, CreateDestinationInput {
        name: "Dest A1".to_string(),
        url: "https://tenant-a.com/d1".to_string(),
        description: None,
        rate_limit_rps: None,
        timeout_ms: None,
        secret: None,
        max_retries: None,
        headers: None,
    }).await.unwrap();
    let _ = state.destination_service.create_destination(tenant_a, CreateDestinationInput {
        name: "Dest A2".to_string(),
        url: "https://tenant-a.com/d2".to_string(),
        description: None,
        rate_limit_rps: None,
        timeout_ms: None,
        secret: None,
        max_retries: None,
        headers: None,
    }).await.unwrap();

    // Create 1 destination for Tenant B
    let _ = state.destination_service.create_destination(tenant_b, CreateDestinationInput {
        name: "Dest B1".to_string(),
        url: "https://tenant-b.com/d1".to_string(),
        description: None,
        rate_limit_rps: None,
        timeout_ms: None,
        secret: None,
        max_retries: None,
        headers: None,
    }).await.unwrap();

    // Query as Tenant A
    let req_a = Request::builder()
        .uri("/v1/destinations")
        .method("GET")
        .header(header::AUTHORIZATION, format!("Bearer {key_a}"))
        .body(Body::empty())
        .unwrap();

    let res_a = app.clone().oneshot(req_a).await.unwrap();
    assert_eq!(res_a.status(), StatusCode::OK);
    let list_a: Value = serde_json::from_slice(&axum::body::to_bytes(res_a.into_body(), usize::MAX).await.unwrap()).unwrap();
    let arr_a = list_a.as_array().unwrap();
    assert_eq!(arr_a.len(), 2);
    for item in arr_a {
        assert_eq!(item["tenant_id"], tenant_a.to_string());
        assert_ne!(item["name"], "Dest B1");
    }

    // Query as Tenant B
    let req_b = Request::builder()
        .uri("/v1/destinations")
        .method("GET")
        .header(header::AUTHORIZATION, format!("Bearer {key_b}"))
        .body(Body::empty())
        .unwrap();

    let res_b = app.clone().oneshot(req_b).await.unwrap();
    assert_eq!(res_b.status(), StatusCode::OK);
    let list_b: Value = serde_json::from_slice(&axum::body::to_bytes(res_b.into_body(), usize::MAX).await.unwrap()).unwrap();
    let arr_b = list_b.as_array().unwrap();
    assert_eq!(arr_b.len(), 1);
    assert_eq!(arr_b[0]["name"], "Dest B1");
    assert_eq!(arr_b[0]["tenant_id"], tenant_b.to_string());

    // Spoofed query param ?tenant_id=<tenant_a> as Tenant B
    let req_spoof = Request::builder()
        .uri(format!("/v1/destinations?tenant_id={tenant_a}"))
        .method("GET")
        .header(header::AUTHORIZATION, format!("Bearer {key_b}"))
        .body(Body::empty())
        .unwrap();

    let res_spoof = app.oneshot(req_spoof).await.unwrap();
    assert_eq!(res_spoof.status(), StatusCode::OK);
    let list_spoof: Value = serde_json::from_slice(&axum::body::to_bytes(res_spoof.into_body(), usize::MAX).await.unwrap()).unwrap();
    let arr_spoof = list_spoof.as_array().unwrap();
    assert_eq!(arr_spoof.len(), 1);
    assert_eq!(arr_spoof[0]["name"], "Dest B1");
}

// 4. Unauthenticated request fails
#[tokio::test]
async fn test_get_destinations_unauthenticated_fails() {
    let (app, _state, _pool) = setup_test_app().await;

    // Missing auth header
    let req = Request::builder()
        .uri("/v1/destinations")
        .method("GET")
        .body(Body::empty())
        .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);

    // Invalid API key
    let req_invalid = Request::builder()
        .uri("/v1/destinations")
        .method("GET")
        .header(header::AUTHORIZATION, "Bearer invalid_nonexistent_key")
        .body(Body::empty())
        .unwrap();

    let res_invalid = app.oneshot(req_invalid).await.unwrap();
    assert_eq!(res_invalid.status(), StatusCode::UNAUTHORIZED);
}

// 5. Sensitive fields are not returned
#[tokio::test]
async fn test_get_destinations_sensitive_fields_not_returned() {
    let (app, state, _pool) = setup_test_app().await;
    let (tenant_id, raw_key) = create_test_tenant_and_key(&state, "get-dest-sens").await;

    let _ = state
        .destination_service
        .create_destination(
            tenant_id,
            CreateDestinationInput {
                name: "Secure Destination".to_string(),
                url: "https://secure.example.com/webhook".to_string(),
                description: None,
                rate_limit_rps: None,
                timeout_ms: None,
                secret: Some("whsec_super_secret_dest_key".to_string()),
                max_retries: Some(5),
                headers: Some(json!({"Authorization": "Bearer internal_token"})),
            },
        )
        .await
        .unwrap();

    let req = Request::builder()
        .uri("/v1/destinations")
        .method("GET")
        .header(header::AUTHORIZATION, format!("Bearer {raw_key}"))
        .body(Body::empty())
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let body_bytes = axum::body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let json_res: Value = serde_json::from_slice(&body_bytes).unwrap();
    let list = json_res.as_array().expect("Expected JSON array");

    assert_eq!(list.len(), 1);
    let item = &list[0];

    assert_eq!(item["name"], "Secure Destination");
    assert_eq!(item["tenant_id"], tenant_id.to_string());

    // Verify sensitive keys are NOT in the response
    assert!(item.get("secret").is_none());
    assert!(item.get("secret_encrypted").is_none());
    assert!(item.get("encrypted_secret").is_none());
}

// 6. Database failure is handled
#[tokio::test]
async fn test_get_destinations_database_failure() {
    let (_app, state, _pool) = setup_test_app().await;
    let (tenant_id, _) = create_test_tenant_and_key(&state, "get-dest-db-fail").await;

    let invalid_pool = Arc::new(
        sqlx::postgres::PgPoolOptions::new()
            .acquire_timeout(std::time::Duration::from_millis(10))
            .connect_lazy("postgres://invalid:invalid@localhost:9999/nonexistent")
            .unwrap(),
    );
    let broken_destination_service = domain::services::DestinationService::new(invalid_pool, [0u8; 32]);

    let res = broken_destination_service.list_destinations(tenant_id, 20, 0).await;
    assert!(res.is_err());
    match res.unwrap_err() {
        relay_core::error::CoreError::Internal(msg) => {
            assert!(msg.contains("Database error") || msg.contains("connection"));
        }
        err => panic!("Expected CoreError::Internal, got {:?}", err),
    }
}
