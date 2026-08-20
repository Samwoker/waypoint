use axum::{
    body::Body,
    http::{header, Request, StatusCode},
};
use serde_json::{json, Value};
use sqlx::PgPool;
use tower::ServiceExt;
use uuid::Uuid;

use api::{create_router, AppState};
use relay_core::config::Config;
use data::pool::{create_pg_pool, run_migrations};
use data::queue::RedisQueue;
use data::repositories::DeliveryRepository;
use domain::dto::{
    CreateApiKeyInput, CreateDestinationInput, CreateEventInput, CreateSourceInput,
    CreateSubscriptionInput, CreateTenantInput, ReplayBatchInput, ReplayDeliveryInput,
};

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

// 1. Successful lookup & attempt history with snippets (Endpoint #45)
#[tokio::test]
async fn test_successful_delivery_lookup_with_attempts() {
    let (app, state, pool) = setup_test_app().await;
    let (tenant_id, raw_key) = create_test_tenant_and_key(&state, "succ-del").await;

    // Create source
    let source = state
        .source_service
        .create_source(
            tenant_id,
            CreateSourceInput {
                name: "Delivery Source".to_string(),
                slug: format!("src-{}", Uuid::new_v4().simple()),
                description: None,
                provider: "generic".to_string(),
                verification_type: "none".to_string(),
                secret: None,
            },
        )
        .await
        .unwrap();

    // Create destination
    let dest = state
        .destination_service
        .create_destination(
            tenant_id,
            CreateDestinationInput {
                name: "Delivery Dest".to_string(),
                url: "https://example.com/webhooks".to_string(),
                description: None,
                rate_limit_rps: None,
                timeout_ms: None,
                secret: None,
                max_retries: Some(5),
                headers: None,
            },
        )
        .await
        .unwrap();

    // Create subscription
    let sub = state
        .subscription_service
        .create_subscription(
            tenant_id,
            CreateSubscriptionInput {
                source_id: source.id,
                destination_id: dest.id,
                event_types: vec!["payment.succeeded".to_string()],
                filter_rules: None,
                transformation_template: None,
            },
        )
        .await
        .unwrap();

    // Create event
    let event = state
        .ingestion_service
        .create_event(
            tenant_id,
            CreateEventInput {
                source_id: Some(source.id),
                event_type: "payment.succeeded".to_string(),
                payload: serde_json::json!({"amount": 100}),
                idempotency_key: None,
                headers: None,
            },
        )
        .await
        .unwrap();

    // Create delivery in DB
    let delivery_repo = DeliveryRepository::new(&pool);
    let delivery = delivery_repo
        .create(tenant_id, event.id, sub.id, dest.id, 5)
        .await
        .unwrap();

    // Record 2 attempts: Attempt 1 (failed), Attempt 2 (success)
    let long_body = "A".repeat(800);
    delivery_repo
        .record_attempt(
            delivery.id,
            1,
            Some(500),
            None,
            None,
            None,
            Some(&long_body),
            Some("Internal Server Error"),
            Some(120),
        )
        .await
        .unwrap();

    delivery_repo
        .record_attempt(
            delivery.id,
            2,
            Some(200),
            None,
            None,
            None,
            Some("{\"status\":\"ok\"}"),
            None,
            Some(45),
        )
        .await
        .unwrap();

    // Query GET /deliveries/{id}
    let req = Request::builder()
        .uri(format!("/deliveries/{}", delivery.id))
        .method("GET")
        .header(header::AUTHORIZATION, format!("Bearer {raw_key}"))
        .body(Body::empty())
        .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let body_bytes = axum::body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let json_res: Value = serde_json::from_slice(&body_bytes).unwrap();

    assert_eq!(json_res["id"], delivery.id.to_string());
    assert_eq!(json_res["tenant_id"], tenant_id.to_string());
    assert_eq!(json_res["event_id"], event.id.to_string());
    assert_eq!(json_res["destination_id"], dest.id.to_string());
    assert_eq!(json_res["status"], "pending");
    assert_eq!(json_res["attempt_count"], 0);
    assert_eq!(json_res["max_attempts"], 5);

    let attempts = json_res["attempts"].as_array().unwrap();
    assert_eq!(attempts.len(), 2);
    assert_eq!(attempts[0]["attempt_number"], 1);
    assert_eq!(attempts[0]["http_status"], 500);
    assert_eq!(attempts[0]["error_message"], "Internal Server Error");
    assert_eq!(attempts[0]["latency_ms"], 120);
    // Verify snippet is truncated to 500 chars + "..."
    let snippet = attempts[0]["response_body_snippet"].as_str().unwrap();
    assert!(snippet.len() <= 503);
    assert!(snippet.ends_with("..."));

    assert_eq!(attempts[1]["attempt_number"], 2);
    assert_eq!(attempts[1]["http_status"], 200);
    assert_eq!(attempts[1]["latency_ms"], 45);
    assert_eq!(attempts[1]["response_body_snippet"], "{\"status\":\"ok\"}");
}

