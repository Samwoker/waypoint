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
use domain::dto::{CreateApiKeyInput, CreateDestinationInput, CreateSourceInput, CreateSubscriptionInput, CreateTenantInput};

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

// 1. Successful subscription creation test
#[tokio::test]
async fn test_successful_subscription_creation() {
    let (app, state, _pool) = setup_test_app().await;
    let (tenant_id, raw_key) = create_test_tenant_and_key(&state, "succ-sub").await;

    // Create source
    let source = state
        .source_service
        .create_source(
            tenant_id,
            CreateSourceInput {
                name: "Stripe Source".to_string(),
                slug: format!("stripe-{}", Uuid::new_v4().simple()),
                description: None,
                provider: "stripe".to_string(),
                verification_type: "hmac_sha256".to_string(),
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
                name: "Analytics Destination".to_string(),
                url: "https://analytics.example.com/events".to_string(),
                description: None,
                rate_limit_rps: Some(100),
                timeout_ms: Some(5000),
                secret: None,
                max_retries: Some(5),
                headers: None,
            },
        )
        .await
        .unwrap();

    let payload = json!({
        "source_id": source.id,
        "destination_id": dest.id,
        "event_types": ["payment.succeeded", "order.created"],
        "filter_rules": {
            "amount": { "$gt": 100 }
        }
    });

    let req = Request::builder()
        .uri("/v1/subscriptions")
        .method("POST")
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::AUTHORIZATION, format!("Bearer {raw_key}"))
        .body(Body::from(payload.to_string()))
        .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);

    let body_bytes = axum::body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let json_res: Value = serde_json::from_slice(&body_bytes).unwrap();

    assert_eq!(json_res["tenant_id"], tenant_id.to_string());
    assert_eq!(json_res["source_id"], source.id.to_string());
    assert_eq!(json_res["destination_id"], dest.id.to_string());
    assert_eq!(json_res["is_active"], true);
    assert!(json_res["id"].is_string());
    assert_eq!(
        json_res["event_types"].as_array().unwrap(),
        &vec![json!("payment.succeeded"), json!("order.created")]
    );

    // Also verify /api/v1/subscriptions alias works
    let payload2 = json!({
        "source_id": source.id,
        "destination_id": dest.id,
        "event_types": ["*"]
    });

    let req2 = Request::builder()
        .uri("/api/v1/subscriptions")
        .method("POST")
        .header(header::CONTENT_TYPE, "application/json")
        .header("X-Api-Key", &raw_key)
        .body(Body::from(payload2.to_string()))
        .unwrap();

    let res2 = app.oneshot(req2).await.unwrap();
    assert_eq!(res2.status(), StatusCode::CREATED);
}

// 2. Source does not exist test
#[tokio::test]
async fn test_subscription_source_not_found() {
    let (app, state, _pool) = setup_test_app().await;
    let (tenant_id, raw_key) = create_test_tenant_and_key(&state, "sub-no-src").await;

    let dest = state
        .destination_service
        .create_destination(
            tenant_id,
            CreateDestinationInput {
                name: "Valid Dest".to_string(),
                url: "https://example.com/dest".to_string(),
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

    let non_existent_source_id = Uuid::new_v4();
    let payload = json!({
        "source_id": non_existent_source_id,
        "destination_id": dest.id,
        "event_types": ["*"]
    });

    let req = Request::builder()
        .uri("/v1/subscriptions")
        .method("POST")
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::AUTHORIZATION, format!("Bearer {raw_key}"))
        .body(Body::from(payload.to_string()))
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}

// 3. Destination does not exist test
#[tokio::test]
async fn test_subscription_destination_not_found() {
    let (app, state, _pool) = setup_test_app().await;
    let (tenant_id, raw_key) = create_test_tenant_and_key(&state, "sub-no-dest").await;

    let source = state
        .source_service
        .create_source(
            tenant_id,
            CreateSourceInput {
                name: "Valid Source".to_string(),
                slug: format!("source-{}", Uuid::new_v4().simple()),
                description: None,
                provider: "generic".to_string(),
                verification_type: "none".to_string(),
                secret: None,
            },
        )
        .await
        .unwrap();

    let non_existent_dest_id = Uuid::new_v4();
    let payload = json!({
        "source_id": source.id,
        "destination_id": non_existent_dest_id,
        "event_types": ["*"]
    });

    let req = Request::builder()
        .uri("/v1/subscriptions")
        .method("POST")
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::AUTHORIZATION, format!("Bearer {raw_key}"))
        .body(Body::from(payload.to_string()))
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}

