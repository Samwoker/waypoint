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

    // Secret is returned once on creation
    assert_eq!(json_res["secret"], "whsec_client_secret_xyz123");
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

// 7. GET /destinations/{destination_id} (Endpoint #27)
#[tokio::test]
async fn test_get_destination_by_id_existing() {
    let (app, state, _pool) = setup_test_app().await;
    let (tenant_id, raw_key) = create_test_tenant_and_key(&state, "get-dest-by-id").await;

    let dest = state
        .destination_service
        .create_destination(
            tenant_id,
            CreateDestinationInput {
                name: "Get Single Dest".to_string(),
                url: "https://get-single.example.com/hooks".to_string(),
                description: Some("Single destination test".to_string()),
                rate_limit_rps: Some(25),
                timeout_ms: Some(8000),
                max_retries: Some(7),
                headers: None,
                secret: None,
            },
        )
        .await
        .unwrap();

    let req = Request::builder()
        .uri(format!("/destinations/{}", dest.id))
        .method("GET")
        .header(header::AUTHORIZATION, format!("Bearer {raw_key}"))
        .body(Body::empty())
        .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let body_bytes = axum::body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let json_res: Value = serde_json::from_slice(&body_bytes).unwrap();

    assert_eq!(json_res["id"], dest.id.to_string());
    assert_eq!(json_res["name"], "Get Single Dest");
    assert_eq!(json_res["status"], "active");
    assert_eq!(json_res["consecutive_failures"], 0);
    assert_eq!(json_res["max_retries"], 7);
    assert_eq!(json_res["timeout_ms"], 8000);
    assert_eq!(json_res["retry_backoff_strategy"], "exponential");
    assert!(json_res.get("secret").is_none());
    assert!(json_res.get("secret_encrypted").is_none());

    // Also test /v1/destinations/{id}
    let req_v1 = Request::builder()
        .uri(format!("/v1/destinations/{}", dest.id))
        .method("GET")
        .header(header::AUTHORIZATION, format!("Bearer {raw_key}"))
        .body(Body::empty())
        .unwrap();

    let res_v1 = app.oneshot(req_v1).await.unwrap();
    assert_eq!(res_v1.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_get_destination_by_id_nonexistent() {
    let (app, state, _pool) = setup_test_app().await;
    let (_tenant_id, raw_key) = create_test_tenant_and_key(&state, "get-dest-none").await;

    let req = Request::builder()
        .uri(format!("/destinations/{}", Uuid::new_v4()))
        .method("GET")
        .header(header::AUTHORIZATION, format!("Bearer {raw_key}"))
        .body(Body::empty())
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_get_destination_by_id_cross_tenant() {
    let (app, state, _pool) = setup_test_app().await;
    let (tenant_a, _key_a) = create_test_tenant_and_key(&state, "dest-cross-a").await;
    let (_tenant_b, key_b) = create_test_tenant_and_key(&state, "dest-cross-b").await;

    let dest_a = state
        .destination_service
        .create_destination(
            tenant_a,
            CreateDestinationInput {
                name: "Tenant A Dest".to_string(),
                url: "https://tenant-a.com/hook".to_string(),
                description: None,
                rate_limit_rps: None,
                timeout_ms: None,
                max_retries: None,
                headers: None,
                secret: None,
            },
        )
        .await
        .unwrap();

    let req = Request::builder()
        .uri(format!("/destinations/{}", dest_a.id))
        .method("GET")
        .header(header::AUTHORIZATION, format!("Bearer {key_b}"))
        .body(Body::empty())
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}

// 8. POST /destinations (Endpoint #26)
#[tokio::test]
async fn test_post_destination_secret_generation_and_encryption() {
    let (app, state, pool) = setup_test_app().await;
    let (tenant_id, raw_key) = create_test_tenant_and_key(&state, "post-dest-sec").await;

    let req = Request::builder()
        .uri("/destinations")
        .method("POST")
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::AUTHORIZATION, format!("Bearer {raw_key}"))
        .body(Body::from(
            json!({
                "name": "Auto Secret Dest",
                "url": "https://auto-sec.example.com/webhook",
                "max_retries": 10,
                "timeout_ms": 10000
            })
            .to_string(),
        ))
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);

    let body_bytes = axum::body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let json_res: Value = serde_json::from_slice(&body_bytes).unwrap();

    assert!(json_res["secret"].is_string());
    let plaintext_secret = json_res["secret"].as_str().unwrap();
    assert!(!plaintext_secret.is_empty());

    // Verify secret is NOT stored plaintext in PostgreSQL
    let dest_id = Uuid::parse_str(json_res["id"].as_str().unwrap()).unwrap();
    let dest_repo = data::repositories::DestinationRepository::new(&pool);
    let db_dest = dest_repo.find_by_tenant_and_id(tenant_id, dest_id).await.unwrap().unwrap();

    assert!(db_dest.secret_encrypted.is_some());
    let encrypted_in_db = db_dest.secret_encrypted.unwrap();
    assert_ne!(encrypted_in_db, plaintext_secret);

    // Verify decryption matches
    let decrypted_bytes = relay_core::crypto::decrypt_secret(&encrypted_in_db, &state.destination_service.encryption_key).unwrap();
    let decrypted_str = String::from_utf8(decrypted_bytes).unwrap();
    assert_eq!(decrypted_str, plaintext_secret);
}

// 9. PATCH /destinations/{destination_id} (Endpoint #28)
#[tokio::test]
async fn test_patch_destination_updates_and_url_health_reset() {
    let (app, state, pool) = setup_test_app().await;
    let (tenant_id, raw_key) = create_test_tenant_and_key(&state, "patch-dest").await;

    let dest = state
        .destination_service
        .create_destination(
            tenant_id,
            CreateDestinationInput {
                name: "Original Dest".to_string(),
                url: "https://orig.example.com/hook".to_string(),
                description: None,
                rate_limit_rps: None,
                timeout_ms: Some(5000),
                max_retries: Some(3),
                headers: None,
                secret: None,
            },
        )
        .await
        .unwrap();

    // Artificially simulate circuit break / consecutive failures in DB metadata
    sqlx::query(
        r#"
        UPDATE destinations
        SET metadata = metadata || jsonb_build_object('consecutive_failures', 5, 'circuit_opened_at', NOW())
        WHERE id = $1
        "#,
    )
    .bind(dest.id)
    .execute(&pool)
    .await
    .unwrap();

    // Verify failures are 5
    let before = state.destination_service.get_destination(tenant_id, dest.id).await.unwrap().unwrap();
    assert_eq!(before.consecutive_failures, 5);
    assert!(before.circuit_opened_at.is_some());

    // Patch URL -> should reset consecutive_failures = 0 and circuit_opened_at = None
    let patch_payload = json!({
        "name": "Updated Dest Name",
        "url": "https://new-url.example.com/hook",
        "max_retries": 15,
        "timeout_ms": 12000,
        "retry_backoff_strategy": "linear"
    });

    let req = Request::builder()
        .uri(format!("/destinations/{}", dest.id))
        .method("PATCH")
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::AUTHORIZATION, format!("Bearer {raw_key}"))
        .body(Body::from(patch_payload.to_string()))
        .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let body_bytes = axum::body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let json_res: Value = serde_json::from_slice(&body_bytes).unwrap();

    assert_eq!(json_res["name"], "Updated Dest Name");
    assert_eq!(json_res["url"], "https://new-url.example.com/hook");
    assert_eq!(json_res["max_retries"], 15);
    assert_eq!(json_res["timeout_ms"], 12000);
    assert_eq!(json_res["retry_backoff_strategy"], "linear");
    assert_eq!(json_res["consecutive_failures"], 0);
    assert!(json_res["circuit_opened_at"].is_null());

    // Direct secret mutation attempt is rejected
    let reject_req = Request::builder()
        .uri(format!("/destinations/{}", dest.id))
        .method("PATCH")
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::AUTHORIZATION, format!("Bearer {raw_key}"))
        .body(Body::from(json!({"secret": "hacked_secret"}).to_string()))
        .unwrap();

    let reject_res = app.oneshot(reject_req).await.unwrap();
    assert_eq!(reject_res.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

// 10. DELETE /destinations/{destination_id} (Endpoint #29)
#[tokio::test]
async fn test_delete_destination_soft_delete_preserves_history() {
    let (app, state, pool) = setup_test_app().await;
    let (tenant_id, raw_key) = create_test_tenant_and_key(&state, "del-dest").await;

    let dest = state
        .destination_service
        .create_destination(
            tenant_id,
            CreateDestinationInput {
                name: "To Delete".to_string(),
                url: "https://delete.example.com/hook".to_string(),
                description: None,
                rate_limit_rps: None,
                timeout_ms: None,
                max_retries: None,
                headers: None,
                secret: None,
            },
        )
        .await
        .unwrap();

    // Create a source & event & delivery for historical preservation test
    let source = state
        .source_service
        .create_source(
            tenant_id,
            domain::dto::CreateSourceInput {
                name: "Del Source".to_string(),
                slug: format!("del-src-{}", Uuid::new_v4().simple()),
                description: None,
                provider: "generic".to_string(),
                verification_type: "none".to_string(),
                secret: None,
            },
        )
        .await
        .unwrap();

    let event = data::repositories::EventRepository::new(&pool)
        .create(tenant_id, source.id, "test.del", None, json!({}), json!({"test": 1}))
        .await
        .unwrap();

    let sub = state
        .subscription_service
        .create_subscription(
            tenant_id,
            domain::dto::CreateSubscriptionInput {
                source_id: source.id,
                destination_id: dest.id,
                event_types: vec!["*".to_string()],
                filter_rules: None,
                transformation_template: None,
            },
        )
        .await
        .unwrap();

    let delivery = data::repositories::DeliveryRepository::new(&pool)
        .create(tenant_id, event.id, sub.id, dest.id, 5)
        .await
        .unwrap();

    let req = Request::builder()
        .uri(format!("/destinations/{}", dest.id))
        .method("DELETE")
        .header(header::AUTHORIZATION, format!("Bearer {raw_key}"))
        .body(Body::empty())
        .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::NO_CONTENT);

    // Verify GET returns 404
    let get_req = Request::builder()
        .uri(format!("/destinations/{}", dest.id))
        .method("GET")
        .header(header::AUTHORIZATION, format!("Bearer {raw_key}"))
        .body(Body::empty())
        .unwrap();

    let get_res = app.oneshot(get_req).await.unwrap();
    assert_eq!(get_res.status(), StatusCode::NOT_FOUND);

    // Verify delivery history remains intact in DB
    let deliv_repo = data::repositories::DeliveryRepository::new(&pool);
    let historical_delivery = deliv_repo.find_by_tenant_and_id(tenant_id, delivery.id).await.unwrap();
    assert!(historical_delivery.is_some());
    assert_eq!(historical_delivery.unwrap().destination_id, dest.id);
}

