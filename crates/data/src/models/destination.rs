use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, FromRow)]
pub struct Destination {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub name: String,
    pub url: String,
    pub description: Option<String>,
    pub status: String,
    pub timeout_ms: i32,
    pub max_retries: i32,
    pub is_active: bool,
    pub consecutive_failures: i32,
    pub circuit_opened_at: Option<DateTime<Utc>>,
    pub retry_backoff_strategy: String,
    pub rate_limit_rps: Option<i32>,
    pub secret_encrypted: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, FromRow)]
pub struct DestinationHealthStats {
    pub status: String,
    pub consecutive_failures: i32,
    pub circuit_opened_at: Option<DateTime<Utc>>,
    pub successes: i64,
    pub total: i64,
}