// 4. Source belongs to another tenant test
#[tokio::test]
async fn test_subscription_cross_tenant_source_rejected() {
    let (app, state, _pool) = setup_test_app().await;
    let (tenant_a, _key_a) = create_test_tenant_and_key(&state, "sub-src-a").await;
    let (tenant_b, key_b) = create_test_tenant_and_key(&state, "sub-src-b").await;

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

    // Tenant B creates Destination B
    let dest_b = state
        .destination_service
        .create_destination(
            tenant_b,
            CreateDestinationInput {
                name: "Dest B".to_string(),
                url: "https://tenant-b.com/webhook".to_string(),
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

    // Tenant B attempts to connect Source A to Destination B -> Rejected!
    let payload = json!({
        "source_id": source_a.id,
        "destination_id": dest_b.id,
        "event_types": ["*"]
    });

    let req = Request::builder()
        .uri("/v1/subscriptions")
        .method("POST")
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::AUTHORIZATION, format!("Bearer {key_b}"))
        .body(Body::from(payload.to_string()))
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert!(res.status() == StatusCode::FORBIDDEN || res.status() == StatusCode::NOT_FOUND);
}

// 5. Destination belongs to another tenant test
#[tokio::test]
async fn test_subscription_cross_tenant_destination_rejected() {
    let (app, state, _pool) = setup_test_app().await;
    let (tenant_a, _key_a) = create_test_tenant_and_key(&state, "sub-dst-a").await;
    let (tenant_b, key_b) = create_test_tenant_and_key(&state, "sub-dst-b").await;

    // Tenant A creates Destination A
    let dest_a = state
        .destination_service
        .create_destination(
            tenant_a,
            CreateDestinationInput {
                name: "Dest A".to_string(),
                url: "https://tenant-a.com/webhook".to_string(),
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

    // Tenant B creates Source B
    let source_b = state
        .source_service
        .create_source(
            tenant_b,
            CreateSourceInput {
                name: "Source B".to_string(),
                slug: format!("src-b-{}", Uuid::new_v4().simple()),
                description: None,
                provider: "generic".to_string(),
                verification_type: "none".to_string(),
                secret: None,
            },
        )
        .await
        .unwrap();

    // Tenant B attempts to connect Source B to Destination A -> Rejected!
    let payload = json!({
        "source_id": source_b.id,
        "destination_id": dest_a.id,
        "event_types": ["*"]
    });

    let req = Request::builder()
        .uri("/v1/subscriptions")
        .method("POST")
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::AUTHORIZATION, format!("Bearer {key_b}"))
        .body(Body::from(payload.to_string()))
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert!(res.status() == StatusCode::FORBIDDEN || res.status() == StatusCode::NOT_FOUND);
}

// 6. Invalid event types test
#[tokio::test]
async fn test_subscription_invalid_event_types() {
    let (app, state, _pool) = setup_test_app().await;
    let (tenant_id, raw_key) = create_test_tenant_and_key(&state, "sub-inv-et").await;

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

    let dest = state
        .destination_service
        .create_destination(tenant_id, CreateDestinationInput {
            name: "Dst".to_string(),
            url: "https://example.com/dest".to_string(),
            description: None,
            rate_limit_rps: None,
            timeout_ms: None,
            secret: None,
            max_retries: None,
            headers: None,
        })
        .await
        .unwrap();

    // Empty event types array
    let req1 = Request::builder()
        .uri("/v1/subscriptions")
        .method("POST")
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::AUTHORIZATION, format!("Bearer {raw_key}"))
        .body(Body::from(
            json!({
                "source_id": source.id,
                "destination_id": dest.id,
                "event_types": []
            })
            .to_string(),
        ))
        .unwrap();

    let res1 = app.clone().oneshot(req1).await.unwrap();
    assert_eq!(res1.status(), StatusCode::UNPROCESSABLE_ENTITY);

    // Array containing empty string
    let req2 = Request::builder()
        .uri("/v1/subscriptions")
        .method("POST")
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::AUTHORIZATION, format!("Bearer {raw_key}"))
        .body(Body::from(
            json!({
                "source_id": source.id,
                "destination_id": dest.id,
                "event_types": ["   "]
            })
            .to_string(),
        ))
        .unwrap();

    let res2 = app.oneshot(req2).await.unwrap();
    assert_eq!(res2.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

// 7. Invalid filter test
#[tokio::test]
async fn test_subscription_invalid_filter() {
    let (app, state, _pool) = setup_test_app().await;
    let (tenant_id, raw_key) = create_test_tenant_and_key(&state, "sub-inv-flt").await;

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

    let dest = state
        .destination_service
        .create_destination(tenant_id, CreateDestinationInput {
            name: "Dst".to_string(),
            url: "https://example.com/dest".to_string(),
            description: None,
            rate_limit_rps: None,
            timeout_ms: None,
            secret: None,
            max_retries: None,
            headers: None,
        })
        .await
        .unwrap();

    // Filter passed as a string or array instead of object
    let req = Request::builder()
        .uri("/v1/subscriptions")
        .method("POST")
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::AUTHORIZATION, format!("Bearer {raw_key}"))
        .body(Body::from(
            json!({
                "source_id": source.id,
                "destination_id": dest.id,
                "event_types": ["payment.created"],
                "filter_rules": ["invalid", "filter", "array"]
            })
            .to_string(),
        ))
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

// 8. Unauthenticated request test
#[tokio::test]
async fn test_subscription_unauthenticated() {
    let (app, _state, _pool) = setup_test_app().await;

    let req = Request::builder()
        .uri("/v1/subscriptions")
        .method("POST")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "source_id": Uuid::new_v4(),
                "destination_id": Uuid::new_v4(),
                "event_types": ["*"]
            })
            .to_string(),
        ))
        .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);

    let req_invalid = Request::builder()
        .uri("/v1/subscriptions")
        .method("POST")
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::AUTHORIZATION, "Bearer invalid_nonexistent_key")
        .body(Body::from(
            json!({
                "source_id": Uuid::new_v4(),
                "destination_id": Uuid::new_v4(),
                "event_types": ["*"]
            })
            .to_string(),
        ))
        .unwrap();

    let res_invalid = app.oneshot(req_invalid).await.unwrap();
    assert_eq!(res_invalid.status(), StatusCode::UNAUTHORIZED);
}

