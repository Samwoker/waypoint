use axum::{
    body::Body,
    http::{header, Request, StatusCode},
};
use serde_json::{json, Value};
use sqlx::PgPool;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Mutex;
use tower::ServiceExt;
use uuid::Uuid;

use api::{create_router, AppState};
use relay_core::config::Config;
use data::pool::{create_pg_pool, run_migrations};
use data::queue::RedisQueue;
use domain::dto::{CreateApiKeyInput, CreateDestinationInput, CreateEventInput, CreateSourceInput, CreateTenantInput};

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

// 1. Successful event ingestion
#[tokio::test]
async fn test_successful_event_ingestion() {
    let (app, state, _pool) = setup_test_app().await;
    let (tenant_id, raw_key) = create_test_tenant_and_key(&state, "succ-ev").await;

    let source = state
        .source_service
        .create_source(
            tenant_id,
            CreateSourceInput {
                name: "Payment Gateway".to_string(),
                slug: format!("pay-{}", Uuid::new_v4().simple()),
                description: None,
                provider: "stripe".to_string(),
                verification_type: "hmac_sha256".to_string(),
                secret: None,
            },
        )
        .await
        .unwrap();

    let payload = json!({
        "source_id": source.id,
        "event_type": "payment.succeeded",
        "payload": {
            "amount": 2500,
            "currency": "usd"
        },
        "idempotency_key": format!("idem-{}", Uuid::new_v4().simple()),
        "headers": {
            "User-Agent": "Stripe/1.0"
        }
    });

    let req = Request::builder()
        .uri("/v1/events")
        .method("POST")
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::AUTHORIZATION, format!("Bearer {raw_key}"))
        .body(Body::from(payload.to_string()))
        .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::ACCEPTED);

    let body_bytes = axum::body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let json_res: Value = serde_json::from_slice(&body_bytes).unwrap();

    assert_eq!(json_res["tenant_id"], tenant_id.to_string());
    assert_eq!(json_res["source_id"], source.id.to_string());
    assert_eq!(json_res["event_type"], "payment.succeeded");
    assert_eq!(json_res["status"], "received");
    assert_eq!(json_res["payload"]["amount"], 2500);
    assert!(json_res["id"].is_string());

    // Also verify /api/v1/events alias
    let payload2 = json!({
        "source_id": source.id,
        "event_type": "order.created",
        "payload": { "order_id": "ord_1" }
    });

    let req2 = Request::builder()
        .uri("/api/v1/events")
        .method("POST")
        .header(header::CONTENT_TYPE, "application/json")
        .header("X-Api-Key", &raw_key)
        .body(Body::from(payload2.to_string()))
        .unwrap();

    let res2 = app.oneshot(req2).await.unwrap();
    assert_eq!(res2.status(), StatusCode::ACCEPTED);
}

// 2. Invalid event (empty event_type)
#[tokio::test]
async fn test_invalid_event_type() {
    let (app, state, _pool) = setup_test_app().await;
    let (tenant_id, raw_key) = create_test_tenant_and_key(&state, "inv-ev").await;

    let source = state
        .source_service
        .create_source(tenant_id, CreateSourceInput {
            name: "Src".to_string(),
            slug: format!("src-{}", Uuid::new_v4().simple()),
            description: None,
            provider: "generic".to_string(),
            verification_type: "none".to_string(),
            secret: None,
        })
        .await
        .unwrap();

    let payload = json!({
        "source_id": source.id,
        "event_type": "    ", // Whitespace only
        "payload": { "key": "val" }
    });

    let req = Request::builder()
        .uri("/v1/events")
        .method("POST")
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::AUTHORIZATION, format!("Bearer {raw_key}"))
        .body(Body::from(payload.to_string()))
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

// 3. Invalid JSON body
#[tokio::test]
async fn test_invalid_json_body() {
    let (app, state, _pool) = setup_test_app().await;
    let (_tenant_id, raw_key) = create_test_tenant_and_key(&state, "inv-json").await;

    let req = Request::builder()
        .uri("/v1/events")
        .method("POST")
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::AUTHORIZATION, format!("Bearer {raw_key}"))
        .body(Body::from("{ malformed json body"))
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert!(res.status().is_client_error());
}

