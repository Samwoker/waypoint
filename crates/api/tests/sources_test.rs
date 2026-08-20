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
use domain::dto::{CreateApiKeyInput, CreateSourceInput, CreateTenantInput};

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

// 1. Successful source creation test
#[tokio::test]
async fn test_successful_source_creation() {
    let (app, state, _pool) = setup_test_app().await;
    let (tenant_id, raw_key) = create_test_tenant_and_key(&state, "succ-src").await;

    let source_slug = format!("stripe-payments-{}", Uuid::new_v4().simple());
    let payload = json!({
        "name": "Stripe Payments",
        "slug": source_slug,
        "description": "Inbound stripe webhooks",
        "provider": "stripe",
        "verification_type": "hmac_sha256",
        "secret": "whsec_test_secret_12345"
    });

    let req = Request::builder()
        .uri("/v1/sources")
        .method("POST")
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::AUTHORIZATION, format!("Bearer {raw_key}"))
        .body(Body::from(payload.to_string()))
        .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);

    let body_bytes = axum::body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let json_res: Value = serde_json::from_slice(&body_bytes).unwrap();

    assert_eq!(json_res["name"], "Stripe Payments");
    assert_eq!(json_res["slug"], source_slug);
    assert_eq!(json_res["tenant_id"], tenant_id.to_string());
    assert_eq!(json_res["provider"], "stripe");
    assert_eq!(json_res["verification_type"], "hmac_sha256");
    assert_eq!(json_res["is_active"], true);
    assert_eq!(json_res["has_secret"], true);
    assert_eq!(json_res["secret"], "whsec_test_secret_12345");
    assert!(json_res["id"].is_string());
    assert!(json_res["created_at"].is_string());
    assert!(json_res["updated_at"].is_string());

    // Security check: encrypted/internal fields must never be exposed
    assert!(json_res.get("encrypted_secret").is_none());
    assert!(json_res.get("signing_secret_encrypted").is_none());

    // Also verify /api/v1/sources alias works and generated secret is returned when omitted
    let source_slug2 = format!("github-src-{}", Uuid::new_v4().simple());
    let payload2 = json!({
        "name": "GitHub Source",
        "slug": source_slug2
    });

    let req2 = Request::builder()
        .uri("/api/v1/sources")
        .method("POST")
        .header(header::CONTENT_TYPE, "application/json")
        .header("X-Api-Key", &raw_key)
        .body(Body::from(payload2.to_string()))
        .unwrap();

    let res2 = app.clone().oneshot(req2).await.unwrap();
    assert_eq!(res2.status(), StatusCode::CREATED);
    let json_res2: Value = serde_json::from_slice(&axum::body::to_bytes(res2.into_body(), usize::MAX).await.unwrap()).unwrap();
    assert!(json_res2["secret"].is_string());
    assert!(!json_res2["secret"].as_str().unwrap().is_empty());
    assert_eq!(json_res2["has_secret"], true);

    // Also verify POST /sources (without /v1) works
    let source_slug3 = format!("base-src-{}", Uuid::new_v4().simple());
    let payload3 = json!({
        "name": "Base Source",
        "slug": source_slug3
    });

    let req3 = Request::builder()
        .uri("/sources")
        .method("POST")
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::AUTHORIZATION, format!("Bearer {raw_key}"))
        .body(Body::from(payload3.to_string()))
        .unwrap();

    let res3 = app.oneshot(req3).await.unwrap();
    assert_eq!(res3.status(), StatusCode::CREATED);
}

// 2. Unauthenticated request tests
#[tokio::test]
async fn test_unauthenticated_requests() {
    let (app, _state, _pool) = setup_test_app().await;

    // Missing auth header
    let payload = json!({
        "name": "Test Source",
        "slug": "test-slug-no-auth"
    });

    let req = Request::builder()
        .uri("/v1/sources")
        .method("POST")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(payload.to_string()))
        .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);

    // Invalid API key
    let req_invalid = Request::builder()
        .uri("/v1/sources")
        .method("POST")
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::AUTHORIZATION, "Bearer invalid_nonexistent_key")
        .body(Body::from(payload.to_string()))
        .unwrap();

    let res_invalid = app.oneshot(req_invalid).await.unwrap();
    assert_eq!(res_invalid.status(), StatusCode::UNAUTHORIZED);
}

// 3. Invalid body test
#[tokio::test]
async fn test_invalid_request_body() {
    let (app, state, _pool) = setup_test_app().await;
    let (_tenant_id, raw_key) = create_test_tenant_and_key(&state, "inv-body").await;

    let req = Request::builder()
        .uri("/v1/sources")
        .method("POST")
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::AUTHORIZATION, format!("Bearer {raw_key}"))
        .body(Body::from("{ malformed json body"))
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert!(res.status().is_client_error());
}

// 4. Empty name test
#[tokio::test]
async fn test_empty_name_validation() {
    let (app, state, _pool) = setup_test_app().await;
    let (_tenant_id, raw_key) = create_test_tenant_and_key(&state, "empty-name").await;

    // Completely empty string
    let payload1 = json!({
        "name": "",
        "slug": "valid-slug"
    });

    let req1 = Request::builder()
        .uri("/v1/sources")
        .method("POST")
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::AUTHORIZATION, format!("Bearer {raw_key}"))
        .body(Body::from(payload1.to_string()))
        .unwrap();

    let res1 = app.clone().oneshot(req1).await.unwrap();
    assert_eq!(res1.status(), StatusCode::UNPROCESSABLE_ENTITY);

    // Whitespace only name
    let payload2 = json!({
        "name": "    ",
        "slug": "valid-slug-2"
    });

    let req2 = Request::builder()
        .uri("/v1/sources")
        .method("POST")
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::AUTHORIZATION, format!("Bearer {raw_key}"))
        .body(Body::from(payload2.to_string()))
        .unwrap();

    let res2 = app.oneshot(req2).await.unwrap();
    assert_eq!(res2.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

// 5. Invalid slug test
#[tokio::test]
async fn test_invalid_slug_validation() {
    let (app, state, _pool) = setup_test_app().await;
    let (_tenant_id, raw_key) = create_test_tenant_and_key(&state, "inv-slug").await;

    let long_slug = "a".repeat(101);
    let invalid_slugs = vec![
        "",
        "   ",
        "UppercaseSlug",
        "slug with spaces",
        "slug@with#symbols!",
        "-starts-with-hyphen",
        "_starts-with-underscore",
        "ends-with-hyphen-",
        "ends-with-underscore_",
        long_slug.as_str(),
    ];

    for slug in invalid_slugs {
        let payload = json!({
            "name": "Valid Name",
            "slug": slug
        });

        let req = Request::builder()
            .uri("/v1/sources")
            .method("POST")
            .header(header::CONTENT_TYPE, "application/json")
            .header(header::AUTHORIZATION, format!("Bearer {raw_key}"))
            .body(Body::from(payload.to_string()))
            .unwrap();

        let res = app.clone().oneshot(req).await.unwrap();
        assert_eq!(
            res.status(),
            StatusCode::UNPROCESSABLE_ENTITY,
            "Expected 422 for invalid slug: '{slug}'"
        );
    }
}

// 6. Duplicate slug within same tenant test
#[tokio::test]
async fn test_duplicate_slug_within_same_tenant() {
    let (app, state, _pool) = setup_test_app().await;
    let (_tenant_id, raw_key) = create_test_tenant_and_key(&state, "dup-tenant").await;

    let slug = format!("shared-slug-{}", Uuid::new_v4().simple());
    let payload = json!({
        "name": "First Source",
        "slug": slug
    });

    // 1st creation -> 201
    let req1 = Request::builder()
        .uri("/v1/sources")
        .method("POST")
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::AUTHORIZATION, format!("Bearer {raw_key}"))
        .body(Body::from(payload.to_string()))
        .unwrap();

    let res1 = app.clone().oneshot(req1).await.unwrap();
    assert_eq!(res1.status(), StatusCode::CREATED);

    // 2nd creation with same slug -> 409 Conflict
    let req2 = Request::builder()
        .uri("/v1/sources")
        .method("POST")
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::AUTHORIZATION, format!("Bearer {raw_key}"))
        .body(Body::from(payload.to_string()))
        .unwrap();

    let res2 = app.oneshot(req2).await.unwrap();
    assert_eq!(res2.status(), StatusCode::CONFLICT);
}