// 9. Database failure test
#[tokio::test]
async fn test_subscription_database_failure() {
    let (_, state, _pool) = setup_test_app().await;
    let (tenant_id, _) = create_test_tenant_and_key(&state, "sub-db-fail").await;

    let invalid_pool = Arc::new(
        sqlx::postgres::PgPoolOptions::new()
            .acquire_timeout(std::time::Duration::from_millis(10))
            .connect_lazy("postgres://invalid:invalid@localhost:9999/nonexistent")
            .unwrap(),
    );
    let broken_subscription_service = domain::services::SubscriptionService::new(invalid_pool);

    let res = broken_subscription_service
        .create_subscription(
            tenant_id,
            CreateSubscriptionInput {
                source_id: Uuid::new_v4(),
                destination_id: Uuid::new_v4(),
                event_types: vec!["*".to_string()],
                filter_rules: None,
                transformation_template: None,
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

// --- GET /v1/subscriptions Tests ---

// 1. Returns tenant subscriptions
#[tokio::test]
async fn test_get_subscriptions_returns_tenant_subscriptions() {
    let (app, state, _pool) = setup_test_app().await;
    let (tenant_id, raw_key) = create_test_tenant_and_key(&state, "get-subs").await;

    let source = state
        .source_service
        .create_source(tenant_id, CreateSourceInput {
            name: "Shared Source".to_string(),
            slug: format!("src-{}", Uuid::new_v4().simple()),
            description: None,
            provider: "generic".to_string(),
            verification_type: "none".to_string(),
            secret: None,
        })
        .await
        .unwrap();

    let dest = state
        .destination_service
        .create_destination(tenant_id, CreateDestinationInput {
            name: "Shared Destination".to_string(),
            url: "https://example.com/dest".to_string(),
            description: None,
            rate_limit_rps: None,
            timeout_ms: None,
            secret: None,
            max_retries: None,
            headers: None,
        })
        .await
        .unwrap();

    // Create 3 subscriptions
    for i in 1..=3 {
        let _ = state
            .subscription_service
            .create_subscription(
                tenant_id,
                CreateSubscriptionInput {
                    source_id: source.id,
                    destination_id: dest.id,
                    event_types: vec![format!("event.type.{i}")],
                    filter_rules: Some(json!({"index": i})),
                    transformation_template: None,
                },
            )
            .await
            .unwrap();
    }

    // Call GET /v1/subscriptions
    let req = Request::builder()
        .uri("/v1/subscriptions?limit=10&offset=0")
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
        assert_eq!(item["source_id"], source.id.to_string());
        assert_eq!(item["destination_id"], dest.id.to_string());
        assert_eq!(item["is_active"], true);
    }

    // Also verify /api/v1/subscriptions alias
    let req_api = Request::builder()
        .uri("/api/v1/subscriptions")
        .method("GET")
        .header("X-Api-Key", &raw_key)
        .body(Body::empty())
        .unwrap();

    let res_api = app.oneshot(req_api).await.unwrap();
    assert_eq!(res_api.status(), StatusCode::OK);
    let list_api: Value = serde_json::from_slice(&axum::body::to_bytes(res_api.into_body(), usize::MAX).await.unwrap()).unwrap();
    assert_eq!(list_api.as_array().unwrap().len(), 3);
}

// 2. Empty result
#[tokio::test]
async fn test_get_subscriptions_empty_result() {
    let (app, state, _pool) = setup_test_app().await;
    let (_tenant_id, raw_key) = create_test_tenant_and_key(&state, "get-subs-empty").await;

    let req = Request::builder()
        .uri("/v1/subscriptions")
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

// 3. Cross-tenant isolation
#[tokio::test]
async fn test_get_subscriptions_cross_tenant_isolation() {
    let (app, state, _pool) = setup_test_app().await;
    let (tenant_a, key_a) = create_test_tenant_and_key(&state, "sub-cross-a").await;
    let (tenant_b, key_b) = create_test_tenant_and_key(&state, "sub-cross-b").await;

    // Tenant A resources
    let source_a = state.source_service.create_source(tenant_a, CreateSourceInput {
        name: "Source A".to_string(),
        slug: format!("src-a-{}", Uuid::new_v4().simple()),
        description: None,
        provider: "generic".to_string(),
        verification_type: "none".to_string(),
        secret: None,
    }).await.unwrap();
    let dest_a = state.destination_service.create_destination(tenant_a, CreateDestinationInput {
        name: "Dest A".to_string(),
        url: "https://tenant-a.com/d".to_string(),
        description: None,
        rate_limit_rps: None,
        timeout_ms: None,
        secret: None,
        max_retries: None,
        headers: None,
    }).await.unwrap();
    let sub_a = state.subscription_service.create_subscription(tenant_a, CreateSubscriptionInput {
        source_id: source_a.id,
        destination_id: dest_a.id,
        event_types: vec!["event.a".to_string()],
        filter_rules: None,
        transformation_template: None,
    }).await.unwrap();

    // Tenant B resources
    let source_b = state.source_service.create_source(tenant_b, CreateSourceInput {
        name: "Source B".to_string(),
        slug: format!("src-b-{}", Uuid::new_v4().simple()),
        description: None,
        provider: "generic".to_string(),
        verification_type: "none".to_string(),
        secret: None,
    }).await.unwrap();
    let dest_b = state.destination_service.create_destination(tenant_b, CreateDestinationInput {
        name: "Dest B".to_string(),
        url: "https://tenant-b.com/d".to_string(),
        description: None,
        rate_limit_rps: None,
        timeout_ms: None,
        secret: None,
        max_retries: None,
        headers: None,
    }).await.unwrap();
    let sub_b = state.subscription_service.create_subscription(tenant_b, CreateSubscriptionInput {
        source_id: source_b.id,
        destination_id: dest_b.id,
        event_types: vec!["event.b".to_string()],
        filter_rules: None,
        transformation_template: None,
    }).await.unwrap();

    // Query as Tenant A -> only see sub_a
    let req_a = Request::builder()
        .uri("/v1/subscriptions")
        .method("GET")
        .header(header::AUTHORIZATION, format!("Bearer {key_a}"))
        .body(Body::empty())
        .unwrap();

    let res_a = app.clone().oneshot(req_a).await.unwrap();
    assert_eq!(res_a.status(), StatusCode::OK);
    let list_a: Value = serde_json::from_slice(&axum::body::to_bytes(res_a.into_body(), usize::MAX).await.unwrap()).unwrap();
    let arr_a = list_a.as_array().unwrap();
    assert_eq!(arr_a.len(), 1);
    assert_eq!(arr_a[0]["id"], sub_a.id.to_string());
    assert_eq!(arr_a[0]["tenant_id"], tenant_a.to_string());

    // Query as Tenant B -> only see sub_b
    let req_b = Request::builder()
        .uri("/v1/subscriptions")
        .method("GET")
        .header(header::AUTHORIZATION, format!("Bearer {key_b}"))
        .body(Body::empty())
        .unwrap();

    let res_b = app.clone().oneshot(req_b).await.unwrap();
    assert_eq!(res_b.status(), StatusCode::OK);
    let list_b: Value = serde_json::from_slice(&axum::body::to_bytes(res_b.into_body(), usize::MAX).await.unwrap()).unwrap();
    let arr_b = list_b.as_array().unwrap();
    assert_eq!(arr_b.len(), 1);
    assert_eq!(arr_b[0]["id"], sub_b.id.to_string());
    assert_eq!(arr_b[0]["tenant_id"], tenant_b.to_string());

    // Query parameter spoofing ?tenant_id=<tenant_a> as Tenant B
    let req_spoof = Request::builder()
        .uri(format!("/v1/subscriptions?tenant_id={tenant_a}"))
        .method("GET")
        .header(header::AUTHORIZATION, format!("Bearer {key_b}"))
        .body(Body::empty())
        .unwrap();

    let res_spoof = app.oneshot(req_spoof).await.unwrap();
    assert_eq!(res_spoof.status(), StatusCode::OK);
    let list_spoof: Value = serde_json::from_slice(&axum::body::to_bytes(res_spoof.into_body(), usize::MAX).await.unwrap()).unwrap();
    let arr_spoof = list_spoof.as_array().unwrap();
    assert_eq!(arr_spoof.len(), 1);
    assert_eq!(arr_spoof[0]["id"], sub_b.id.to_string());
}

// 4. Authentication failure
#[tokio::test]
async fn test_get_subscriptions_unauthenticated() {
    let (app, _state, _pool) = setup_test_app().await;

    // Missing auth header
    let req = Request::builder()
        .uri("/v1/subscriptions")
        .method("GET")
        .body(Body::empty())
        .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);

    // Invalid token
    let req_invalid = Request::builder()
        .uri("/v1/subscriptions")
        .method("GET")
        .header(header::AUTHORIZATION, "Bearer invalid_nonexistent_key_123")
        .body(Body::empty())
        .unwrap();

    let res_invalid = app.oneshot(req_invalid).await.unwrap();
    assert_eq!(res_invalid.status(), StatusCode::UNAUTHORIZED);
}

// 5. Database failure
#[tokio::test]
async fn test_get_subscriptions_database_failure() {
    let (_app, state, _pool) = setup_test_app().await;
    let (tenant_id, _) = create_test_tenant_and_key(&state, "get-sub-db-fail").await;

    let invalid_pool = Arc::new(
        sqlx::postgres::PgPoolOptions::new()
            .acquire_timeout(std::time::Duration::from_millis(10))
            .connect_lazy("postgres://invalid:invalid@localhost:9999/nonexistent")
            .unwrap(),
    );
    let broken_subscription_service = domain::services::SubscriptionService::new(invalid_pool);

    let res = broken_subscription_service.list_subscriptions(tenant_id, 20, 0).await;
    assert!(res.is_err());
    match res.unwrap_err() {
        relay_core::error::CoreError::Internal(msg) => {
            assert!(msg.contains("Database error") || msg.contains("connection"));
        }
        err => panic!("Expected CoreError::Internal, got {:?}", err),
    }
}