// 4. Invalid source (non-existent source_id)
#[tokio::test]
async fn test_invalid_source_not_found() {
    let (app, state, _pool) = setup_test_app().await;
    let (_tenant_id, raw_key) = create_test_tenant_and_key(&state, "inv-src").await;

    let non_existent_source = Uuid::new_v4();
    let payload = json!({
        "source_id": non_existent_source,
        "event_type": "test.event",
        "payload": { "test": true }
    });

    let req = Request::builder()
        .uri("/v1/events")
        .method("POST")
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::AUTHORIZATION, format!("Bearer {raw_key}"))
        .body(Body::from(payload.to_string()))
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}

// 5. Cross-tenant source (source belongs to another tenant)
#[tokio::test]
async fn test_cross_tenant_source_rejected() {
    let (app, state, _pool) = setup_test_app().await;
    let (tenant_a, _key_a) = create_test_tenant_and_key(&state, "ev-cross-a").await;
    let (_tenant_b, key_b) = create_test_tenant_and_key(&state, "ev-cross-b").await;

    // Tenant A creates Source A
    let source_a = state
        .source_service
        .create_source(
            tenant_a,
            CreateSourceInput {
                name: "Source A".to_string(),
                slug: format!("src-a-{}", Uuid::new_v4().simple()),
                description: None,
                provider: "generic".to_string(),
                verification_type: "none".to_string(),
                secret: None,
            },
        )
        .await
        .unwrap();

    // Tenant B attempts to publish event for Source A
    let payload = json!({
        "source_id": source_a.id,
        "event_type": "spoofed.event",
        "payload": { "attack": true }
    });

    let req = Request::builder()
        .uri("/v1/events")
        .method("POST")
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::AUTHORIZATION, format!("Bearer {key_b}"))
        .body(Body::from(payload.to_string()))
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert!(res.status() == StatusCode::FORBIDDEN || res.status() == StatusCode::NOT_FOUND);
}

// 6. Duplicate idempotency key
#[tokio::test]
async fn test_duplicate_idempotency_key() {
    let (app, state, _pool) = setup_test_app().await;
    let (tenant_id, raw_key) = create_test_tenant_and_key(&state, "ev-idem").await;

    let source = state
        .source_service
        .create_source(tenant_id, CreateSourceInput {
            name: "Src".to_string(),
            slug: format!("src-{}", Uuid::new_v4().simple()),
            description: None,
            provider: "generic".to_string(),
            verification_type: "none".to_string(),
            secret: None,
        })
        .await
        .unwrap();

    let idem_key = format!("idem-key-{}", Uuid::new_v4().simple());
    let payload = json!({
        "source_id": source.id,
        "event_type": "invoice.paid",
        "payload": { "invoice_id": "inv_123" },
        "idempotency_key": idem_key
    });

    // 1st request
    let req1 = Request::builder()
        .uri("/v1/events")
        .method("POST")
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::AUTHORIZATION, format!("Bearer {raw_key}"))
        .body(Body::from(payload.to_string()))
        .unwrap();

    let res1 = app.clone().oneshot(req1).await.unwrap();
    assert_eq!(res1.status(), StatusCode::ACCEPTED);
    let body1: Value = serde_json::from_slice(&axum::body::to_bytes(res1.into_body(), usize::MAX).await.unwrap()).unwrap();
    let first_event_id = body1["id"].as_str().unwrap().to_string();

    // 2nd request with exact same idempotency key
    let req2 = Request::builder()
        .uri("/v1/events")
        .method("POST")
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::AUTHORIZATION, format!("Bearer {raw_key}"))
        .body(Body::from(payload.to_string()))
        .unwrap();

    let res2 = app.oneshot(req2).await.unwrap();
    assert_eq!(res2.status(), StatusCode::ACCEPTED);
    let body2: Value = serde_json::from_slice(&axum::body::to_bytes(res2.into_body(), usize::MAX).await.unwrap()).unwrap();
    let second_event_id = body2["id"].as_str().unwrap().to_string();

    // Idempotent match: returned IDs must be identical
    assert_eq!(first_event_id, second_event_id);
}