// 7. Same slug belonging to different tenants test
#[tokio::test]
async fn test_same_slug_different_tenants() {
    let (app, state, _pool) = setup_test_app().await;
    let (tenant_a, key_a) = create_test_tenant_and_key(&state, "tenant-a").await;
    let (tenant_b, key_b) = create_test_tenant_and_key(&state, "tenant-b").await;

    let slug = format!("cross-tenant-slug-{}", Uuid::new_v4().simple());

    // Tenant A creates source with slug
    let req_a = Request::builder()
        .uri("/v1/sources")
        .method("POST")
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::AUTHORIZATION, format!("Bearer {key_a}"))
        .body(Body::from(json!({ "name": "Source A", "slug": slug }).to_string()))
        .unwrap();

    let res_a = app.clone().oneshot(req_a).await.unwrap();
    assert_eq!(res_a.status(), StatusCode::CREATED);
    let body_a: Value = serde_json::from_slice(&axum::body::to_bytes(res_a.into_body(), usize::MAX).await.unwrap()).unwrap();
    assert_eq!(body_a["tenant_id"], tenant_a.to_string());

    // Tenant B creates source with SAME slug
    let req_b = Request::builder()
        .uri("/v1/sources")
        .method("POST")
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::AUTHORIZATION, format!("Bearer {key_b}"))
        .body(Body::from(json!({ "name": "Source B", "slug": slug }).to_string()))
        .unwrap();

    let res_b = app.oneshot(req_b).await.unwrap();
    assert_eq!(res_b.status(), StatusCode::CREATED);
    let body_b: Value = serde_json::from_slice(&axum::body::to_bytes(res_b.into_body(), usize::MAX).await.unwrap()).unwrap();
    assert_eq!(body_b["tenant_id"], tenant_b.to_string());

    // IDs must be distinct
    assert_ne!(body_a["id"], body_b["id"]);
}

