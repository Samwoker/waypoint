use axum::{
    body::Body,
    http::{header, Request, StatusCode},
};
use serde_json::Value;
use sqlx::PgPool;
use tower::ServiceExt;
use uuid::Uuid;
use domain::dto::{
    CreateApiKeyInput, CreateDestinationInput, CreateSourceInput,
    CreateSubscriptionInput, CreateTenantInput,
};

use api::{create_router, AppState};
use relay_core::config::Config;
use data::pool::{create_pg_pool, run_migrations};
use data::queue::RedisQueue;

async fn setup_test_app() -> (axum::Router, AppState, PgPool) {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/webhook_relay".to_string());
    let redis_url = std::env::var("REDIS_URL")
        .unwrap_or_else(|_| "redis://localhost:6379".to_string());
    let pool = create_pg_pool(&database_url).await.expect("DB connect");
    run_migrations(&pool).await.expect("Migrations");
    let queue = RedisQueue::new(&redis_url).await.expect("Redis connect");
    let config = Config {
        database_url, redis_url,
        data_encryption_key: "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f".to_string(),
        jwt_secret: "super-secret-jwt-key-with-at-least-32-chars-length".to_string(),
        api_port: 3000,
        environment: "test".to_string(),
    };
    let state = AppState::new(config, pool.clone(), queue).expect("AppState");
    let router = create_router(state.clone());
    (router, state, pool)
}

async fn create_tenant_and_key(state: &AppState, slug_prefix: &str) -> (Uuid, String) {
    let slug = format!("{slug_prefix}-{}", Uuid::new_v4().simple());
    let tenant = state.tenant_service.create_tenant(CreateTenantInput {
        name: format!("Tenant {slug}"),
        slug,
    }).await.expect("create tenant");
    let key = state.auth_service.create_api_key(tenant.id, CreateApiKeyInput {
        name: "Test Key".to_string(),
        expires_at: None,
    }).await.expect("create api key");
    (tenant.id, key.raw_key)
}

async fn create_subscription(state: &AppState, tenant_id: Uuid) -> Uuid {
    let source = state.source_service.create_source(tenant_id, CreateSourceInput {
        name: "Transformation Source".to_string(),
        slug: format!("tf-src-{}", Uuid::new_v4().simple()),
        description: None,
        provider: "generic".to_string(),
        verification_type: "none".to_string(),
        secret: None,
    }).await.unwrap();

    let dest = state.destination_service.create_destination(tenant_id, CreateDestinationInput {
        name: "Transformation Dest".to_string(),
        url: "https://example.com/transform".to_string(),
        description: None,
        rate_limit_rps: None,
        timeout_ms: None,
        max_retries: None,
        headers: None,
        secret: None,
    }).await.unwrap();

    state.subscription_service.create_subscription(tenant_id, CreateSubscriptionInput {
        source_id: source.id,
        destination_id: dest.id,
        event_types: vec!["tf.event".to_string()],
        filter_rules: None,
        transformation_template: None,
    }).await.unwrap().id
}

// 1. Create + list transformations (Endpoint #59, #58)
#[tokio::test]
async fn test_create_and_list_transformations() {
    let (app, state, _pool) = setup_test_app().await;
    let (tenant_id, raw_key) = create_tenant_and_key(&state, "tf-cl").await;
    let sub_id = create_subscription(&state, tenant_id).await;

    let create_body = serde_json::json!({
        "subscription_id": sub_id,
        "rules": [
            { "source_path": "$.customer.id", "dest_path": "$.user.id" },
            { "source_path": "$.order.amount", "dest_path": "$.payment.total" }
        ]
    });

    let create_req = Request::builder()
        .uri("/transformations")
        .method("POST")
        .header(header::AUTHORIZATION, format!("Bearer {raw_key}"))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::to_string(&create_body).unwrap()))
        .unwrap();

    let create_res = app.clone().oneshot(create_req).await.unwrap();
    assert_eq!(create_res.status(), StatusCode::CREATED);
    let created: Value = serde_json::from_slice(&axum::body::to_bytes(create_res.into_body(), usize::MAX).await.unwrap()).unwrap();
    assert!(!created["id"].as_str().unwrap().is_empty());
    assert_eq!(created["subscription_id"], sub_id.to_string());
    let rules = created["rules"].as_array().unwrap();
    assert_eq!(rules.len(), 2);
    assert_eq!(rules[0]["source_path"], "$.customer.id");

    // List
    let list_req = Request::builder()
        .uri(format!("/transformations?subscription_id={sub_id}"))
        .method("GET")
        .header(header::AUTHORIZATION, format!("Bearer {raw_key}"))
        .body(Body::empty())
        .unwrap();

    let list_res = app.oneshot(list_req).await.unwrap();
    assert_eq!(list_res.status(), StatusCode::OK);
    let list: Value = serde_json::from_slice(&axum::body::to_bytes(list_res.into_body(), usize::MAX).await.unwrap()).unwrap();
    assert!(list.as_array().unwrap().len() >= 1);
}