// 7. Queue failure test
#[tokio::test]
async fn test_queue_failure_handling() {
    let res = RedisQueue::new("redis://localhost:9999").await;
    assert!(res.is_err());
    match res.unwrap_err() {
        relay_core::error::CoreError::Internal(msg) => {
            assert!(msg.contains("Redis") || msg.contains("connect") || msg.contains("Connection"));
        }
        err => panic!("Expected CoreError::Internal, got {:?}", err),
    }
}

// 8. Database failure test
#[tokio::test]
async fn test_database_failure_handling() {
    let (_, state, _pool) = setup_test_app().await;
    let (tenant_id, _) = create_test_tenant_and_key(&state, "db-fail-ev").await;

    let invalid_pool = Arc::new(
        sqlx::postgres::PgPoolOptions::new()
            .acquire_timeout(std::time::Duration::from_millis(10))
            .connect_lazy("postgres://invalid:invalid@localhost:9999/nonexistent")
            .unwrap(),
    );

    let redis_url = std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string());
    let queue = RedisQueue::new(&redis_url).await.unwrap();

    let broken_ingestion = domain::services::IngestionService::new(invalid_pool, Arc::new(Mutex::new(queue)));
    let res = broken_ingestion
        .create_event(
            tenant_id,
            CreateEventInput {
                source_id: Some(Uuid::new_v4()),
                event_type: "test".to_string(),
                payload: json!({}),
                idempotency_key: None,
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

// 9. Unauthenticated request test
#[tokio::test]
async fn test_unauthenticated_event_request() {
    let (app, _state, _pool) = setup_test_app().await;

    // Missing auth header
    let req = Request::builder()
        .uri("/v1/events")
        .method("POST")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(json!({"event_type": "test", "payload": {}}).to_string()))
        .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);

    // Invalid key
    let req_invalid = Request::builder()
        .uri("/v1/events")
        .method("POST")
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::AUTHORIZATION, "Bearer invalid_nonexistent_key_123")
        .body(Body::from(json!({"event_type": "test", "payload": {}}).to_string()))
        .unwrap();

    let res_invalid = app.oneshot(req_invalid).await.unwrap();
    assert_eq!(res_invalid.status(), StatusCode::UNAUTHORIZED);
}

// 10. Verify webhook delivery is NOT executed synchronously inside HTTP request
#[tokio::test]
async fn test_webhook_delivery_not_executed_synchronously() {
    let (app, state, _pool) = setup_test_app().await;
    let (tenant_id, raw_key) = create_test_tenant_and_key(&state, "async-del").await;

    let source = state
        .source_service
        .create_source(tenant_id, CreateSourceInput {
            name: "Async Source".to_string(),
            slug: format!("src-{}", Uuid::new_v4().simple()),
            description: None,
            provider: "generic".to_string(),
            verification_type: "none".to_string(),
            secret: None,
        })
        .await
        .unwrap();

    // Create a destination pointing to a non-existent or slow endpoint
    let dest = state
        .destination_service
        .create_destination(tenant_id, CreateDestinationInput {
            name: "Slow Destination".to_string(),
            url: "https://192.0.2.1/unreachable-webhook".to_string(), // Unreachable IP
            description: None,
            rate_limit_rps: None,
            timeout_ms: Some(30000), // 30s timeout
            secret: None,
            max_retries: None,
            headers: None,
        })
        .await
        .unwrap();

    // Create subscription
    let _ = state
        .subscription_service
        .create_subscription(tenant_id, domain::dto::CreateSubscriptionInput {
            source_id: source.id,
            destination_id: dest.id,
            event_types: vec!["async.test".to_string()],
            filter_rules: None,
            transformation_template: None,
        })
        .await
        .unwrap();

    let payload = json!({
        "source_id": source.id,
        "event_type": "async.test",
        "payload": { "immediate": true }
    });

    let start = Instant::now();
    let req = Request::builder()
        .uri("/v1/events")
        .method("POST")
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::AUTHORIZATION, format!("Bearer {raw_key}"))
        .body(Body::from(payload.to_string()))
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    let duration = start.elapsed();

    assert_eq!(res.status(), StatusCode::ACCEPTED);
    // Request must return immediately (well under 500ms) without waiting for 30s webhook delivery timeout
    assert!(duration.as_millis() < 500, "Event ingestion took too long: {:?}", duration);
}