// 11. POST /destinations/{destination_id}/pause (Endpoint #30)
#[tokio::test]
async fn test_pause_destination_active_and_skip_logic() {
    let (app, state, _pool) = setup_test_app().await;
    let (tenant_id, raw_key) = create_test_tenant_and_key(&state, "pause-dest").await;

    let dest = state
        .destination_service
        .create_destination(
            tenant_id,
            CreateDestinationInput {
                name: "Pause Dest".to_string(),
                url: "https://pause.example.com/hook".to_string(),
                description: None,
                rate_limit_rps: None,
                timeout_ms: None,
                max_retries: None,
                headers: None,
                secret: None,
            },
        )
        .await
        .unwrap();

    let req = Request::builder()
        .uri(format!("/destinations/{}/pause", dest.id))
        .method("POST")
        .header(header::AUTHORIZATION, format!("Bearer {raw_key}"))
        .body(Body::empty())
        .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let body_bytes = axum::body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let json_res: Value = serde_json::from_slice(&body_bytes).unwrap();

    assert_eq!(json_res["status"], "paused");
    assert_eq!(json_res["is_active"], false);

    // Pausing again is idempotent
    let req2 = Request::builder()
        .uri(format!("/destinations/{}/pause", dest.id))
        .method("POST")
        .header(header::AUTHORIZATION, format!("Bearer {raw_key}"))
        .body(Body::empty())
        .unwrap();

    let res2 = app.oneshot(req2).await.unwrap();
    assert_eq!(res2.status(), StatusCode::OK);
}