// 2. Invalid source_path JSONPath (#59)
#[tokio::test]
async fn test_create_transformation_invalid_source_path() {
    let (app, state, _pool) = setup_test_app().await;
    let (tenant_id, raw_key) = create_tenant_and_key(&state, "tf-inv-src").await;
    let sub_id = create_subscription(&state, tenant_id).await;

    let body = serde_json::json!({
        "subscription_id": sub_id,
        "rules": [{ "source_path": "customer.id", "dest_path": "$.user.id" }]
    });

    let req = Request::builder()
        .uri("/transformations")
        .method("POST")
        .header(header::AUTHORIZATION, format!("Bearer {raw_key}"))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::to_string(&body).unwrap()))
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

// 3. Invalid dest_path JSONPath (#59)
#[tokio::test]
async fn test_create_transformation_invalid_dest_path() {
    let (app, state, _pool) = setup_test_app().await;
    let (tenant_id, raw_key) = create_tenant_and_key(&state, "tf-inv-dst").await;
    let sub_id = create_subscription(&state, tenant_id).await;

    let body = serde_json::json!({
        "subscription_id": sub_id,
        "rules": [{ "source_path": "$.customer.id", "dest_path": "user.id" }]
    });

    let req = Request::builder()
        .uri("/transformations")
        .method("POST")
        .header(header::AUTHORIZATION, format!("Bearer {raw_key}"))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::to_string(&body).unwrap()))
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

// 4. Empty rules rejected (#59)
#[tokio::test]
async fn test_create_transformation_empty_rules() {
    let (app, state, _pool) = setup_test_app().await;
    let (tenant_id, raw_key) = create_tenant_and_key(&state, "tf-empty").await;
    let sub_id = create_subscription(&state, tenant_id).await;

    let body = serde_json::json!({ "subscription_id": sub_id, "rules": [] });

    let req = Request::builder()
        .uri("/transformations")
        .method("POST")
        .header(header::AUTHORIZATION, format!("Bearer {raw_key}"))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::to_string(&body).unwrap()))
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

// 5. Nonexistent subscription (#59)
#[tokio::test]
async fn test_create_transformation_nonexistent_subscription() {
    let (app, state, _pool) = setup_test_app().await;
    let (_tenant_id, raw_key) = create_tenant_and_key(&state, "tf-nosub").await;
    let fake_sub = Uuid::new_v4();

    let body = serde_json::json!({
        "subscription_id": fake_sub,
        "rules": [{ "source_path": "$.a", "dest_path": "$.b" }]
    });

    let req = Request::builder()
        .uri("/transformations")
        .method("POST")
        .header(header::AUTHORIZATION, format!("Bearer {raw_key}"))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::to_string(&body).unwrap()))
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}

// 6. Cross-tenant subscription access (#59)
#[tokio::test]
async fn test_create_transformation_cross_tenant_subscription() {
    let (app, state, _pool) = setup_test_app().await;
    let (tenant_a_id, _key_a) = create_tenant_and_key(&state, "tf-ct-a").await;
    let (_tenant_b_id, key_b) = create_tenant_and_key(&state, "tf-ct-b").await;
    let sub_id = create_subscription(&state, tenant_a_id).await;

    let body = serde_json::json!({
        "subscription_id": sub_id,
        "rules": [{ "source_path": "$.a", "dest_path": "$.b" }]
    });

    let req = Request::builder()
        .uri("/transformations")
        .method("POST")
        .header(header::AUTHORIZATION, format!("Bearer {key_b}"))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::to_string(&body).unwrap()))
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}