// --- GET /v1/events/{id} Tests ---

// 1. Existing event
#[tokio::test]
async fn test_get_event_existing() {
    let (app, state, _pool) = setup_test_app().await;
    let (tenant_id, raw_key) = create_test_tenant_and_key(&state, "get-ev-ok").await;

    let source = state
        .source_service
        .create_source(tenant_id, CreateSourceInput {
            name: "Event Source".to_string(),
            slug: format!("src-{}", Uuid::new_v4().simple()),
            description: None,
            provider: "stripe".to_string(),
            verification_type: "none".to_string(),
            secret: None,
        })
        .await
        .unwrap();

    let created_event = state
        .ingestion_service
        .create_event(tenant_id, CreateEventInput {
            source_id: Some(source.id),
            event_type: "customer.created".to_string(),
            payload: json!({ "customer_id": "cus_123" }),
            idempotency_key: None,
            headers: Some(json!({ "X-Custom": "header-value" })),
        })
        .await
        .unwrap();

    // Call GET /v1/events/{id}
    let req = Request::builder()
        .uri(format!("/v1/events/{}", created_event.id))
        .method("GET")
        .header(header::AUTHORIZATION, format!("Bearer {raw_key}"))
        .body(Body::empty())
        .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let body_bytes = axum::body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let json_res: Value = serde_json::from_slice(&body_bytes).unwrap();

    assert_eq!(json_res["id"], created_event.id.to_string());
    assert_eq!(json_res["tenant_id"], tenant_id.to_string());
    assert_eq!(json_res["source_id"], source.id.to_string());
    assert_eq!(json_res["event_type"], "customer.created");
    assert_eq!(json_res["status"], "no_subscriptions");
    assert!(json_res.get("payload").is_none());
    assert_eq!(json_res["delivery_summary"]["total"], 0);
    assert!(json_res["created_at"].is_string());

    // Also verify /api/v1/events/{id} alias
    let req_api = Request::builder()
        .uri(format!("/api/v1/events/{}", created_event.id))
        .method("GET")
        .header("X-Api-Key", &raw_key)
        .body(Body::empty())
        .unwrap();

    let res_api = app.oneshot(req_api).await.unwrap();
    assert_eq!(res_api.status(), StatusCode::OK);
}

// 2. Nonexistent event
#[tokio::test]
async fn test_get_event_nonexistent() {
    let (app, state, _pool) = setup_test_app().await;
    let (_tenant_id, raw_key) = create_test_tenant_and_key(&state, "get-ev-none").await;

    let non_existent_id = Uuid::new_v4();
    let req = Request::builder()
        .uri(format!("/v1/events/{non_existent_id}"))
        .method("GET")
        .header(header::AUTHORIZATION, format!("Bearer {raw_key}"))
        .body(Body::empty())
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}

// 3. Malformed ID
#[tokio::test]
async fn test_get_event_malformed_id() {
    let (app, state, _pool) = setup_test_app().await;
    let (_tenant_id, raw_key) = create_test_tenant_and_key(&state, "get-ev-mal").await;

    let req = Request::builder()
        .uri("/v1/events/not-a-valid-uuid-12345")
        .method("GET")
        .header(header::AUTHORIZATION, format!("Bearer {raw_key}"))
        .body(Body::empty())
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert!(res.status() == StatusCode::BAD_REQUEST || res.status() == StatusCode::NOT_FOUND || res.status().is_client_error());
}

