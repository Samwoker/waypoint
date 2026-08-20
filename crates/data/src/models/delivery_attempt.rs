use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, FromRow)]
pub struct DeliveryAttempt {
    pub id: Uuid,
    pub delivery_id: Uuid,
    pub attempt_number: i32,
    #[sqlx(rename = "response_status")]
    pub status_code: Option<i32>,
    pub request_headers: Option<serde_json::Value>,
    pub request_body: Option<serde_json::Value>,
    pub response_headers: Option<serde_json::Value>,
    pub response_body: Option<String>,
    pub error_message: Option<String>,
    #[sqlx(rename = "duration_ms")]
    pub execution_duration_ms: Option<i32>,
    pub created_at: DateTime<Utc>,
}
