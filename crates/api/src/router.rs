use axum::{
    routing::{get, post},
    Router,
};
use tower_http::trace::TraceLayer;

use crate::routes::{
    api_keys, audit_logs, auth, deliveries, destinations, dlq, events, health, hooks, sources, stats,
    subscriptions, tenants, transformations,
};
use crate::state::AppState;

pub fn create_router(state: AppState) -> Router {
    Router::new()
        // Webhook Ingestion Hook
        .route("/hooks/:slug", post(hooks::receive_webhook))
        // Health & Metrics
        .route("/health/liveness", get(health::liveness))
        .route("/health/readiness", get(health::readiness))
        .route("/health", get(health::health_check))
        .route("/v1/health", get(health::health_check))
        .route("/api/v1/health", get(health::health_check))
        .route("/metrics", get(health::metrics))
        .route("/v1/metrics", get(health::metrics))
        .route("/api/v1/metrics", get(health::metrics))
        // Auth API
        .route("/api/v1/auth/login", post(auth::login))
        .route("/api/v1/auth/refresh", post(auth::refresh_token))
        .route("/api/v1/auth/me", get(auth::me))
        // Tenants API
        .route("/tenants/:id/usage", get(tenants::get_tenant_usage))
        .route("/v1/tenants/:id/usage", get(tenants::get_tenant_usage))
        .route("/api/v1/tenants", get(tenants::list_tenants).post(tenants::create_tenant))
        .route(
            "/api/v1/tenants/:id",
            get(tenants::get_tenant)
                .put(tenants::update_tenant)
                .delete(tenants::delete_tenant),
        )
        .route("/api/v1/tenants/:id/usage", get(tenants::get_tenant_usage))
        // API Keys API
        .route("/v1/api-keys", get(api_keys::list_api_keys).post(api_keys::create_api_key))
        .route(
            "/v1/api-keys/:id",
            get(api_keys::get_api_key).delete(api_keys::revoke_api_key),
        )
        .route("/api/v1/api-keys", get(api_keys::list_api_keys).post(api_keys::create_api_key))
        .route(
            "/api/v1/api-keys/:id",
            get(api_keys::get_api_key).delete(api_keys::revoke_api_key),
        )
        // Sources API
        .route("/sources", get(sources::list_sources).post(sources::create_source))
        .route(
            "/sources/:id",
            get(sources::get_source)
                .put(sources::update_source)
                .patch(sources::update_source)
                .delete(sources::delete_source),
        )
        .route("/sources/:id/rotate-secret", post(sources::rotate_source_secret))
        .route("/sources/:id/verification-log", get(sources::get_source_verification_log))
        .route("/v1/sources", get(sources::list_sources).post(sources::create_source))
        .route(
            "/v1/sources/:id",
            get(sources::get_source)
                .put(sources::update_source)
                .patch(sources::update_source)
                .delete(sources::delete_source),
        )
        .route("/v1/sources/:id/rotate-secret", post(sources::rotate_source_secret))
        .route("/v1/sources/:id/verification-log", get(sources::get_source_verification_log))
        .route("/api/v1/sources", get(sources::list_sources).post(sources::create_source))
        .route(
            "/api/v1/sources/:id",
            get(sources::get_source)
                .put(sources::update_source)
                .patch(sources::update_source)
                .delete(sources::delete_source),
        )
        .route("/api/v1/sources/:id/rotate-secret", post(sources::rotate_source_secret))
        .route("/api/v1/sources/:id/verification-log", get(sources::get_source_verification_log))
        // Destinations API
        .route(
            "/destinations",
            get(destinations::list_destinations).post(destinations::create_destination),
        )
        .route(
            "/destinations/:id",
            get(destinations::get_destination)
                .put(destinations::update_destination)
                .patch(destinations::update_destination)
                .delete(destinations::delete_destination),
        )
        .route("/destinations/:id/pause", post(destinations::pause_destination))
        .route("/destinations/:id/resume", post(destinations::resume_destination))
        .route("/destinations/:id/test", post(destinations::test_destination))
        .route("/destinations/:id/health", get(destinations::get_destination_health))
        .route(
            "/v1/destinations",
            get(destinations::list_destinations).post(destinations::create_destination),
        )
        .route(
            "/v1/destinations/:id",
            get(destinations::get_destination)
                .put(destinations::update_destination)
                .patch(destinations::update_destination)
                .delete(destinations::delete_destination),
        )
        .route("/v1/destinations/:id/pause", post(destinations::pause_destination))
        .route("/v1/destinations/:id/resume", post(destinations::resume_destination))
        .route("/v1/destinations/:id/test", post(destinations::test_destination))
        .route("/v1/destinations/:id/health", get(destinations::get_destination_health))
        .route(
            "/api/v1/destinations",
            get(destinations::list_destinations).post(destinations::create_destination),
        )
        .route(
            "/api/v1/destinations/:id",
            get(destinations::get_destination)
                .put(destinations::update_destination)
                .patch(destinations::update_destination)
                .delete(destinations::delete_destination),
        )
        .route("/api/v1/destinations/:id/pause", post(destinations::pause_destination))
        .route("/api/v1/destinations/:id/resume", post(destinations::resume_destination))
        .route("/api/v1/destinations/:id/test", post(destinations::test_destination))
        .route("/api/v1/destinations/:id/health", get(destinations::get_destination_health))
        // Subscriptions API
        .route(
            "/subscriptions",
            get(subscriptions::list_subscriptions).post(subscriptions::create_subscription),
        )
        .route(
            "/subscriptions/:id",
            get(subscriptions::get_subscription)
                .put(subscriptions::update_subscription)
                .patch(subscriptions::update_subscription)
                .delete(subscriptions::delete_subscription),
        )
        .route("/subscriptions/:id/pause", post(subscriptions::pause_subscription))
        .route("/subscriptions/:id/resume", post(subscriptions::resume_subscription))
        .route(
            "/v1/subscriptions",
            get(subscriptions::list_subscriptions).post(subscriptions::create_subscription),
        )
        .route(
            "/v1/subscriptions/:id",
            get(subscriptions::get_subscription)
                .put(subscriptions::update_subscription)
                .patch(subscriptions::update_subscription)
                .delete(subscriptions::delete_subscription),
        )
        .route("/v1/subscriptions/:id/pause", post(subscriptions::pause_subscription))
        .route("/v1/subscriptions/:id/resume", post(subscriptions::resume_subscription))
        .route(
            "/api/v1/subscriptions",
            get(subscriptions::list_subscriptions).post(subscriptions::create_subscription),
        )
        .route(
            "/api/v1/subscriptions/:id",
            get(subscriptions::get_subscription)
                .put(subscriptions::update_subscription)
                .patch(subscriptions::update_subscription)
                .delete(subscriptions::delete_subscription),
        )
        .route("/api/v1/subscriptions/:id/pause", post(subscriptions::pause_subscription))
        .route("/api/v1/subscriptions/:id/resume", post(subscriptions::resume_subscription))
        // Events API
        .route("/v1/events", get(events::list_events).post(events::create_event))
        .route("/v1/events/:id", get(events::get_event))
        .route("/v1/events/:id/deliveries", get(events::get_event_deliveries))
        .route("/v1/events/:id/retry", post(events::retry_event))
        .route("/api/v1/events", get(events::list_events).post(events::create_event))
        .route("/api/v1/events/:id", get(events::get_event))
        .route("/api/v1/events/:id/deliveries", get(events::get_event_deliveries))
        .route("/api/v1/events/:id/retry", post(events::retry_event))
        // Deliveries API
        .route("/v1/deliveries", get(deliveries::list_deliveries))
        .route("/v1/deliveries/:id", get(deliveries::get_delivery))
        .route("/v1/deliveries/:id/attempts", get(deliveries::list_delivery_attempts))
        .route("/v1/deliveries/:id/retry", post(deliveries::retry_delivery))
        .route("/api/v1/deliveries", get(deliveries::list_deliveries))
        .route("/api/v1/deliveries/:id", get(deliveries::get_delivery))
        .route("/api/v1/deliveries/:id/attempts", get(deliveries::list_delivery_attempts))
        .route("/api/v1/deliveries/:id/retry", post(deliveries::retry_delivery))
        // Audit Logs API
        .route("/v1/audit-logs", get(audit_logs::list_audit_logs))
        .route("/api/v1/audit-logs", get(audit_logs::list_audit_logs))
        // DLQ API
        .route("/api/v1/dlq", get(dlq::list_dlq))
        .route("/api/v1/dlq/retry-all", post(dlq::retry_all_dlq))
        .route("/api/v1/dlq/:id", get(dlq::get_dlq_item).delete(dlq::purge_dlq_item))
        .route("/api/v1/dlq/:id/retry", post(dlq::retry_dlq_item))
        // Transformations API
        .route("/api/v1/transformations/test", post(transformations::test_transformation))
        // Stats API
        .route("/api/v1/stats", get(stats::get_tenant_stats))
        .route("/api/v1/stats/system", get(stats::get_system_stats))
        // Middleware layers
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