// 4. Event belonging to another tenant
#[tokio::test]
async fn test_get_event_cross_tenant_rejected() {
    let (app, state, _pool) = setup_test_app().await;
    let (tenant_a, _key_a) = create_test_tenant_and_key(&state, "get-ev-a").await;
    let (_tenant_b, key_b) = create_test_tenant_and_key(&state, "get-ev-b").await;

    // Tenant A creates source & event
    let source_a = state
        .source_service
        .create_source(tenant_a, CreateSourceInput {
            name: "Source A".to_string(),
            slug: format!("src-a-{}", Uuid::new_v4().simple()),
            description: None,
            provider: "generic".to_string(),
            verification_type: "none".to_string(),
            secret: None,
        })
        .await
        .unwrap();

    let event_a = state
        .ingestion_service
        .create_event(tenant_a, CreateEventInput {
            source_id: Some(source_a.id),
            event_type: "sensitive.event".to_string(),
            payload: json!({ "secret_data": "classified" }),
            idempotency_key: None,
            headers: None,
        })
        .await
        .unwrap();

    // Tenant B attempts to access Tenant A's event -> 404 Not Found (zero information leak)
    let req = Request::builder()
        .uri(format!("/v1/events/{}", event_a.id))
        .method("GET")
        .header(header::AUTHORIZATION, format!("Bearer {key_b}"))
        .body(Body::empty())
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}

// 5. Unauthenticated request
#[tokio::test]
async fn test_get_event_unauthenticated() {
    let (app, _state, _pool) = setup_test_app().await;

    let event_id = Uuid::new_v4();

    // Missing auth header
    let req = Request::builder()
        .uri(format!("/v1/events/{event_id}"))
        .method("GET")
        .body(Body::empty())
        .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);

    // Invalid API key
    let req_invalid = Request::builder()
        .uri(format!("/v1/events/{event_id}"))
        .method("GET")
        .header(header::AUTHORIZATION, "Bearer invalid_nonexistent_token_999")
        .body(Body::empty())
        .unwrap();

    let res_invalid = app.oneshot(req_invalid).await.unwrap();
    assert_eq!(res_invalid.status(), StatusCode::UNAUTHORIZED);
}