// 2. Nonexistent delivery
#[tokio::test]
async fn test_get_delivery_nonexistent() {
    let (app, state, _pool) = setup_test_app().await;
    let (_tenant_id, raw_key) = create_test_tenant_and_key(&state, "del-none").await;

    let non_existent_id = Uuid::new_v4();
    let req = Request::builder()
        .uri(format!("/v1/deliveries/{non_existent_id}"))
        .method("GET")
        .header(header::AUTHORIZATION, format!("Bearer {raw_key}"))
        .body(Body::empty())
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}

// 3. Malformed ID
#[tokio::test]
async fn test_get_delivery_malformed_id() {
    let (app, state, _pool) = setup_test_app().await;
    let (_tenant_id, raw_key) = create_test_tenant_and_key(&state, "del-mal").await;

    let req = Request::builder()
        .uri("/v1/deliveries/not-a-valid-uuid-12345")
        .method("GET")
        .header(header::AUTHORIZATION, format!("Bearer {raw_key}"))
        .body(Body::empty())
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert!(res.status() == StatusCode::BAD_REQUEST || res.status() == StatusCode::NOT_FOUND || res.status().is_client_error());
}

// 4. Cross-tenant delivery
#[tokio::test]
async fn test_get_delivery_cross_tenant_rejected() {
    let (app, state, pool) = setup_test_app().await;
    let (tenant_a, _key_a) = create_test_tenant_and_key(&state, "del-cross-a").await;
    let (_tenant_b, key_b) = create_test_tenant_and_key(&state, "del-cross-b").await;

    let source_a = state
        .source_service
        .create_source(
            tenant_a,
            CreateSourceInput {
                name: "Src A".to_string(),
                slug: format!("src-a-{}", Uuid::new_v4().simple()),
                description: None,
                provider: "generic".to_string(),
                verification_type: "none".to_string(),
                secret: None,
            },
        )
        .await
        .unwrap();

    let dest_a = state
        .destination_service
        .create_destination(
            tenant_a,
            CreateDestinationInput {
                name: "Dest A".to_string(),
                url: "https://example.com/dest-a".to_string(),
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

    let sub_a = state
        .subscription_service
        .create_subscription(
            tenant_a,
            CreateSubscriptionInput {
                source_id: source_a.id,
                destination_id: dest_a.id,
                event_types: vec!["test.event".to_string()],
                filter_rules: None,
                transformation_template: None,
            },
        )
        .await
        .unwrap();

    let event_a = state
        .ingestion_service
        .create_event(
            tenant_a,
            CreateEventInput {
                source_id: Some(source_a.id),
                event_type: "test.event".to_string(),
                payload: serde_json::json!({"secret": "data"}),
                idempotency_key: None,
                headers: None,
            },
        )
        .await
        .unwrap();

    let delivery_repo = DeliveryRepository::new(&pool);
    let delivery_a = delivery_repo
        .create(tenant_a, event_a.id, sub_a.id, dest_a.id, 3)
        .await
        .unwrap();

    // Tenant B queries Tenant A's delivery -> 404 Not Found (strict multi-tenant isolation)
    let req = Request::builder()
        .uri(format!("/v1/deliveries/{}", delivery_a.id))
        .method("GET")
        .header(header::AUTHORIZATION, format!("Bearer {key_b}"))
        .body(Body::empty())
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}

// 5. GET /deliveries with Destination, Status, Date Filters & Keyset Pagination (Endpoint #44)
#[tokio::test]
async fn test_list_deliveries_filters_and_keyset_pagination() {
    let (app, state, pool) = setup_test_app().await;
    let (tenant_id, raw_key) = create_test_tenant_and_key(&state, "list-delivs").await;

    let source = state
        .source_service
        .create_source(
            tenant_id,
            CreateSourceInput {
                name: "Deliveries Test Source".to_string(),
                slug: format!("del-src-{}", Uuid::new_v4().simple()),
                description: None,
                provider: "generic".to_string(),
                verification_type: "none".to_string(),
                secret: None,
            },
        )
        .await
        .unwrap();

    let dest1 = state
        .destination_service
        .create_destination(
            tenant_id,
            CreateDestinationInput {
                name: "Dest 1".to_string(),
                url: "https://example.com/dest1".to_string(),
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

    let dest2 = state
        .destination_service
        .create_destination(
            tenant_id,
            CreateDestinationInput {
                name: "Dest 2".to_string(),
                url: "https://example.com/dest2".to_string(),
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

    let sub1 = state
        .subscription_service
        .create_subscription(
            tenant_id,
            CreateSubscriptionInput {
                source_id: source.id,
                destination_id: dest1.id,
                event_types: vec!["*".to_string()],
                filter_rules: None,
                transformation_template: None,
            },
        )
        .await
        .unwrap();

    let sub2 = state
        .subscription_service
        .create_subscription(
            tenant_id,
            CreateSubscriptionInput {
                source_id: source.id,
                destination_id: dest2.id,
                event_types: vec!["*".to_string()],
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
                event_type: "evt.item".to_string(),
                payload: json!({"key": "val"}),
                idempotency_key: None,
                headers: None,
            },
        )
        .await
        .unwrap();

    let deliv_repo = DeliveryRepository::new(&pool);
    // Create 4 deliveries: 2 for dest1, 2 for dest2
    let d1 = deliv_repo.create(tenant_id, event.id, sub1.id, dest1.id, 5).await.unwrap();
    let d2 = deliv_repo.create(tenant_id, event.id, sub2.id, dest2.id, 5).await.unwrap();
    deliv_repo.update_status(d2.id, "delivered", 1, None).await.unwrap();

    // 1. Filter by destination_id
    let req_dest = Request::builder()
        .uri(format!("/deliveries?destination_id={}", dest1.id))
        .method("GET")
        .header(header::AUTHORIZATION, format!("Bearer {raw_key}"))
        .body(Body::empty())
        .unwrap();

    let res_dest = app.clone().oneshot(req_dest).await.unwrap();
    assert_eq!(res_dest.status(), StatusCode::OK);
    let page_dest: Value = serde_json::from_slice(&axum::body::to_bytes(res_dest.into_body(), usize::MAX).await.unwrap()).unwrap();
    assert_eq!(page_dest["deliveries"].as_array().unwrap().len(), 1);
    assert_eq!(page_dest["deliveries"][0]["id"], d1.id.to_string());

    // 2. Filter by status = delivered
    let req_st = Request::builder()
        .uri("/deliveries?status=delivered")
        .method("GET")
        .header(header::AUTHORIZATION, format!("Bearer {raw_key}"))
        .body(Body::empty())
        .unwrap();

    let res_st = app.clone().oneshot(req_st).await.unwrap();
    assert_eq!(res_st.status(), StatusCode::OK);
    let page_st: Value = serde_json::from_slice(&axum::body::to_bytes(res_st.into_body(), usize::MAX).await.unwrap()).unwrap();
    assert_eq!(page_st["deliveries"].as_array().unwrap().len(), 1);
    assert_eq!(page_st["deliveries"][0]["id"], d2.id.to_string());

    // 3. Keyset Pagination
    let req_pg1 = Request::builder()
        .uri("/deliveries?limit=1")
        .method("GET")
        .header(header::AUTHORIZATION, format!("Bearer {raw_key}"))
        .body(Body::empty())
        .unwrap();

    let res_pg1 = app.clone().oneshot(req_pg1).await.unwrap();
    assert_eq!(res_pg1.status(), StatusCode::OK);
    let page_pg1: Value = serde_json::from_slice(&axum::body::to_bytes(res_pg1.into_body(), usize::MAX).await.unwrap()).unwrap();
    assert_eq!(page_pg1["deliveries"].as_array().unwrap().len(), 1);
    assert_eq!(page_pg1["has_more"], true);
    let cursor = page_pg1["next_cursor"].as_str().unwrap();

    let req_pg2 = Request::builder()
        .uri(format!("/deliveries?limit=1&cursor={cursor}"))
        .method("GET")
        .header(header::AUTHORIZATION, format!("Bearer {raw_key}"))
        .body(Body::empty())
        .unwrap();

    let res_pg2 = app.clone().oneshot(req_pg2).await.unwrap();
    assert_eq!(res_pg2.status(), StatusCode::OK);
    let page_pg2: Value = serde_json::from_slice(&axum::body::to_bytes(res_pg2.into_body(), usize::MAX).await.unwrap()).unwrap();
    assert_eq!(page_pg2["deliveries"].as_array().unwrap().len(), 1);

    // 4. Cross tenant returns empty result
    let (_tenant_b, key_b) = create_test_tenant_and_key(&state, "list-delivs-b").await;
    let req_b = Request::builder()
        .uri("/deliveries")
        .method("GET")
        .header(header::AUTHORIZATION, format!("Bearer {key_b}"))
        .body(Body::empty())
        .unwrap();

    let res_b = app.oneshot(req_b).await.unwrap();
    assert_eq!(res_b.status(), StatusCode::OK);
    let page_b: Value = serde_json::from_slice(&axum::body::to_bytes(res_b.into_body(), usize::MAX).await.unwrap()).unwrap();
    assert_eq!(page_b["deliveries"].as_array().unwrap().len(), 0);
}

// 6. Replay single delivery (Endpoint #46)
#[tokio::test]
async fn test_replay_delivery_pending_and_failed() {
    let (app, state, pool) = setup_test_app().await;
    let (tenant_id, raw_key) = create_test_tenant_and_key(&state, "replay-deliv").await;

    let source = state
        .source_service
        .create_source(
            tenant_id,
            CreateSourceInput {
                name: "Replay Source".to_string(),
                slug: format!("rep-src-{}", Uuid::new_v4().simple()),
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
                name: "Replay Dest".to_string(),
                url: "https://example.com/rep".to_string(),
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
            CreateSubscriptionInput {
                source_id: source.id,
                destination_id: dest.id,
                event_types: vec!["replay.test".to_string()],
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
                event_type: "replay.test".to_string(),
                payload: json!({"action": "replay"}),
                idempotency_key: None,
                headers: None,
            },
        )
        .await
        .unwrap();

    let deliv_repo = DeliveryRepository::new(&pool);
    let delivery = deliv_repo.create(tenant_id, event.id, sub.id, dest.id, 5).await.unwrap();
    deliv_repo.update_status(delivery.id, "failed", 3, None).await.unwrap();

    // 1. Replay with reset_attempt_count = false
    let req_no_reset = Request::builder()
        .uri(format!("/deliveries/{}/replay", delivery.id))
        .method("POST")
        .header(header::AUTHORIZATION, format!("Bearer {raw_key}"))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::to_vec(&ReplayDeliveryInput { reset_attempt_count: false }).unwrap()))
        .unwrap();

    let res_no_reset = app.clone().oneshot(req_no_reset).await.unwrap();
    assert_eq!(res_no_reset.status(), StatusCode::OK);
    let json_no_reset: Value = serde_json::from_slice(&axum::body::to_bytes(res_no_reset.into_body(), usize::MAX).await.unwrap()).unwrap();
    assert_eq!(json_no_reset["status"], "pending");
    assert_eq!(json_no_reset["attempt_count"], 3);

    // 2. Replay with reset_attempt_count = true
    let req_reset = Request::builder()
        .uri(format!("/deliveries/{}/replay", delivery.id))
        .method("POST")
        .header(header::AUTHORIZATION, format!("Bearer {raw_key}"))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::to_vec(&ReplayDeliveryInput { reset_attempt_count: true }).unwrap()))
        .unwrap();

    let res_reset = app.clone().oneshot(req_reset).await.unwrap();
    assert_eq!(res_reset.status(), StatusCode::OK);
    let json_reset: Value = serde_json::from_slice(&axum::body::to_bytes(res_reset.into_body(), usize::MAX).await.unwrap()).unwrap();
    assert_eq!(json_reset["status"], "pending");
    assert_eq!(json_reset["attempt_count"], 0);

    // 3. Cross-tenant replay -> 404
    let (_tenant_b, key_b) = create_test_tenant_and_key(&state, "replay-deliv-b").await;
    let req_cross = Request::builder()
        .uri(format!("/deliveries/{}/replay", delivery.id))
        .method("POST")
        .header(header::AUTHORIZATION, format!("Bearer {key_b}"))
        .body(Body::empty())
        .unwrap();

    let res_cross = app.oneshot(req_cross).await.unwrap();
    assert_eq!(res_cross.status(), StatusCode::NOT_FOUND);
}

// 7. Replay Event and Fanout (Endpoint #47)
#[tokio::test]
async fn test_replay_event_fanout() {
    let (app, state, pool) = setup_test_app().await;
    let (tenant_id, raw_key) = create_test_tenant_and_key(&state, "rep-ev-fanout").await;

    let source = state
        .source_service
        .create_source(
            tenant_id,
            CreateSourceInput {
                name: "Fanout Source".to_string(),
                slug: format!("fo-src-{}", Uuid::new_v4().simple()),
                description: None,
                provider: "generic".to_string(),
                verification_type: "none".to_string(),
                secret: None,
            },
        )
        .await
        .unwrap();

    let dest1 = state
        .destination_service
        .create_destination(
            tenant_id,
            CreateDestinationInput {
                name: "Dest 1".to_string(),
                url: "https://example.com/d1".to_string(),
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

    let sub1 = state
        .subscription_service
        .create_subscription(
            tenant_id,
            CreateSubscriptionInput {
                source_id: source.id,
                destination_id: dest1.id,
                event_types: vec!["order.created".to_string()],
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
                event_type: "order.created".to_string(),
                payload: json!({"order_id": "ord_555"}),
                idempotency_key: None,
                headers: None,
            },
        )
        .await
        .unwrap();

    // Create an initial delivery for sub1 and mark as failed
    let deliv_repo = DeliveryRepository::new(&pool);
    let d1 = deliv_repo.create(tenant_id, event.id, sub1.id, dest1.id, 5).await.unwrap();
    deliv_repo.update_status(d1.id, "failed", 5, None).await.unwrap();

    // Now add a second destination & subscription created AFTER the original event
    let dest2 = state
        .destination_service
        .create_destination(
            tenant_id,
            CreateDestinationInput {
                name: "Dest 2".to_string(),
                url: "https://example.com/d2".to_string(),
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

    let _sub2 = state
        .subscription_service
        .create_subscription(
            tenant_id,
            CreateSubscriptionInput {
                source_id: source.id,
                destination_id: dest2.id,
                event_types: vec!["order.*".to_string()],
                filter_rules: None,
                transformation_template: None,
            },
        )
        .await
        .unwrap();

    // Also add an inactive subscription which should be IGNORED
    let dest3 = state
        .destination_service
        .create_destination(
            tenant_id,
            CreateDestinationInput {
                name: "Dest 3".to_string(),
                url: "https://example.com/d3".to_string(),
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

    let sub3 = state
        .subscription_service
        .create_subscription(
            tenant_id,
            CreateSubscriptionInput {
                source_id: source.id,
                destination_id: dest3.id,
                event_types: vec!["order.created".to_string()],
                filter_rules: None,
                transformation_template: None,
            },
        )
        .await
        .unwrap();
    state
        .subscription_service
        .update_subscription(
            tenant_id,
            sub3.id,
            domain::dto::UpdateSubscriptionInput {
                is_active: Some(false),
                event_types: None,
                filter_rules: None,
                transformation_template: None,
            },
        )
        .await
        .unwrap();

    // Replay event: POST /events/{id}/replay
    let req = Request::builder()
        .uri(format!("/events/{}/replay", event.id))
        .method("POST")
        .header(header::AUTHORIZATION, format!("Bearer {raw_key}"))
        .body(Body::empty())
        .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let body_bytes = axum::body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let json_res: Value = serde_json::from_slice(&body_bytes).unwrap();

    assert_eq!(json_res["event_id"], event.id.to_string());
    assert_eq!(json_res["deliveries_created"], 1); // dest2 created
    assert_eq!(json_res["deliveries_reset"], 1);   // dest1 reset from failed to pending
    assert_eq!(json_res["total_deliveries"], 2);

    // Verify existing delivery was reset to pending
    let updated_d1 = deliv_repo.find_by_tenant_and_id(tenant_id, d1.id).await.unwrap().unwrap();
    assert_eq!(updated_d1.status, "pending");

    // Cross-tenant replay returns 404
    let (_tenant_b, key_b) = create_test_tenant_and_key(&state, "rep-ev-fanout-b").await;
    let req_cross = Request::builder()
        .uri(format!("/events/{}/replay", event.id))
        .method("POST")
        .header(header::AUTHORIZATION, format!("Bearer {key_b}"))
        .body(Body::empty())
        .unwrap();

    let res_cross = app.oneshot(req_cross).await.unwrap();
    assert_eq!(res_cross.status(), StatusCode::NOT_FOUND);
}

// 8. Batch Replay (Endpoint #48)
#[tokio::test]
async fn test_replay_events_batch() {
    let (app, state, pool) = setup_test_app().await;
    let (tenant_id, raw_key) = create_test_tenant_and_key(&state, "batch-rep").await;

    let source = state
        .source_service
        .create_source(
            tenant_id,
            CreateSourceInput {
                name: "Batch Source".to_string(),
                slug: format!("b-src-{}", Uuid::new_v4().simple()),
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
                name: "Batch Dest".to_string(),
                url: "https://example.com/batch".to_string(),
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
            CreateSubscriptionInput {
                source_id: source.id,
                destination_id: dest.id,
                event_types: vec!["batch.item".to_string()],
                filter_rules: None,
                transformation_template: None,
            },
        )
        .await
        .unwrap();

    let deliv_repo = DeliveryRepository::new(&pool);

    // Create 3 events and deliveries, 2 failed and 1 delivered
    for i in 1..=3 {
        let ev = state
            .ingestion_service
            .create_event(
                tenant_id,
                CreateEventInput {
                    source_id: Some(source.id),
                    event_type: "batch.item".to_string(),
                    payload: json!({"idx": i}),
                    idempotency_key: None,
                    headers: None,
                },
            )
            .await
            .unwrap();

        let d = deliv_repo.create(tenant_id, ev.id, sub.id, dest.id, 5).await.unwrap();
        if i <= 2 {
            deliv_repo.update_status(d.id, "failed", 5, None).await.unwrap();
        } else {
            deliv_repo.update_status(d.id, "delivered", 1, None).await.unwrap();
        }
    }

    // Call POST /events/replay-batch with status_filter = "failed"
    let batch_input = ReplayBatchInput {
        destination_id: Some(dest.id),
        source_id: Some(source.id),
        from: None,
        to: None,
        status_filter: Some("failed".to_string()),
    };

    let req = Request::builder()
        .uri("/events/replay-batch")
        .method("POST")
        .header(header::AUTHORIZATION, format!("Bearer {raw_key}"))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::to_vec(&batch_input).unwrap()))
        .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let body_bytes = axum::body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let json_res: Value = serde_json::from_slice(&body_bytes).unwrap();

    assert_eq!(json_res["replayed_count"], 2);
    assert_eq!(json_res["has_more"], false);

    // Cross-tenant replay returns 0
    let (_tenant_b, key_b) = create_test_tenant_and_key(&state, "batch-rep-b").await;
    let req_b = Request::builder()
        .uri("/events/replay-batch")
        .method("POST")
        .header(header::AUTHORIZATION, format!("Bearer {key_b}"))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::to_vec(&batch_input).unwrap()))
        .unwrap();

    let res_b = app.oneshot(req_b).await.unwrap();
    assert_eq!(res_b.status(), StatusCode::OK);
    let json_b: Value = serde_json::from_slice(&axum::body::to_bytes(res_b.into_body(), usize::MAX).await.unwrap()).unwrap();
    assert_eq!(json_b["replayed_count"], 0);
}
