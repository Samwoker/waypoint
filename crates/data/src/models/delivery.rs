use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, FromRow)]
pub struct Delivery {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub event_id: Uuid,
    pub subscription_id: Uuid,
    pub destination_id: Uuid,
    pub status: String,
    pub attempt_count: i32,
    pub max_attempts: i32,
    pub next_retry_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