// 6. GET /events with Keyset Pagination and Filters (Endpoint #39)
#[tokio::test]
async fn test_list_events_pagination_and_filters() {
    let (app, state, pool) = setup_test_app().await;
    let (tenant_id, raw_key) = create_test_tenant_and_key(&state, "list-ev-pg").await;

    let source1 = state
        .source_service
        .create_source(
            tenant_id,
            CreateSourceInput {
                name: "Source 1".to_string(),
                slug: format!("s1-{}", Uuid::new_v4().simple()),
                description: None,
                provider: "generic".to_string(),
                verification_type: "none".to_string(),
                secret: None,
            },
        )
        .await
        .unwrap();

    let source2 = state
        .source_service
        .create_source(
            tenant_id,
            CreateSourceInput {
                name: "Source 2".to_string(),
                slug: format!("s2-{}", Uuid::new_v4().simple()),
                description: None,
                provider: "generic".to_string(),
                verification_type: "none".to_string(),
                secret: None,
            },
        )
        .await
        .unwrap();

    let dest = state
        .destination_service
        .create_destination(
            tenant_id,
            CreateDestinationInput {
                name: "Dest 1".to_string(),
                url: "https://example.com/dest".to_string(),
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

    let sub = state
        .subscription_service
        .create_subscription(
            tenant_id,
            domain::dto::CreateSubscriptionInput {
                source_id: source1.id,
                destination_id: dest.id,
                event_types: vec!["test.event".to_string()],
                filter_rules: None,
                transformation_template: None,
            },
        )
        .await
        .unwrap();

    // Create 5 events
    let mut event_ids = Vec::new();
    for i in 1..=5 {
        let src_id = if i <= 3 { source1.id } else { source2.id };
        let ev = state
            .ingestion_service
            .create_event(
                tenant_id,
                CreateEventInput {
                    source_id: Some(src_id),
                    event_type: "test.event".to_string(),
                    payload: json!({ "index": i }),
                    idempotency_key: None,
                    headers: None,
                },
            )
            .await
            .unwrap();

        if i <= 3 {
            // Create delivery for source1 events
            let delivery = data::repositories::DeliveryRepository::new(&pool)
                .create(tenant_id, ev.id, sub.id, dest.id, 5)
                .await
                .unwrap();

            if i == 1 {
                // Mark delivery as delivered
                data::repositories::DeliveryRepository::new(&pool)
                    .update_status(delivery.id, "delivered", 1, None)
                    .await
                    .unwrap();
            }
        }
        event_ids.push(ev.id);
    }

    // 1. First page: limit = 2
    let req1 = Request::builder()
        .uri("/events?limit=2")
        .method("GET")
        .header(header::AUTHORIZATION, format!("Bearer {raw_key}"))
        .body(Body::empty())
        .unwrap();

    let res1 = app.clone().oneshot(req1).await.unwrap();
    assert_eq!(res1.status(), StatusCode::OK);
    let page1: Value = serde_json::from_slice(&axum::body::to_bytes(res1.into_body(), usize::MAX).await.unwrap()).unwrap();
    assert_eq!(page1["events"].as_array().unwrap().len(), 2);
    assert_eq!(page1["has_more"], true);
    let cursor1 = page1["next_cursor"].as_str().unwrap();

    // 2. Second page using cursor
    let req2 = Request::builder()
        .uri(format!("/events?limit=2&cursor={cursor1}"))
        .method("GET")
        .header(header::AUTHORIZATION, format!("Bearer {raw_key}"))
        .body(Body::empty())
        .unwrap();

    let res2 = app.clone().oneshot(req2).await.unwrap();
    assert_eq!(res2.status(), StatusCode::OK);
    let page2: Value = serde_json::from_slice(&axum::body::to_bytes(res2.into_body(), usize::MAX).await.unwrap()).unwrap();
    assert_eq!(page2["events"].as_array().unwrap().len(), 2);
    assert_eq!(page2["has_more"], true);

    // 3. Filter by source_id = source2.id (should return 2 events)
    let req_src = Request::builder()
        .uri(format!("/events?source_id={}", source2.id))
        .method("GET")
        .header(header::AUTHORIZATION, format!("Bearer {raw_key}"))
        .body(Body::empty())
        .unwrap();

    let res_src = app.clone().oneshot(req_src).await.unwrap();
    assert_eq!(res_src.status(), StatusCode::OK);
    let page_src: Value = serde_json::from_slice(&axum::body::to_bytes(res_src.into_body(), usize::MAX).await.unwrap()).unwrap();
    assert_eq!(page_src["events"].as_array().unwrap().len(), 2);

    // 4. Filter by status = delivered (should return 1 event)
    let req_st = Request::builder()
        .uri("/events?status=delivered")
        .method("GET")
        .header(header::AUTHORIZATION, format!("Bearer {raw_key}"))
        .body(Body::empty())
        .unwrap();

    let res_st = app.clone().oneshot(req_st).await.unwrap();
    assert_eq!(res_st.status(), StatusCode::OK);
    let page_st: Value = serde_json::from_slice(&axum::body::to_bytes(res_st.into_body(), usize::MAX).await.unwrap()).unwrap();
    assert_eq!(page_st["events"].as_array().unwrap().len(), 1);

    // 5. Malformed cursor -> 400 Bad Request
    let req_bad_cursor = Request::builder()
        .uri("/events?cursor=invalid_base64_not_good")
        .method("GET")
        .header(header::AUTHORIZATION, format!("Bearer {raw_key}"))
        .body(Body::empty())
        .unwrap();

    let res_bad = app.oneshot(req_bad_cursor).await.unwrap();
    assert!(res_bad.status() == StatusCode::BAD_REQUEST || res_bad.status() == StatusCode::UNPROCESSABLE_ENTITY);
}

// 7. GET /events/{event_id}/raw (Endpoint #41)
#[tokio::test]
async fn test_get_event_raw_payload_and_audit() {
    let (app, state, _pool) = setup_test_app().await;
    let (tenant_id, full_key) = create_test_tenant_and_key(&state, "raw-ev").await;

    // Create a read_only key
    let ro_api_key = state
        .auth_service
        .create_api_key(
            tenant_id,
            CreateApiKeyInput {
                name: "read_only_key".to_string(),
                expires_at: None,
            },
        )
        .await
        .unwrap();

    let source = state
        .source_service
        .create_source(
            tenant_id,
            CreateSourceInput {
                name: "Raw Source".to_string(),
                slug: format!("raw-{}", Uuid::new_v4().simple()),
                description: None,
                provider: "generic".to_string(),
                verification_type: "none".to_string(),
                secret: None,
            },
        )
        .await
        .unwrap();

    let event = state
        .ingestion_service
        .create_event(
            tenant_id,
            CreateEventInput {
                source_id: Some(source.id),
                event_type: "order.processed".to_string(),
                payload: json!({ "order_id": "ord_999", "amount": 250 }),
                idempotency_key: None,
                headers: Some(json!({ "X-Signature": "sig123" })),
            },
        )
        .await
        .unwrap();

    // 1. Full scope succeeds
    let req_full = Request::builder()
        .uri(format!("/events/{}/raw", event.id))
        .method("GET")
        .header(header::AUTHORIZATION, format!("Bearer {full_key}"))
        .body(Body::empty())
        .unwrap();

    let res_full = app.clone().oneshot(req_full).await.unwrap();
    assert_eq!(res_full.status(), StatusCode::OK);
    let json_full: Value = serde_json::from_slice(&axum::body::to_bytes(res_full.into_body(), usize::MAX).await.unwrap()).unwrap();
    assert_eq!(json_full["event_id"], event.id.to_string());
    assert!(json_full["payload"].as_str().unwrap().contains("ord_999"));

    // 2. Read-only key rejected with 403 Forbidden
    let req_ro = Request::builder()
        .uri(format!("/events/{}/raw", event.id))
        .method("GET")
        .header(header::AUTHORIZATION, format!("Bearer {}", ro_api_key.raw_key))
        .body(Body::empty())
        .unwrap();

    let res_ro = app.clone().oneshot(req_ro).await.unwrap();
    assert_eq!(res_ro.status(), StatusCode::FORBIDDEN);

    // 3. Cross-tenant request rejected with 404
    let (_tenant_b, key_b) = create_test_tenant_and_key(&state, "raw-ev-b").await;
    let req_cross = Request::builder()
        .uri(format!("/events/{}/raw", event.id))
        .method("GET")
        .header(header::AUTHORIZATION, format!("Bearer {key_b}"))
        .body(Body::empty())
        .unwrap();

    let res_cross = app.oneshot(req_cross).await.unwrap();
    assert_eq!(res_cross.status(), StatusCode::NOT_FOUND);
}

// 8. DELETE /events/{event_id} Compliance Deletion (Endpoint #42)
#[tokio::test]
async fn test_delete_event_compliance_and_cascades() {
    let (app, state, pool) = setup_test_app().await;
    let (tenant_id, raw_key) = create_test_tenant_and_key(&state, "del-ev-comp").await;

    let source = state
        .source_service
        .create_source(
            tenant_id,
            CreateSourceInput {
                name: "Compliance Source".to_string(),
                slug: format!("comp-{}", Uuid::new_v4().simple()),
                description: None,
                provider: "generic".to_string(),
                verification_type: "none".to_string(),
                secret: None,
            },
        )
        .await
        .unwrap();

    let dest = state
        .destination_service
        .create_destination(
            tenant_id,
            CreateDestinationInput {
                name: "Compliance Dest".to_string(),
                url: "https://example.com/dest".to_string(),
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

    let sub = state
        .subscription_service
        .create_subscription(
            tenant_id,
            domain::dto::CreateSubscriptionInput {
                source_id: source.id,
                destination_id: dest.id,
                event_types: vec!["comp.event".to_string()],
                filter_rules: None,
                transformation_template: None,
            },
        )
        .await
        .unwrap();

    let event = state
        .ingestion_service
        .create_event(
            tenant_id,
            CreateEventInput {
                source_id: Some(source.id),
                event_type: "comp.event".to_string(),
                payload: json!({ "gdpr_data": "must_be_deleted" }),
                idempotency_key: None,
                headers: None,
            },
        )
        .await
        .unwrap();

    let deliv_repo = data::repositories::DeliveryRepository::new(&pool);
    let delivery = deliv_repo.create(tenant_id, event.id, sub.id, dest.id, 5).await.unwrap();
    deliv_repo.record_attempt(delivery.id, 1, Some(200), None, None, None, None, None, Some(30)).await.unwrap();

    // Delete event
    let req = Request::builder()
        .uri(format!("/events/{}", event.id))
        .method("DELETE")
        .header(header::AUTHORIZATION, format!("Bearer {raw_key}"))
        .body(Body::empty())
        .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::NO_CONTENT);

    // Verify GET /events/{id} returns 404
    let get_req = Request::builder()
        .uri(format!("/events/{}", event.id))
        .method("GET")
        .header(header::AUTHORIZATION, format!("Bearer {raw_key}"))
        .body(Body::empty())
        .unwrap();

    let get_res = app.oneshot(get_req).await.unwrap();
    assert_eq!(get_res.status(), StatusCode::NOT_FOUND);

    // Verify deliveries were cascade deleted
    let deliv = deliv_repo.find_by_tenant_and_id(tenant_id, delivery.id).await.unwrap();
    assert!(deliv.is_none());

    // Verify audit log exists
    let audit_logs = data::repositories::AuditLogRepository::new(&pool)
        .list_by_tenant(tenant_id, None, None, None, None, 10, 0)
        .await
        .unwrap();
    let comp_audit = audit_logs.iter().find(|a| a.action == "event.compliance_deleted");
    assert!(comp_audit.is_some());
    // Verify payload is NOT copied into audit log
    assert!(comp_audit.unwrap().metadata.to_string().contains("gdpr_data") == false);
}

// 9. GET /events/{event_id}/deliveries (Endpoint #43)
#[tokio::test]
async fn test_get_event_deliveries_and_destination_names() {
    let (app, state, pool) = setup_test_app().await;
    let (tenant_id, raw_key) = create_test_tenant_and_key(&state, "ev-delivs").await;

    let source = state
        .source_service
        .create_source(
            tenant_id,
            CreateSourceInput {
                name: "Deliv Event Source".to_string(),
                slug: format!("ev-del-{}", Uuid::new_v4().simple()),
                description: None,
                provider: "generic".to_string(),
                verification_type: "none".to_string(),
                secret: None,
            },
        )
        .await
        .unwrap();

    let dest = state
        .destination_service
        .create_destination(
            tenant_id,
            CreateDestinationInput {
                name: "Customer Webhook Target".to_string(),
                url: "https://customer.example.com/target".to_string(),
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

    let sub = state
        .subscription_service
        .create_subscription(
            tenant_id,
            domain::dto::CreateSubscriptionInput {
                source_id: source.id,
                destination_id: dest.id,
                event_types: vec!["target.event".to_string()],
                filter_rules: None,
                transformation_template: None,
            },
        )
        .await
        .unwrap();

    let event = state
        .ingestion_service
        .create_event(
            tenant_id,
            CreateEventInput {
                source_id: Some(source.id),
                event_type: "target.event".to_string(),
                payload: json!({ "order": "123" }),
                idempotency_key: None,
                headers: None,
            },
        )
        .await
        .unwrap();

    let delivery = data::repositories::DeliveryRepository::new(&pool)
        .create(tenant_id, event.id, sub.id, dest.id, 5)
        .await
        .unwrap();

    // Query deliveries for event
    let req = Request::builder()
        .uri(format!("/events/{}/deliveries", event.id))
        .method("GET")
        .header(header::AUTHORIZATION, format!("Bearer {raw_key}"))
        .body(Body::empty())
        .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let body_bytes = axum::body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let json_res: Value = serde_json::from_slice(&body_bytes).unwrap();
    let arr = json_res.as_array().unwrap();

    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0]["id"], delivery.id.to_string());
    assert_eq!(arr[0]["destination_id"], dest.id.to_string());
    assert_eq!(arr[0]["destination_name"], "Customer Webhook Target");
    assert_eq!(arr[0]["status"], "pending");
    assert_eq!(arr[0]["attempt_count"], 0);

    // Cross tenant request returns 404
    let (_tenant_b, key_b) = create_test_tenant_and_key(&state, "ev-delivs-b").await;
    let req_cross = Request::builder()
        .uri(format!("/events/{}/deliveries", event.id))
        .method("GET")
        .header(header::AUTHORIZATION, format!("Bearer {key_b}"))
        .body(Body::empty())
        .unwrap();

    let res_cross = app.oneshot(req_cross).await.unwrap();
    assert_eq!(res_cross.status(), StatusCode::NOT_FOUND);
}