// 7. Update transformation (#60)
#[tokio::test]
async fn test_update_transformation() {
    let (app, state, _pool) = setup_test_app().await;
    let (tenant_id, raw_key) = create_tenant_and_key(&state, "tf-upd").await;
    let sub_id = create_subscription(&state, tenant_id).await;

    // Create first
    let create_body = serde_json::json!({
        "subscription_id": sub_id,
        "rules": [{ "source_path": "$.old", "dest_path": "$.new" }]
    });
    let create_req = Request::builder()
        .uri("/transformations")
        .method("POST")
        .header(header::AUTHORIZATION, format!("Bearer {raw_key}"))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::to_string(&create_body).unwrap()))
        .unwrap();
    let create_res = app.clone().oneshot(create_req).await.unwrap();
    let created: Value = serde_json::from_slice(&axum::body::to_bytes(create_res.into_body(), usize::MAX).await.unwrap()).unwrap();
    let tf_id = created["id"].as_str().unwrap().to_string();

    // Update
    let update_body = serde_json::json!({
        "rules": [
            { "source_path": "$.customer.name", "dest_path": "$.user.name" }
        ]
    });
    let update_req = Request::builder()
        .uri(format!("/transformations/{tf_id}"))
        .method("PATCH")
        .header(header::AUTHORIZATION, format!("Bearer {raw_key}"))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::to_string(&update_body).unwrap()))
        .unwrap();

    let update_res = app.clone().oneshot(update_req).await.unwrap();
    assert_eq!(update_res.status(), StatusCode::OK);
    let updated: Value = serde_json::from_slice(&axum::body::to_bytes(update_res.into_body(), usize::MAX).await.unwrap()).unwrap();
    let rules = updated["rules"].as_array().unwrap();
    assert_eq!(rules.len(), 1);
    assert_eq!(rules[0]["source_path"], "$.customer.name");

    // Update with invalid JSONPath returns 422
    let bad_update = serde_json::json!({
        "rules": [{ "source_path": "bad_path", "dest_path": "$.b" }]
    });
    let bad_req = Request::builder()
        .uri(format!("/transformations/{tf_id}"))
        .method("PATCH")
        .header(header::AUTHORIZATION, format!("Bearer {raw_key}"))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::to_string(&bad_update).unwrap()))
        .unwrap();

    let bad_res = app.oneshot(bad_req).await.unwrap();
    assert_eq!(bad_res.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

// 8. Cross-tenant update (#60)
#[tokio::test]
async fn test_update_transformation_cross_tenant() {
    let (app, state, _pool) = setup_test_app().await;
    let (tenant_a_id, key_a) = create_tenant_and_key(&state, "tf-upd-ct-a").await;
    let (_tenant_b_id, key_b) = create_tenant_and_key(&state, "tf-upd-ct-b").await;
    let sub_id = create_subscription(&state, tenant_a_id).await;

    let create_body = serde_json::json!({
        "subscription_id": sub_id,
        "rules": [{ "source_path": "$.a", "dest_path": "$.b" }]
    });
    let create_req = Request::builder()
        .uri("/transformations")
        .method("POST")
        .header(header::AUTHORIZATION, format!("Bearer {key_a}"))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::to_string(&create_body).unwrap()))
        .unwrap();
    let create_res = app.clone().oneshot(create_req).await.unwrap();
    let created: Value = serde_json::from_slice(&axum::body::to_bytes(create_res.into_body(), usize::MAX).await.unwrap()).unwrap();
    let tf_id = created["id"].as_str().unwrap().to_string();

    let update_body = serde_json::json!({ "rules": [{ "source_path": "$.x", "dest_path": "$.y" }] });
    let req = Request::builder()
        .uri(format!("/transformations/{tf_id}"))
        .method("PATCH")
        .header(header::AUTHORIZATION, format!("Bearer {key_b}"))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::to_string(&update_body).unwrap()))
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}

