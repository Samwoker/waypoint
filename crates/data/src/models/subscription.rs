use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, FromRow)]
pub struct Subscription {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub source_id: Uuid,
    pub destination_id: Uuid,
    pub event_types: Vec<String>,
    pub filter_rules: Option<serde_json::Value>,
    pub transformation_template: Option<String>,
    pub is_active: bool,
    pub source_name: Option<String>,
    pub destination_name: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
