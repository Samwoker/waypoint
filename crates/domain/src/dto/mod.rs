use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// --- Source DTOs ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSourceInput {
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    #[serde(default = "default_provider")]
    pub provider: String,
    #[serde(default = "default_verification_type")]
    pub verification_type: String,
    pub secret: Option<String>,
}

fn default_provider() -> String {
    "generic".to_string()
}

fn default_verification_type() -> String {
    "none".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateSourceInput {
    pub name: Option<String>,
    pub description: Option<String>,
    pub is_active: Option<bool>,
    pub timestamp_tolerance_secs: Option<i32>,
    pub secret: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceView {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub name: String,
    pub slug: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub provider: String,
    pub verification_type: String,
    pub is_active: bool,
    pub has_secret: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp_tolerance_secs: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secret: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RotateSecretResponse {
    pub source_id: Uuid,
    pub secret: String,
    pub warning: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct SourceVerificationLogEntry {
    pub received_at: DateTime<Utc>,
    pub signature_valid: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_event_id: Option<String>,
}

// --- Destination DTOs ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateDestinationInput {
    pub name: String,
    pub url: String,
    pub description: Option<String>,
    pub rate_limit_rps: Option<i32>,
    pub timeout_ms: Option<i32>,
    pub max_retries: Option<i32>,
    pub headers: Option<serde_json::Value>,
    pub secret: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateDestinationInput {
    pub name: Option<String>,
    pub url: Option<String>,
    pub description: Option<String>,
    pub rate_limit_rps: Option<i32>,
    pub timeout_ms: Option<i32>,
    pub max_retries: Option<i32>,
    pub retry_backoff_strategy: Option<String>,
    pub is_active: Option<bool>,
    pub secret: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DestinationView {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub name: String,
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub status: String,
    pub is_active: bool,
    pub consecutive_failures: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub circuit_opened_at: Option<DateTime<Utc>>,
    pub max_retries: i32,
    pub timeout_ms: i32,
    pub retry_backoff_strategy: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rate_limit_rps: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secret: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestDestinationResponse {
    pub success: bool,
    pub http_status: Option<i32>,
    pub latency_ms: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DestinationHealthView {
    pub status: String,
    pub consecutive_failures: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub circuit_opened_at: Option<DateTime<Utc>>,
    pub success_rate: f64,
    pub total_attempts: i64,
    pub successful_attempts: i64,
}

// --- Subscription DTOs ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSubscriptionInput {
    pub source_id: Uuid,
    pub destination_id: Uuid,
    #[serde(alias = "event_type_filter")]
    pub event_types: Vec<String>,
    pub filter_rules: Option<serde_json::Value>,
    pub transformation_template: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateSubscriptionInput {
    #[serde(alias = "event_type_filter")]
    pub event_types: Option<Vec<String>>,
    pub filter_rules: Option<serde_json::Value>,
    pub transformation_template: Option<String>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionView {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub source_id: Uuid,
    pub destination_id: Uuid,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub destination_name: Option<String>,
    pub event_types: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter_rules: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transformation_template: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// --- Tenant & Auth DTOs ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTenantInput {
    pub name: String,
    pub slug: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateTenantInput {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantView {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateApiKeyInput {
    pub name: String,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKeyCreatedView {
    pub id: Uuid,
    pub name: String,
    pub raw_key: String,
    pub key_prefix: String,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKeyView {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub name: String,
    pub key_prefix: String,
    pub expires_at: Option<DateTime<Utc>>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginInput {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthTokenView {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: i64,
}

// --- Event & Delivery DTOs ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateEventInput {
    pub source_id: Option<Uuid>,
    pub event_type: String,
    pub payload: serde_json::Value,
    pub idempotency_key: Option<String>,
    pub headers: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngestWebhookInput {
    pub event_type: String,
    pub payload: serde_json::Value,
    pub idempotency_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventView {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub source_id: Uuid,
    pub event_type: String,
    pub idempotency_key: Option<String>,
    pub headers: serde_json::Value,
    pub payload: serde_json::Value,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventMetadataView {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub source_id: Uuid,
    pub event_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub idempotency_key: Option<String>,
    pub status: String,
    pub received_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedEventsView {
    pub events: Vec<EventMetadataView>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<String>,
    pub has_more: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeliverySummary {
    pub total: i64,
    pub delivered: i64,
    pub failed: i64,
    pub pending: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventDetailView {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub source_id: Uuid,
    pub event_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub idempotency_key: Option<String>,
    pub status: String,
    pub delivery_summary: DeliverySummary,
    pub received_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawEventPayloadView {
    pub event_id: Uuid,
    pub headers: serde_json::Value,
    pub payload: String,
    pub is_binary: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventDeliveryView {
    pub id: Uuid,
    pub destination_id: Uuid,
    pub destination_name: String,
    pub status: String,
    pub attempt_count: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_attempt_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delivered_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeliveryView {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub event_id: Uuid,
    pub subscription_id: Uuid,
    pub destination_id: Uuid,
    pub status: String,
    pub attempt_count: i32,
    pub max_attempts: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_retry_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedDeliveriesView {
    pub deliveries: Vec<DeliveryView>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<String>,
    pub has_more: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeliveryDetailAttemptView {
    pub id: Uuid,
    pub attempt_number: i32,
    #[serde(alias = "status_code")]
    pub http_status: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_body_snippet: Option<String>,
    #[serde(alias = "duration_ms")]
    pub latency_ms: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeliveryDetailView {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub event_id: Uuid,
    pub subscription_id: Uuid,
    pub destination_id: Uuid,
    pub status: String,
    pub attempt_count: i32,
    pub max_attempts: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_retry_at: Option<DateTime<Utc>>,
    pub attempts: Vec<DeliveryDetailAttemptView>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ReplayDeliveryInput {
    #[serde(default)]
    pub reset_attempt_count: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayEventResult {
    pub event_id: Uuid,
    pub deliveries_created: usize,
    pub deliveries_reset: usize,
    pub total_deliveries: usize,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ReplayBatchInput {
    pub destination_id: Option<Uuid>,
    pub source_id: Option<Uuid>,
    pub from: Option<DateTime<Utc>>,
    pub to: Option<DateTime<Utc>>,
    pub status_filter: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayBatchResult {
    pub replayed_count: i64,
    pub has_more: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeliveryAttemptView {
    pub id: Uuid,
    pub delivery_id: Uuid,
    pub attempt_number: i32,
    pub status_code: Option<i32>,
    pub request_headers: Option<serde_json::Value>,
    pub request_body: Option<String>,
    pub response_headers: Option<serde_json::Value>,
    pub response_body: Option<String>,
    pub error_message: Option<String>,
    pub execution_duration_ms: Option<i32>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatsSummaryView {
    pub total_events: i64,
    pub successful_deliveries: i64,
    pub failed_deliveries: i64,
    pub pending_deliveries: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestTransformationInput {
    pub template: String,
    pub payload: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestTransformationOutput {
    pub transformed_payload: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogView {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub user_id: Option<Uuid>,
    pub action: String,
    pub resource_type: Option<String>,
    pub resource_id: Option<Uuid>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyEventCount {
    pub date: String,
    pub count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantUsageView {
    pub tenant_id: Uuid,
    pub period: String,
    pub total_events: i64,
    pub total_delivery_attempts: i64,
    pub daily_events: Vec<DailyEventCount>,
}