// 9. Delete transformation (#61)
#[tokio::test]
async fn test_delete_transformation() {
    let (app, state, _pool) = setup_test_app().await;
    let (tenant_id, raw_key) = create_tenant_and_key(&state, "tf-del").await;
    let sub_id = create_subscription(&state, tenant_id).await;

    let create_body = serde_json::json!({
        "subscription_id": sub_id,
        "rules": [{ "source_path": "$.a", "dest_path": "$.b" }]
    });
    let create_req = Request::builder()
        .uri("/transformations")
        .method("POST")
        .header(header::AUTHORIZATION, format!("Bearer {raw_key}"))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::to_string(&create_body).unwrap()))
        .unwrap();
    let create_res = app.clone().oneshot(create_req).await.unwrap();
    let created: Value = serde_json::from_slice(&axum::body::to_bytes(create_res.into_body(), usize::MAX).await.unwrap()).unwrap();
    let tf_id = created["id"].as_str().unwrap().to_string();

    // Delete
    let del_req = Request::builder()
        .uri(format!("/transformations/{tf_id}"))
        .method("DELETE")
        .header(header::AUTHORIZATION, format!("Bearer {raw_key}"))
        .body(Body::empty())
        .unwrap();

    let del_res = app.clone().oneshot(del_req).await.unwrap();
    assert_eq!(del_res.status(), StatusCode::NO_CONTENT);

    // Subscription still exists (preserved)
    let sub_check = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM subscriptions WHERE id = $1)"
    )
    .bind(sub_id)
    .fetch_one(&_pool)
    .await
    .unwrap();
    assert!(sub_check, "Subscription must still exist after transformation deletion");

    // Second delete returns 404
    let del2_req = Request::builder()
        .uri(format!("/transformations/{tf_id}"))
        .method("DELETE")
        .header(header::AUTHORIZATION, format!("Bearer {raw_key}"))
        .body(Body::empty())
        .unwrap();

    let del2_res = app.clone().oneshot(del2_req).await.unwrap();
    assert_eq!(del2_res.status(), StatusCode::NOT_FOUND);
}

// 10. Cross-tenant delete (#61)
#[tokio::test]
async fn test_delete_transformation_cross_tenant() {
    let (app, state, _pool) = setup_test_app().await;
    let (tenant_a_id, key_a) = create_tenant_and_key(&state, "tf-del-ct-a").await;
    let (_tenant_b_id, key_b) = create_tenant_and_key(&state, "tf-del-ct-b").await;
    let sub_id = create_subscription(&state, tenant_a_id).await;

    let create_body = serde_json::json!({
        "subscription_id": sub_id,
        "rules": [{ "source_path": "$.a", "dest_path": "$.b" }]
    });
    let create_req = Request::builder()
        .uri("/transformations")
        .method("POST")
        .header(header::AUTHORIZATION, format!("Bearer {key_a}"))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::to_string(&create_body).unwrap()))
        .unwrap();
    let create_res = app.clone().oneshot(create_req).await.unwrap();
    let created: Value = serde_json::from_slice(&axum::body::to_bytes(create_res.into_body(), usize::MAX).await.unwrap()).unwrap();
    let tf_id = created["id"].as_str().unwrap().to_string();

    let del_req = Request::builder()
        .uri(format!("/transformations/{tf_id}"))
        .method("DELETE")
        .header(header::AUTHORIZATION, format!("Bearer {key_b}"))
        .body(Body::empty())
        .unwrap();

    let del_res = app.oneshot(del_req).await.unwrap();
    assert_eq!(del_res.status(), StatusCode::NOT_FOUND);
}

// 11. Empty list (Endpoint #58)
#[tokio::test]
async fn test_list_transformations_empty() {
    let (app, state, _pool) = setup_test_app().await;
    let (tenant_id, raw_key) = create_tenant_and_key(&state, "tf-empty-list").await;
    let sub_id = create_subscription(&state, tenant_id).await;

    let req = Request::builder()
        .uri(format!("/transformations?subscription_id={sub_id}"))
        .method("GET")
        .header(header::AUTHORIZATION, format!("Bearer {raw_key}"))
        .body(Body::empty())
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let list: Value = serde_json::from_slice(&axum::body::to_bytes(res.into_body(), usize::MAX).await.unwrap()).unwrap();
    assert_eq!(list.as_array().unwrap().len(), 0);
}