// 8. Database failure test
#[tokio::test]
async fn test_database_failure_handling() {
    let (_, state, _pool) = setup_test_app().await;
    let (tenant_id, _) = create_test_tenant_and_key(&state, "db-fail").await;

    // Create a SourceService with an invalid pool that will fail
    let invalid_pool = Arc::new(
        sqlx::postgres::PgPoolOptions::new()
            .acquire_timeout(std::time::Duration::from_millis(10))
            .connect_lazy("postgres://invalid:invalid@localhost:9999/nonexistent")
            .unwrap(),
    );
    let broken_source_service = domain::services::SourceService::new(invalid_pool, [0u8; 32]);

    let res = broken_source_service
        .create_source(
            tenant_id,
            CreateSourceInput {
                name: "Test".to_string(),
                slug: "test-db-failure".to_string(),
                description: None,
                provider: "generic".to_string(),
                verification_type: "none".to_string(),
                secret: None,
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

// 9. Tenant isolation test
#[tokio::test]
async fn test_tenant_isolation() {
    let (app, state, _pool) = setup_test_app().await;
    let (tenant_a, key_a) = create_test_tenant_and_key(&state, "iso-a").await;
    let (tenant_b, key_b) = create_test_tenant_and_key(&state, "iso-b").await;

    // Tenant A attempts to spoof tenant_id by injecting tenant B's ID in request body
    let spoofed_slug = format!("spoofed-source-{}", Uuid::new_v4().simple());
    let req = Request::builder()
        .uri("/v1/sources")
        .method("POST")
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::AUTHORIZATION, format!("Bearer {key_a}"))
        .body(Body::from(
            json!({
                "name": "Spoofed Attempt",
                "slug": spoofed_slug,
                "tenant_id": tenant_b.to_string() // Attacker tries to create in Tenant B
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

    // Verify Tenant B listing sources does NOT see Tenant A's source
    let list_req = Request::builder()
        .uri("/v1/sources")
        .method("GET")
        .header(header::AUTHORIZATION, format!("Bearer {key_b}"))
        .body(Body::empty())
        .unwrap();

    let list_res = app.oneshot(list_req).await.unwrap();
    assert_eq!(list_res.status(), StatusCode::OK);

    let list_body: Value = serde_json::from_slice(&axum::body::to_bytes(list_res.into_body(), usize::MAX).await.unwrap()).unwrap();
    let sources_list = list_body.as_array().unwrap();
    for src in sources_list {
        assert_eq!(src["tenant_id"], tenant_b.to_string());
        assert_ne!(src["slug"], spoofed_slug);
    }
}

// --- GET /v1/sources Endpoint Tests ---

// 1. Authenticated tenant receives its sources
#[tokio::test]
async fn test_get_sources_authenticated_tenant() {
    let (app, state, _pool) = setup_test_app().await;
    let (tenant_id, raw_key) = create_test_tenant_and_key(&state, "get-src").await;

    // Create 3 sources for this tenant
    for i in 1..=3 {
        let slug = format!("source-{i}-{}", Uuid::new_v4().simple());
        let _ = state
            .source_service
            .create_source(
                tenant_id,
                CreateSourceInput {
                    name: format!("Source {i}"),
                    slug,
                    description: Some(format!("Description for source {i}")),
                    provider: "custom".to_string(),
                    verification_type: "hmac_sha256".to_string(),
                    secret: Some(format!("secret-{i}")),
                },
            )
            .await
            .unwrap();
    }

    // Call GET /v1/sources
    let req = Request::builder()
        .uri("/v1/sources?limit=10&offset=0")
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
        assert!(item["slug"].is_string());
        assert_eq!(item["is_active"], true);

        // Verify has_secret is present and true
        assert_eq!(item["has_secret"], true);
        assert!(item.get("verification_type").is_some());

        // Verify sensitive fields are never exposed
        assert!(item.get("secret").is_none());
        assert!(item.get("encrypted_secret").is_none());
        assert!(item.get("signing_secret_encrypted").is_none());
        assert!(item.get("secret_nonce").is_none());
    }

    // Also verify /sources (without /v1) works
    let req_base = Request::builder()
        .uri("/sources")
        .method("GET")
        .header(header::AUTHORIZATION, format!("Bearer {raw_key}"))
        .body(Body::empty())
        .unwrap();

    let res_base = app.clone().oneshot(req_base).await.unwrap();
    assert_eq!(res_base.status(), StatusCode::OK);
    let list_base: Value = serde_json::from_slice(&axum::body::to_bytes(res_base.into_body(), usize::MAX).await.unwrap()).unwrap();
    assert_eq!(list_base.as_array().unwrap().len(), 3);

    // Also verify /api/v1/sources route works with X-Api-Key
    let req_api = Request::builder()
        .uri("/api/v1/sources")
        .method("GET")
        .header("X-Api-Key", &raw_key)
        .body(Body::empty())
        .unwrap();

    let res_api = app.oneshot(req_api).await.unwrap();
    assert_eq!(res_api.status(), StatusCode::OK);
    let list_api: Value = serde_json::from_slice(&axum::body::to_bytes(res_api.into_body(), usize::MAX).await.unwrap()).unwrap();
    assert_eq!(list_api.as_array().unwrap().len(), 3);
}

// 2. Empty tenant returns empty result
#[tokio::test]
async fn test_get_sources_empty_tenant_returns_empty_list() {
    let (app, state, _pool) = setup_test_app().await;
    let (_tenant_id, raw_key) = create_test_tenant_and_key(&state, "get-empty").await;

    let req = Request::builder()
        .uri("/v1/sources")
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

// 3. Sources belonging to another tenant never appear (cross-tenant isolation)
#[tokio::test]
async fn test_get_sources_cross_tenant_isolation() {
    let (app, state, _pool) = setup_test_app().await;
    let (tenant_a, key_a) = create_test_tenant_and_key(&state, "cross-a").await;
    let (tenant_b, key_b) = create_test_tenant_and_key(&state, "cross-b").await;

    // Create 2 sources for Tenant A
    let slug_a1 = format!("a1-{}", Uuid::new_v4().simple());
    let slug_a2 = format!("a2-{}", Uuid::new_v4().simple());
    let _ = state.source_service.create_source(tenant_a, CreateSourceInput {
        name: "Source A1".to_string(),
        slug: slug_a1.clone(),
        description: None,
        provider: "generic".to_string(),
        verification_type: "none".to_string(),
        secret: None,
    }).await.unwrap();
    let _ = state.source_service.create_source(tenant_a, CreateSourceInput {
        name: "Source A2".to_string(),
        slug: slug_a2.clone(),
        description: None,
        provider: "generic".to_string(),
        verification_type: "none".to_string(),
        secret: None,
    }).await.unwrap();

    // Create 1 source for Tenant B
    let slug_b1 = format!("b1-{}", Uuid::new_v4().simple());
    let _ = state.source_service.create_source(tenant_b, CreateSourceInput {
        name: "Source B1".to_string(),
        slug: slug_b1.clone(),
        description: None,
        provider: "generic".to_string(),
        verification_type: "none".to_string(),
        secret: None,
    }).await.unwrap();

    // Query as Tenant A -> must only see A1 and A2
    let req_a = Request::builder()
        .uri("/v1/sources")
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
        assert_ne!(item["slug"], slug_b1);
    }

    // Query as Tenant B -> must only see B1
    let req_b = Request::builder()
        .uri("/v1/sources")
        .method("GET")
        .header(header::AUTHORIZATION, format!("Bearer {key_b}"))
        .body(Body::empty())
        .unwrap();

    let res_b = app.clone().oneshot(req_b).await.unwrap();
    assert_eq!(res_b.status(), StatusCode::OK);
    let list_b: Value = serde_json::from_slice(&axum::body::to_bytes(res_b.into_body(), usize::MAX).await.unwrap()).unwrap();
    let arr_b = list_b.as_array().unwrap();
    assert_eq!(arr_b.len(), 1);
    assert_eq!(arr_b[0]["slug"], slug_b1);
    assert_eq!(arr_b[0]["tenant_id"], tenant_b.to_string());

    // Security check: passing ?tenant_id=<tenant_a> as Tenant B must NOT leak Tenant A's sources
    let req_spoof = Request::builder()
        .uri(format!("/v1/sources?tenant_id={tenant_a}"))
        .method("GET")
        .header(header::AUTHORIZATION, format!("Bearer {key_b}"))
        .body(Body::empty())
        .unwrap();

    let res_spoof = app.oneshot(req_spoof).await.unwrap();
    assert_eq!(res_spoof.status(), StatusCode::OK);
    let list_spoof: Value = serde_json::from_slice(&axum::body::to_bytes(res_spoof.into_body(), usize::MAX).await.unwrap()).unwrap();
    let arr_spoof = list_spoof.as_array().unwrap();
    assert_eq!(arr_spoof.len(), 1);
    assert_eq!(arr_spoof[0]["tenant_id"], tenant_b.to_string());
}

// 4. Unauthenticated request fails
#[tokio::test]
async fn test_get_sources_unauthenticated_request_fails() {
    let (app, _state, _pool) = setup_test_app().await;

    // Missing auth header
    let req = Request::builder()
        .uri("/v1/sources")
        .method("GET")
        .body(Body::empty())
        .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);

    // Invalid auth token
    let req_invalid = Request::builder()
        .uri("/v1/sources")
        .method("GET")
        .header(header::AUTHORIZATION, "Bearer invalid_nonexistent_key_xyz")
        .body(Body::empty())
        .unwrap();

    let res_invalid = app.oneshot(req_invalid).await.unwrap();
    assert_eq!(res_invalid.status(), StatusCode::UNAUTHORIZED);
}

// 5. Database failure is handled correctly
#[tokio::test]
async fn test_get_sources_database_failure() {
    let (_app, state, _pool) = setup_test_app().await;
    let (tenant_id, _) = create_test_tenant_and_key(&state, "db-fail-get").await;

    let invalid_pool = Arc::new(
        sqlx::postgres::PgPoolOptions::new()
            .acquire_timeout(std::time::Duration::from_millis(10))
            .connect_lazy("postgres://invalid:invalid@localhost:9999/nonexistent")
            .unwrap(),
    );
    let broken_source_service = domain::services::SourceService::new(invalid_pool, [0u8; 32]);

    let res = broken_source_service.list_sources(tenant_id, 20, 0).await;
    assert!(res.is_err());
    match res.unwrap_err() {
        relay_core::error::CoreError::Internal(msg) => {
            assert!(msg.contains("Database error") || msg.contains("connection"));
        }
        err => panic!("Expected CoreError::Internal, got {:?}", err),
    }
}

// 6. GET /sources/{source_id} tests
#[tokio::test]
async fn test_get_source_by_id_existing() {
    let (app, state, _pool) = setup_test_app().await;
    let (tenant_id, raw_key) = create_test_tenant_and_key(&state, "get-src-id").await;

    let source = state
        .source_service
        .create_source(
            tenant_id,
            CreateSourceInput {
                name: "Specific Source".to_string(),
                slug: format!("spec-{}", Uuid::new_v4().simple()),
                description: Some("Specific source description".to_string()),
                provider: "shopify".to_string(),
                verification_type: "hmac_sha256".to_string(),
                secret: Some("my_secret_123".to_string()),
            },
        )
        .await
        .unwrap();

    // Query GET /sources/{id}
    let req = Request::builder()
        .uri(format!("/sources/{}", source.id))
        .method("GET")
        .header(header::AUTHORIZATION, format!("Bearer {raw_key}"))
        .body(Body::empty())
        .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let body_bytes = axum::body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let json_res: Value = serde_json::from_slice(&body_bytes).unwrap();

    assert_eq!(json_res["id"], source.id.to_string());
    assert_eq!(json_res["name"], "Specific Source");
    assert_eq!(json_res["slug"], source.slug);
    assert_eq!(json_res["provider"], "shopify");
    assert_eq!(json_res["verification_type"], "hmac_sha256");
    assert_eq!(json_res["is_active"], true);
    assert_eq!(json_res["has_secret"], true);

    // Verify secrets are NEVER exposed on GET
    assert!(json_res.get("secret").is_none());
    assert!(json_res.get("encrypted_secret").is_none());
    assert!(json_res.get("signing_secret_encrypted").is_none());
    assert!(json_res.get("secret_nonce").is_none());

    // Also verify /v1/sources/{id}
    let req_v1 = Request::builder()
        .uri(format!("/v1/sources/{}", source.id))
        .method("GET")
        .header(header::AUTHORIZATION, format!("Bearer {raw_key}"))
        .body(Body::empty())
        .unwrap();

    let res_v1 = app.oneshot(req_v1).await.unwrap();
    assert_eq!(res_v1.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_get_source_by_id_nonexistent() {
    let (app, state, _pool) = setup_test_app().await;
    let (_tenant_id, raw_key) = create_test_tenant_and_key(&state, "get-src-none").await;

    let req = Request::builder()
        .uri(format!("/sources/{}", Uuid::new_v4()))
        .method("GET")
        .header(header::AUTHORIZATION, format!("Bearer {raw_key}"))
        .body(Body::empty())
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_get_source_by_id_malformed_uuid() {
    let (app, state, _pool) = setup_test_app().await;
    let (_tenant_id, raw_key) = create_test_tenant_and_key(&state, "get-src-bad-uuid").await;

    let req = Request::builder()
        .uri("/sources/not-a-valid-uuid")
        .method("GET")
        .header(header::AUTHORIZATION, format!("Bearer {raw_key}"))
        .body(Body::empty())
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_get_source_by_id_cross_tenant_isolation() {
    let (app, state, _pool) = setup_test_app().await;
    let (tenant_a, _key_a) = create_test_tenant_and_key(&state, "get-src-iso-a").await;
    let (_tenant_b, key_b) = create_test_tenant_and_key(&state, "get-src-iso-b").await;

    let source_a = state
        .source_service
        .create_source(
            tenant_a,
            CreateSourceInput {
                name: "Tenant A Source".to_string(),
                slug: format!("ta-{}", Uuid::new_v4().simple()),
                description: None,
                provider: "generic".to_string(),
                verification_type: "none".to_string(),
                secret: None,
            },
        )
        .await
        .unwrap();

    // Tenant B queries Tenant A's source -> 404 Not Found
    let req = Request::builder()
        .uri(format!("/sources/{}", source_a.id))
        .method("GET")
        .header(header::AUTHORIZATION, format!("Bearer {key_b}"))
        .body(Body::empty())
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_get_source_by_id_unauthenticated() {
    let (app, _state, _pool) = setup_test_app().await;

    let req = Request::builder()
        .uri(format!("/sources/{}", Uuid::new_v4()))
        .method("GET")
        .body(Body::empty())
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_get_source_by_id_database_failure() {
    let (_app, state, _pool) = setup_test_app().await;
    let (tenant_id, _) = create_test_tenant_and_key(&state, "get-src-db-err").await;

    let invalid_pool = Arc::new(
        sqlx::postgres::PgPoolOptions::new()
            .acquire_timeout(std::time::Duration::from_millis(10))
            .connect_lazy("postgres://invalid:invalid@localhost:9999/nonexistent")
            .unwrap(),
    );
    let broken_source_service = domain::services::SourceService::new(invalid_pool, [0u8; 32]);

    let res = broken_source_service.get_source(tenant_id, Uuid::new_v4()).await;
    assert!(res.is_err());
    match res.unwrap_err() {
        relay_core::error::CoreError::Internal(msg) => {
            assert!(msg.contains("Database error") || msg.contains("connection"));
        }
        err => panic!("Expected CoreError::Internal, got {:?}", err),
    }
}

// 7. PATCH /sources/{source_id} tests
#[tokio::test]
async fn test_patch_source_name_update() {
    let (app, state, _pool) = setup_test_app().await;
    let (tenant_id, raw_key) = create_test_tenant_and_key(&state, "patch-name").await;

    let source = state
        .source_service
        .create_source(
            tenant_id,
            CreateSourceInput {
                name: "Original Name".to_string(),
                slug: format!("orig-{}", Uuid::new_v4().simple()),
                description: None,
                provider: "generic".to_string(),
                verification_type: "none".to_string(),
                secret: None,
            },
        )
        .await
        .unwrap();

    let patch_payload = json!({
        "name": "Updated Name"
    });

    let req = Request::builder()
        .uri(format!("/sources/{}", source.id))
        .method("PATCH")
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::AUTHORIZATION, format!("Bearer {raw_key}"))
        .body(Body::from(patch_payload.to_string()))
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let body_bytes = axum::body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let json_res: Value = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(json_res["name"], "Updated Name");
    assert_eq!(json_res["slug"], source.slug);
}

#[tokio::test]
async fn test_patch_source_active_state_update() {
    let (app, state, _pool) = setup_test_app().await;
    let (tenant_id, raw_key) = create_test_tenant_and_key(&state, "patch-active").await;

    let source = state
        .source_service
        .create_source(
            tenant_id,
            CreateSourceInput {
                name: "Active Source".to_string(),
                slug: format!("active-{}", Uuid::new_v4().simple()),
                description: None,
                provider: "generic".to_string(),
                verification_type: "none".to_string(),
                secret: None,
            },
        )
        .await
        .unwrap();

    // Deactivate
    let req_deactivate = Request::builder()
        .uri(format!("/sources/{}", source.id))
        .method("PATCH")
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::AUTHORIZATION, format!("Bearer {raw_key}"))
        .body(Body::from(json!({"is_active": false}).to_string()))
        .unwrap();

    let res_deactivate = app.clone().oneshot(req_deactivate).await.unwrap();
    assert_eq!(res_deactivate.status(), StatusCode::OK);
    let json_deact: Value = serde_json::from_slice(&axum::body::to_bytes(res_deactivate.into_body(), usize::MAX).await.unwrap()).unwrap();
    assert_eq!(json_deact["is_active"], false);

    // Reactivate
    let req_reactivate = Request::builder()
        .uri(format!("/sources/{}", source.id))
        .method("PATCH")
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::AUTHORIZATION, format!("Bearer {raw_key}"))
        .body(Body::from(json!({"is_active": true}).to_string()))
        .unwrap();

    let res_reactivate = app.oneshot(req_reactivate).await.unwrap();
    assert_eq!(res_reactivate.status(), StatusCode::OK);
    let json_react: Value = serde_json::from_slice(&axum::body::to_bytes(res_reactivate.into_body(), usize::MAX).await.unwrap()).unwrap();
    assert_eq!(json_react["is_active"], true);
}

#[tokio::test]
async fn test_patch_source_tolerance_update() {
    let (app, state, _pool) = setup_test_app().await;
    let (tenant_id, raw_key) = create_test_tenant_and_key(&state, "patch-tol").await;

    let source = state
        .source_service
        .create_source(
            tenant_id,
            CreateSourceInput {
                name: "Tolerance Source".to_string(),
                slug: format!("tol-{}", Uuid::new_v4().simple()),
                description: None,
                provider: "generic".to_string(),
                verification_type: "none".to_string(),
                secret: None,
            },
        )
        .await
        .unwrap();

    let req = Request::builder()
        .uri(format!("/sources/{}", source.id))
        .method("PATCH")
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::AUTHORIZATION, format!("Bearer {raw_key}"))
        .body(Body::from(json!({"timestamp_tolerance_secs": 300}).to_string()))
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let json_res: Value = serde_json::from_slice(&axum::body::to_bytes(res.into_body(), usize::MAX).await.unwrap()).unwrap();
    assert_eq!(json_res["timestamp_tolerance_secs"], 300);
}

#[tokio::test]
async fn test_patch_source_multiple_fields() {
    let (app, state, _pool) = setup_test_app().await;
    let (tenant_id, raw_key) = create_test_tenant_and_key(&state, "patch-multi").await;

    let source = state
        .source_service
        .create_source(
            tenant_id,
            CreateSourceInput {
                name: "Initial Name".to_string(),
                slug: format!("multi-{}", Uuid::new_v4().simple()),
                description: None,
                provider: "generic".to_string(),
                verification_type: "none".to_string(),
                secret: None,
            },
        )
        .await
        .unwrap();

    let req = Request::builder()
        .uri(format!("/v1/sources/{}", source.id))
        .method("PATCH")
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::AUTHORIZATION, format!("Bearer {raw_key}"))
        .body(Body::from(json!({
            "name": "Combined Updated Name",
            "is_active": false,
            "timestamp_tolerance_secs": 600
        }).to_string()))
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let json_res: Value = serde_json::from_slice(&axum::body::to_bytes(res.into_body(), usize::MAX).await.unwrap()).unwrap();
    assert_eq!(json_res["name"], "Combined Updated Name");
    assert_eq!(json_res["is_active"], false);
    assert_eq!(json_res["timestamp_tolerance_secs"], 600);
}

#[tokio::test]
async fn test_patch_source_empty_patch() {
    let (app, state, _pool) = setup_test_app().await;
    let (tenant_id, raw_key) = create_test_tenant_and_key(&state, "patch-empty").await;

    let source = state
        .source_service
        .create_source(
            tenant_id,
            CreateSourceInput {
                name: "Unchanged Source".to_string(),
                slug: format!("unchanged-{}", Uuid::new_v4().simple()),
                description: None,
                provider: "generic".to_string(),
                verification_type: "none".to_string(),
                secret: None,
            },
        )
        .await
        .unwrap();

    let req = Request::builder()
        .uri(format!("/sources/{}", source.id))
        .method("PATCH")
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::AUTHORIZATION, format!("Bearer {raw_key}"))
        .body(Body::from(json!({}).to_string()))
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let json_res: Value = serde_json::from_slice(&axum::body::to_bytes(res.into_body(), usize::MAX).await.unwrap()).unwrap();
    assert_eq!(json_res["name"], "Unchanged Source");
    assert_eq!(json_res["is_active"], true);
}

#[tokio::test]
async fn test_patch_source_secret_supplied_rejected() {
    let (app, state, _pool) = setup_test_app().await;
    let (tenant_id, raw_key) = create_test_tenant_and_key(&state, "patch-sec-rej").await;

    let source = state
        .source_service
        .create_source(
            tenant_id,
            CreateSourceInput {
                name: "Secret Reject Source".to_string(),
                slug: format!("sec-rej-{}", Uuid::new_v4().simple()),
                description: None,
                provider: "generic".to_string(),
                verification_type: "none".to_string(),
                secret: None,
            },
        )
        .await
        .unwrap();

    let req = Request::builder()
        .uri(format!("/sources/{}", source.id))
        .method("PATCH")
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::AUTHORIZATION, format!("Bearer {raw_key}"))
        .body(Body::from(json!({"secret": "new_secret_key"}).to_string()))
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn test_patch_source_nonexistent() {
    let (app, state, _pool) = setup_test_app().await;
    let (_tenant_id, raw_key) = create_test_tenant_and_key(&state, "patch-none").await;

    let req = Request::builder()
        .uri(format!("/sources/{}", Uuid::new_v4()))
        .method("PATCH")
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::AUTHORIZATION, format!("Bearer {raw_key}"))
        .body(Body::from(json!({"name": "New Name"}).to_string()))
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_patch_source_cross_tenant() {
    let (app, state, _pool) = setup_test_app().await;
    let (tenant_a, _key_a) = create_test_tenant_and_key(&state, "patch-iso-a").await;
    let (_tenant_b, key_b) = create_test_tenant_and_key(&state, "patch-iso-b").await;

    let source_a = state
        .source_service
        .create_source(
            tenant_a,
            CreateSourceInput {
                name: "Tenant A Source".to_string(),
                slug: format!("ta-patch-{}", Uuid::new_v4().simple()),
                description: None,
                provider: "generic".to_string(),
                verification_type: "none".to_string(),
                secret: None,
            },
        )
        .await
        .unwrap();

    let req = Request::builder()
        .uri(format!("/sources/{}", source_a.id))
        .method("PATCH")
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::AUTHORIZATION, format!("Bearer {key_b}"))
        .body(Body::from(json!({"name": "Attacker Name"}).to_string()))
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_patch_source_unauthorized() {
    let (app, _state, _pool) = setup_test_app().await;

    let req = Request::builder()
        .uri(format!("/sources/{}", Uuid::new_v4()))
        .method("PATCH")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(json!({"name": "Unauth Name"}).to_string()))
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
}

// 8. DELETE /sources/{source_id} tests
#[tokio::test]
async fn test_delete_source_with_no_subscriptions() {
    let (app, state, _pool) = setup_test_app().await;
    let (tenant_id, raw_key) = create_test_tenant_and_key(&state, "del-no-sub").await;

    let source = state
        .source_service
        .create_source(
            tenant_id,
            CreateSourceInput {
                name: "Delete Me Source".to_string(),
                slug: format!("del-no-sub-{}", Uuid::new_v4().simple()),
                description: None,
                provider: "generic".to_string(),
                verification_type: "none".to_string(),
                secret: None,
            },
        )
        .await
        .unwrap();

    let req = Request::builder()
        .uri(format!("/sources/{}", source.id))
        .method("DELETE")
        .header(header::AUTHORIZATION, format!("Bearer {raw_key}"))
        .body(Body::empty())
        .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::NO_CONTENT);

    // Verify source is now gone (404)
    let req_get = Request::builder()
        .uri(format!("/sources/{}", source.id))
        .method("GET")
        .header(header::AUTHORIZATION, format!("Bearer {raw_key}"))
        .body(Body::empty())
        .unwrap();

    let res_get = app.oneshot(req_get).await.unwrap();
    assert_eq!(res_get.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_delete_source_with_active_subscriptions_without_force_fails() {
    let (app, state, _pool) = setup_test_app().await;
    let (tenant_id, raw_key) = create_test_tenant_and_key(&state, "del-sub-err").await;

    let source = state
        .source_service
        .create_source(
            tenant_id,
            CreateSourceInput {
                name: "Active Sub Source".to_string(),
                slug: format!("active-sub-{}", Uuid::new_v4().simple()),
                description: None,
                provider: "generic".to_string(),
                verification_type: "none".to_string(),
                secret: None,
            },
        )
        .await
        .unwrap();

    let destination = state
        .destination_service
        .create_destination(
            tenant_id,
            domain::dto::CreateDestinationInput {
                name: "Dest 1".to_string(),
                url: "https://api.example.com/webhooks".to_string(),
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

    state
        .subscription_service
        .create_subscription(
            tenant_id,
            domain::dto::CreateSubscriptionInput {
                source_id: source.id,
                destination_id: destination.id,
                event_types: vec!["payment.created".to_string()],
                filter_rules: None,
                transformation_template: None,
            },
        )
        .await
        .unwrap();

    // 1. DELETE /sources/{id} without force -> 409 Conflict
    let req = Request::builder()
        .uri(format!("/sources/{}", source.id))
        .method("DELETE")
        .header(header::AUTHORIZATION, format!("Bearer {raw_key}"))
        .body(Body::empty())
        .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::CONFLICT);
    let body_bytes = axum::body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let json_res: Value = serde_json::from_slice(&body_bytes).unwrap();
    assert!(json_res["error"]["message"].as_str().unwrap().contains("active subscription"));

    // 2. DELETE /sources/{id}?force=false -> 409 Conflict
    let req_force_false = Request::builder()
        .uri(format!("/sources/{}?force=false", source.id))
        .method("DELETE")
        .header(header::AUTHORIZATION, format!("Bearer {raw_key}"))
        .body(Body::empty())
        .unwrap();

    let res_force_false = app.clone().oneshot(req_force_false).await.unwrap();
    assert_eq!(res_force_false.status(), StatusCode::CONFLICT);

    // 3. DELETE /sources/{id}?force=true -> 204 No Content
    let req_force_true = Request::builder()
        .uri(format!("/sources/{}?force=true", source.id))
        .method("DELETE")
        .header(header::AUTHORIZATION, format!("Bearer {raw_key}"))
        .body(Body::empty())
        .unwrap();

    let res_force_true = app.clone().oneshot(req_force_true).await.unwrap();
    assert_eq!(res_force_true.status(), StatusCode::NO_CONTENT);

    // Verify source is now soft-deleted (404)
    let req_get = Request::builder()
        .uri(format!("/sources/{}", source.id))
        .method("GET")
        .header(header::AUTHORIZATION, format!("Bearer {raw_key}"))
        .body(Body::empty())
        .unwrap();

    let res_get = app.oneshot(req_get).await.unwrap();
    assert_eq!(res_get.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_delete_source_nonexistent() {
    let (app, state, _pool) = setup_test_app().await;
    let (_tenant_id, raw_key) = create_test_tenant_and_key(&state, "del-none").await;

    let req = Request::builder()
        .uri(format!("/sources/{}", Uuid::new_v4()))
        .method("DELETE")
        .header(header::AUTHORIZATION, format!("Bearer {raw_key}"))
        .body(Body::empty())
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_delete_source_cross_tenant() {
    let (app, state, _pool) = setup_test_app().await;
    let (tenant_a, _key_a) = create_test_tenant_and_key(&state, "del-iso-a").await;
    let (_tenant_b, key_b) = create_test_tenant_and_key(&state, "del-iso-b").await;

    let source_a = state
        .source_service
        .create_source(
            tenant_a,
            CreateSourceInput {
                name: "Tenant A Source".to_string(),
                slug: format!("ta-del-{}", Uuid::new_v4().simple()),
                description: None,
                provider: "generic".to_string(),
                verification_type: "none".to_string(),
                secret: None,
            },
        )
        .await
        .unwrap();

    let req = Request::builder()
        .uri(format!("/sources/{}", source_a.id))
        .method("DELETE")
        .header(header::AUTHORIZATION, format!("Bearer {key_b}"))
        .body(Body::empty())
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_delete_source_unauthorized() {
    let (app, _state, _pool) = setup_test_app().await;

    let req = Request::builder()
        .uri(format!("/sources/{}", Uuid::new_v4()))
        .method("DELETE")
        .body(Body::empty())
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_delete_source_database_failure() {
    let (_app, state, _pool) = setup_test_app().await;
    let (tenant_id, _) = create_test_tenant_and_key(&state, "del-db-err").await;

    let invalid_pool = Arc::new(
        sqlx::postgres::PgPoolOptions::new()
            .acquire_timeout(std::time::Duration::from_millis(10))
            .connect_lazy("postgres://invalid:invalid@localhost:9999/nonexistent")
            .unwrap(),
    );
    let broken_source_service = domain::services::SourceService::new(invalid_pool, [0u8; 32]);

    let res = broken_source_service.delete_source(tenant_id, Uuid::new_v4(), false).await;
    assert!(res.is_err());
    match res.unwrap_err() {
        relay_core::error::CoreError::Internal(msg) => {
            assert!(msg.contains("Database error") || msg.contains("connection"));
        }
        err => panic!("Expected CoreError::Internal, got {:?}", err),
    }
}

// 9. POST /sources/{source_id}/rotate-secret tests
#[tokio::test]
async fn test_rotate_source_secret_success_and_audit_log() {
    let (app, state, pool) = setup_test_app().await;
    let (tenant_id, raw_key) = create_test_tenant_and_key(&state, "rot-sec").await;

    let source = state
        .source_service
        .create_source(
            tenant_id,
            CreateSourceInput {
                name: "Rotate Source".to_string(),
                slug: format!("rot-{}", Uuid::new_v4().simple()),
                description: None,
                provider: "stripe".to_string(),
                verification_type: "hmac_sha256".to_string(),
                secret: Some("initial_secret_12345".to_string()),
            },
        )
        .await
        .unwrap();

    let req = Request::builder()
        .uri(format!("/sources/{}/rotate-secret", source.id))
        .method("POST")
        .header(header::AUTHORIZATION, format!("Bearer {raw_key}"))
        .body(Body::empty())
        .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let body_bytes = axum::body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let json_res: Value = serde_json::from_slice(&body_bytes).unwrap();

    assert_eq!(json_res["source_id"], source.id.to_string());
    assert!(json_res["secret"].is_string());
    let new_secret = json_res["secret"].as_str().unwrap();
    assert_ne!(new_secret, "initial_secret_12345");
    assert!(!new_secret.is_empty());
    assert!(json_res["warning"].as_str().unwrap().contains("immediately"));

    // Verify audit log entry was created
    let audit_count = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT COUNT(*)
        FROM audit_logs
        WHERE tenant_id = $1 AND action = 'source.secret_rotated' AND resource_id = $2
        "#,
    )
    .bind(tenant_id)
    .bind(source.id)
    .fetch_one(&pool)
    .await
    .unwrap();

    assert_eq!(audit_count, 1);

    // Verify plaintext secret is NOT returned on subsequent GET /sources/{id}
    let req_get = Request::builder()
        .uri(format!("/sources/{}", source.id))
        .method("GET")
        .header(header::AUTHORIZATION, format!("Bearer {raw_key}"))
        .body(Body::empty())
        .unwrap();

    let res_get = app.oneshot(req_get).await.unwrap();
    assert_eq!(res_get.status(), StatusCode::OK);
    let json_get: Value = serde_json::from_slice(&axum::body::to_bytes(res_get.into_body(), usize::MAX).await.unwrap()).unwrap();
    assert!(json_get.get("secret").is_none());
    assert!(json_get.get("encrypted_secret").is_none());
    assert!(json_get.get("signing_secret_encrypted").is_none());
    assert_eq!(json_get["has_secret"], true);
}

#[tokio::test]
async fn test_rotate_source_secret_nonexistent() {
    let (app, state, _pool) = setup_test_app().await;
    let (_tenant_id, raw_key) = create_test_tenant_and_key(&state, "rot-none").await;

    let req = Request::builder()
        .uri(format!("/sources/{}/rotate-secret", Uuid::new_v4()))
        .method("POST")
        .header(header::AUTHORIZATION, format!("Bearer {raw_key}"))
        .body(Body::empty())
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_rotate_source_secret_cross_tenant() {
    let (app, state, _pool) = setup_test_app().await;
    let (tenant_a, _key_a) = create_test_tenant_and_key(&state, "rot-iso-a").await;
    let (_tenant_b, key_b) = create_test_tenant_and_key(&state, "rot-iso-b").await;

    let source_a = state
        .source_service
        .create_source(
            tenant_a,
            CreateSourceInput {
                name: "Tenant A Source".to_string(),
                slug: format!("ta-rot-{}", Uuid::new_v4().simple()),
                description: None,
                provider: "generic".to_string(),
                verification_type: "none".to_string(),
                secret: None,
            },
        )
        .await
        .unwrap();

    let req = Request::builder()
        .uri(format!("/sources/{}/rotate-secret", source_a.id))
        .method("POST")
        .header(header::AUTHORIZATION, format!("Bearer {key_b}"))
        .body(Body::empty())
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_rotate_source_secret_unauthorized() {
    let (app, _state, _pool) = setup_test_app().await;

    let req = Request::builder()
        .uri(format!("/sources/{}/rotate-secret", Uuid::new_v4()))
        .method("POST")
        .body(Body::empty())
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_rotate_source_secret_database_failure() {
    let (_app, state, _pool) = setup_test_app().await;
    let (tenant_id, _) = create_test_tenant_and_key(&state, "rot-db-err").await;

    let invalid_pool = Arc::new(
        sqlx::postgres::PgPoolOptions::new()
            .acquire_timeout(std::time::Duration::from_millis(10))
            .connect_lazy("postgres://invalid:invalid@localhost:9999/nonexistent")
            .unwrap(),
    );
    let broken_source_service = domain::services::SourceService::new(invalid_pool, [0u8; 32]);

    let res = broken_source_service.rotate_source_secret(tenant_id, Uuid::new_v4()).await;
    assert!(res.is_err());
    match res.unwrap_err() {
        relay_core::error::CoreError::Internal(msg) => {
            assert!(msg.contains("Database error") || msg.contains("connection"));
        }
        err => panic!("Expected CoreError::Internal, got {:?}", err),
    }
}

// 10. GET /sources/{source_id}/verification-log tests
#[tokio::test]
async fn test_verification_log_valid_source_and_no_payload_exposed() {
    let (app, state, pool) = setup_test_app().await;
    let (tenant_id, raw_key) = create_test_tenant_and_key(&state, "vlog-valid").await;

    let source = state
        .source_service
        .create_source(
            tenant_id,
            CreateSourceInput {
                name: "VLog Source".to_string(),
                slug: format!("vlog-{}", Uuid::new_v4().simple()),
                description: None,
                provider: "stripe".to_string(),
                verification_type: "hmac_sha256".to_string(),
                secret: Some("whsec_test".to_string()),
            },
        )
        .await
        .unwrap();

    // Ingest 2 events directly via repo
    let event_repo = data::repositories::EventRepository::new(&pool);
    for i in 1..=2 {
        event_repo
            .create(
                tenant_id,
                source.id,
                "charge.captured",
                Some(&format!("vlog-idem-{i}-{}", Uuid::new_v4())),
                json!({"stripe-signature": "sig_valid_123", "signature_valid": true}),
                json!({"sensitive_card": "4111222233334444", "amount": 1000 * i}),
            )
            .await
            .unwrap();
    }

    let req = Request::builder()
        .uri(format!("/sources/{}/verification-log?limit=50", source.id))
        .method("GET")
        .header(header::AUTHORIZATION, format!("Bearer {raw_key}"))
        .body(Body::empty())
        .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let body_bytes = axum::body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let json_res: Value = serde_json::from_slice(&body_bytes).unwrap();
    let arr = json_res.as_array().unwrap();

    assert_eq!(arr.len(), 2);
    for item in arr {
        assert!(item["received_at"].is_string());
        assert_eq!(item["signature_valid"], true);
        // Security check: raw payload and headers must NOT be returned
        assert!(item.get("payload").is_none());
        assert!(item.get("headers").is_none());
        assert!(item.get("sensitive_card").is_none());
    }

    // Also verify /v1/sources/{id}/verification-log
    let req_v1 = Request::builder()
        .uri(format!("/v1/sources/{}/verification-log", source.id))
        .method("GET")
        .header(header::AUTHORIZATION, format!("Bearer {raw_key}"))
        .body(Body::empty())
        .unwrap();

    let res_v1 = app.oneshot(req_v1).await.unwrap();
    assert_eq!(res_v1.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_verification_log_empty_log() {
    let (app, state, _pool) = setup_test_app().await;
    let (tenant_id, raw_key) = create_test_tenant_and_key(&state, "vlog-empty").await;

    let source = state
        .source_service
        .create_source(
            tenant_id,
            CreateSourceInput {
                name: "Empty VLog Source".to_string(),
                slug: format!("vlog-empty-{}", Uuid::new_v4().simple()),
                description: None,
                provider: "generic".to_string(),
                verification_type: "none".to_string(),
                secret: None,
            },
        )
        .await
        .unwrap();

    let req = Request::builder()
        .uri(format!("/sources/{}/verification-log", source.id))
        .method("GET")
        .header(header::AUTHORIZATION, format!("Bearer {raw_key}"))
        .body(Body::empty())
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let body_bytes = axum::body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let json_res: Value = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(json_res.as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn test_verification_log_limit_handling() {
    let (app, state, pool) = setup_test_app().await;
    let (tenant_id, raw_key) = create_test_tenant_and_key(&state, "vlog-lim").await;

    let source = state
        .source_service
        .create_source(
            tenant_id,
            CreateSourceInput {
                name: "Limit Source".to_string(),
                slug: format!("vlog-lim-{}", Uuid::new_v4().simple()),
                description: None,
                provider: "generic".to_string(),
                verification_type: "none".to_string(),
                secret: None,
            },
        )
        .await
        .unwrap();

    let event_repo = data::repositories::EventRepository::new(&pool);
    for i in 1..=5 {
        event_repo
            .create(
                tenant_id,
                source.id,
                "test.event",
                Some(&format!("vlog-lim-idem-{i}-{}", Uuid::new_v4())),
                json!({}),
                json!({"count": i}),
            )
            .await
            .unwrap();
    }

    let req = Request::builder()
        .uri(format!("/sources/{}/verification-log?limit=3", source.id))
        .method("GET")
        .header(header::AUTHORIZATION, format!("Bearer {raw_key}"))
        .body(Body::empty())
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body_bytes = axum::body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let json_res: Value = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(json_res.as_array().unwrap().len(), 3);
}

#[tokio::test]
async fn test_verification_log_maximum_limit_clamping() {
    let (app, state, _pool) = setup_test_app().await;
    let (tenant_id, raw_key) = create_test_tenant_and_key(&state, "vlog-max-lim").await;

    let source = state
        .source_service
        .create_source(
            tenant_id,
            CreateSourceInput {
                name: "Max Limit Source".to_string(),
                slug: format!("vlog-max-{}", Uuid::new_v4().simple()),
                description: None,
                provider: "generic".to_string(),
                verification_type: "none".to_string(),
                secret: None,
            },
        )
        .await
        .unwrap();

    // Querying with limit=500 -> should not error and should clamp gracefully
    let req = Request::builder()
        .uri(format!("/sources/{}/verification-log?limit=500", source.id))
        .method("GET")
        .header(header::AUTHORIZATION, format!("Bearer {raw_key}"))
        .body(Body::empty())
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_verification_log_cross_tenant() {
    let (app, state, _pool) = setup_test_app().await;
    let (tenant_a, _key_a) = create_test_tenant_and_key(&state, "vlog-iso-a").await;
    let (_tenant_b, key_b) = create_test_tenant_and_key(&state, "vlog-iso-b").await;

    let source_a = state
        .source_service
        .create_source(
            tenant_a,
            CreateSourceInput {
                name: "Tenant A Source".to_string(),
                slug: format!("ta-vlog-{}", Uuid::new_v4().simple()),
                description: None,
                provider: "generic".to_string(),
                verification_type: "none".to_string(),
                secret: None,
            },
        )
        .await
        .unwrap();

    let req = Request::builder()
        .uri(format!("/sources/{}/verification-log", source_a.id))
        .method("GET")
        .header(header::AUTHORIZATION, format!("Bearer {key_b}"))
        .body(Body::empty())
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_verification_log_nonexistent_source() {
    let (app, state, _pool) = setup_test_app().await;
    let (_tenant_id, raw_key) = create_test_tenant_and_key(&state, "vlog-none").await;

    let req = Request::builder()
        .uri(format!("/sources/{}/verification-log", Uuid::new_v4()))
        .method("GET")
        .header(header::AUTHORIZATION, format!("Bearer {raw_key}"))
        .body(Body::empty())
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_verification_log_unauthenticated() {
    let (app, _state, _pool) = setup_test_app().await;

    let req = Request::builder()
        .uri(format!("/sources/{}/verification-log", Uuid::new_v4()))
        .method("GET")
        .body(Body::empty())
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
}
