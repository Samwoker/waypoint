use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, FromRow)]
pub struct Event {
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, FromRow)]
pub struct VerificationLogRecord {
    pub received_at: DateTime<Utc>,
    pub signature_valid: bool,
    pub external_event_id: Option<String>,
}