// 12. POST /destinations/{destination_id}/resume (Endpoint #31)
#[tokio::test]
async fn test_resume_destination_resets_circuit_and_reschedules_deliveries() {
    let (app, state, pool) = setup_test_app().await;
    let (tenant_id, raw_key) = create_test_tenant_and_key(&state, "resume-dest").await;

    let dest = state
        .destination_service
        .create_destination(
            tenant_id,
            CreateDestinationInput {
                name: "Resume Dest".to_string(),
                url: "https://resume.example.com/hook".to_string(),
                description: None,
                rate_limit_rps: None,
                timeout_ms: None,
                max_retries: None,
                headers: None,
                secret: None,
            },
        )
        .await
        .unwrap();

    // Pause it first
    state.destination_service.pause_destination(tenant_id, dest.id).await.unwrap();

    // Create a delayed delivery
    let source = state
        .source_service
        .create_source(
            tenant_id,
            domain::dto::CreateSourceInput {
                name: "Resume Source".to_string(),
                slug: format!("res-src-{}", Uuid::new_v4().simple()),
                description: None,
                provider: "generic".to_string(),
                verification_type: "none".to_string(),
                secret: None,
            },
        )
        .await
        .unwrap();

    let event = data::repositories::EventRepository::new(&pool)
        .create(tenant_id, source.id, "test.res", None, json!({}), json!({}))
        .await
        .unwrap();

    let sub = state
        .subscription_service
        .create_subscription(
            tenant_id,
            domain::dto::CreateSubscriptionInput {
                source_id: source.id,
                destination_id: dest.id,
                event_types: vec!["*".to_string()],
                filter_rules: None,
                transformation_template: None,
            },
        )
        .await
        .unwrap();

    let delivery = data::repositories::DeliveryRepository::new(&pool)
        .create(tenant_id, event.id, sub.id, dest.id, 5)
        .await
        .unwrap();

    // Set delivery next_attempt_at in the far future
    sqlx::query(
        r#"
        UPDATE deliveries
        SET next_attempt_at = NOW() + interval '1 day', status = 'failed'
        WHERE id = $1
        "#,
    )
    .bind(delivery.id)
    .execute(&pool)
    .await
    .unwrap();

    // Resume destination
    let req = Request::builder()
        .uri(format!("/destinations/{}/resume", dest.id))
        .method("POST")
        .header(header::AUTHORIZATION, format!("Bearer {raw_key}"))
        .body(Body::empty())
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let body_bytes = axum::body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let json_res: Value = serde_json::from_slice(&body_bytes).unwrap();

    assert_eq!(json_res["status"], "active");
    assert_eq!(json_res["is_active"], true);
    assert_eq!(json_res["consecutive_failures"], 0);
    assert!(json_res["circuit_opened_at"].is_null());

    // Verify delivery was rescheduled for immediate poll (next_attempt_at <= NOW)
    let deliv_repo = data::repositories::DeliveryRepository::new(&pool);
    let updated_delivery = deliv_repo.find_by_tenant_and_id(tenant_id, delivery.id).await.unwrap().unwrap();
    assert_eq!(updated_delivery.status, "pending");
    assert!(updated_delivery.next_retry_at <= Some(chrono::Utc::now()));
}

// 13. POST /destinations/{destination_id}/test (Endpoint #32)
async fn spawn_mock_receiver(status: StatusCode, delay: Option<std::time::Duration>) -> (String, tokio::task::JoinHandle<()>) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let app = axum::Router::new().route(
        "/webhook",
        axum::routing::post(move || async move {
            if let Some(d) = delay {
                tokio::time::sleep(d).await;
            }
            status
        }),
    );

    let handle = tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    (format!("http://127.0.0.1:{port}/webhook"), handle)
}

#[tokio::test]
async fn test_destination_test_endpoint_success_and_failures() {
    let (app, state, pool) = setup_test_app().await;
    let (tenant_id, raw_key) = create_test_tenant_and_key(&state, "test-dest-ep").await;

    // 1. Successful destination
    let (mock_url_200, _h1) = spawn_mock_receiver(StatusCode::OK, None).await;
    let dest_200 = state
        .destination_service
        .create_destination(
            tenant_id,
            CreateDestinationInput {
                name: "Mock 200 Dest".to_string(),
                url: mock_url_200,
                description: None,
                rate_limit_rps: None,
                timeout_ms: Some(2000),
                max_retries: None,
                headers: None,
                secret: Some("test_secret".to_string()),
            },
        )
        .await
        .unwrap();

    let req_200 = Request::builder()
        .uri(format!("/destinations/{}/test", dest_200.id))
        .method("POST")
        .header(header::AUTHORIZATION, format!("Bearer {raw_key}"))
        .body(Body::empty())
        .unwrap();

    let res_200 = app.clone().oneshot(req_200).await.unwrap();
    assert_eq!(res_200.status(), StatusCode::OK);

    let body_bytes = axum::body::to_bytes(res_200.into_body(), usize::MAX).await.unwrap();
    let json_res: Value = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(json_res["success"], true);
    assert_eq!(json_res["http_status"], 200);
    assert!(json_res["latency_ms"].as_i64().unwrap() >= 0);

    // 2. 500 error destination
    let (mock_url_500, _h2) = spawn_mock_receiver(StatusCode::INTERNAL_SERVER_ERROR, None).await;
    let dest_500 = state
        .destination_service
        .create_destination(
            tenant_id,
            CreateDestinationInput {
                name: "Mock 500 Dest".to_string(),
                url: mock_url_500,
                description: None,
                rate_limit_rps: None,
                timeout_ms: Some(2000),
                max_retries: None,
                headers: None,
                secret: None,
            },
        )
        .await
        .unwrap();

    let req_500 = Request::builder()
        .uri(format!("/destinations/{}/test", dest_500.id))
        .method("POST")
        .header(header::AUTHORIZATION, format!("Bearer {raw_key}"))
        .body(Body::empty())
        .unwrap();

    let res_500 = app.clone().oneshot(req_500).await.unwrap();
    assert_eq!(res_500.status(), StatusCode::OK);

    let body_bytes_500 = axum::body::to_bytes(res_500.into_body(), usize::MAX).await.unwrap();
    let json_res_500: Value = serde_json::from_slice(&body_bytes_500).unwrap();
    assert_eq!(json_res_500["success"], false);
    assert_eq!(json_res_500["http_status"], 500);

    // 3. Timeout destination
    let (mock_url_timeout, _h3) = spawn_mock_receiver(StatusCode::OK, Some(std::time::Duration::from_millis(500))).await;
    let dest_timeout = state
        .destination_service
        .create_destination(
            tenant_id,
            CreateDestinationInput {
                name: "Mock Timeout Dest".to_string(),
                url: mock_url_timeout,
                description: None,
                rate_limit_rps: None,
                timeout_ms: Some(50), // 50ms timeout < 500ms delay
                max_retries: None,
                headers: None,
                secret: None,
            },
        )
        .await
        .unwrap();

    let req_timeout = Request::builder()
        .uri(format!("/destinations/{}/test", dest_timeout.id))
        .method("POST")
        .header(header::AUTHORIZATION, format!("Bearer {raw_key}"))
        .body(Body::empty())
        .unwrap();

    let res_timeout = app.oneshot(req_timeout).await.unwrap();
    assert_eq!(res_timeout.status(), StatusCode::OK);

    let body_bytes_to = axum::body::to_bytes(res_timeout.into_body(), usize::MAX).await.unwrap();
    let json_res_to: Value = serde_json::from_slice(&body_bytes_to).unwrap();
    assert_eq!(json_res_to["success"], false);

    // Security & side-effect check: Verify no events or deliveries were created
    let event_count: (i64,) = sqlx::query_as("SELECT count(*) FROM events WHERE tenant_id = $1")
        .bind(tenant_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(event_count.0, 0);

    let delivery_count: (i64,) = sqlx::query_as("SELECT count(*) FROM deliveries WHERE tenant_id = $1")
        .bind(tenant_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(delivery_count.0, 0);
}

// 14. GET /destinations/{destination_id}/health (Endpoint #33)
#[tokio::test]
async fn test_destination_health_endpoint() {
    let (app, state, pool) = setup_test_app().await;
    let (tenant_id, raw_key) = create_test_tenant_and_key(&state, "health-dest").await;

    let dest = state
        .destination_service
        .create_destination(
            tenant_id,
            CreateDestinationInput {
                name: "Health Dest".to_string(),
                url: "https://health.example.com/hook".to_string(),
                description: None,
                rate_limit_rps: None,
                timeout_ms: None,
                max_retries: None,
                headers: None,
                secret: None,
            },
        )
        .await
        .unwrap();

    // 1. Initial health (no attempts) -> success_rate = 1.0
    let req1 = Request::builder()
        .uri(format!("/destinations/{}/health", dest.id))
        .method("GET")
        .header(header::AUTHORIZATION, format!("Bearer {raw_key}"))
        .body(Body::empty())
        .unwrap();

    let res1 = app.clone().oneshot(req1).await.unwrap();
    assert_eq!(res1.status(), StatusCode::OK);

    let body_bytes1 = axum::body::to_bytes(res1.into_body(), usize::MAX).await.unwrap();
    let json_res1: Value = serde_json::from_slice(&body_bytes1).unwrap();

    assert_eq!(json_res1["status"], "active");
    assert_eq!(json_res1["consecutive_failures"], 0);
    assert_eq!(json_res1["total_attempts"], 0);
    assert_eq!(json_res1["success_rate"], 1.0);

    // 2. Add 3 successful attempts and 1 failed attempt in the last hour
    let source = state
        .source_service
        .create_source(
            tenant_id,
            domain::dto::CreateSourceInput {
                name: "Health Source".to_string(),
                slug: format!("hlth-src-{}", Uuid::new_v4().simple()),
                description: None,
                provider: "generic".to_string(),
                verification_type: "none".to_string(),
                secret: None,
            },
        )
        .await
        .unwrap();

    let event = data::repositories::EventRepository::new(&pool)
        .create(tenant_id, source.id, "test.health", None, json!({}), json!({}))
        .await
        .unwrap();

    let sub = state
        .subscription_service
        .create_subscription(
            tenant_id,
            domain::dto::CreateSubscriptionInput {
                source_id: source.id,
                destination_id: dest.id,
                event_types: vec!["*".to_string()],
                filter_rules: None,
                transformation_template: None,
            },
        )
        .await
        .unwrap();

    let delivery = data::repositories::DeliveryRepository::new(&pool)
        .create(tenant_id, event.id, sub.id, dest.id, 5)
        .await
        .unwrap();

    let deliv_repo = data::repositories::DeliveryRepository::new(&pool);
    // 3 successes (200)
    for i in 1..=3 {
        deliv_repo
            .record_attempt(delivery.id, i, Some(200), None, None, None, None, None, Some(50))
            .await
            .unwrap();
    }
    // 1 failure (500)
    deliv_repo
        .record_attempt(delivery.id, 4, Some(500), None, None, None, None, Some("Internal Server Error"), Some(60))
        .await
        .unwrap();

    let req2 = Request::builder()
        .uri(format!("/destinations/{}/health", dest.id))
        .method("GET")
        .header(header::AUTHORIZATION, format!("Bearer {raw_key}"))
        .body(Body::empty())
        .unwrap();

    let res2 = app.oneshot(req2).await.unwrap();
    assert_eq!(res2.status(), StatusCode::OK);

    let body_bytes2 = axum::body::to_bytes(res2.into_body(), usize::MAX).await.unwrap();
    let json_res2: Value = serde_json::from_slice(&body_bytes2).unwrap();

    assert_eq!(json_res2["total_attempts"], 4);
    assert_eq!(json_res2["successful_attempts"], 3);
    assert_eq!(json_res2["success_rate"], 0.75);
}
