use std::sync::Arc;
use sqlx::PgPool;
use uuid::Uuid;

use data::repositories::{DeliveryRepository, DestinationRepository, EventRepository, SubscriptionRepository};
use relay_core::error::CoreError;
use data::queue::RedisQueue;
use crate::dto::ReplayEventResult;

#[derive(Clone)]
pub struct FanoutService {
    pub pool: Arc<PgPool>,
    pub queue: Arc<tokio::sync::Mutex<RedisQueue>>,
}

impl FanoutService {
    pub fn new(pool: Arc<PgPool>, queue: Arc<tokio::sync::Mutex<RedisQueue>>) -> Self {
        Self { pool, queue }
    }

    pub async fn fan_out_event(
        &self,
        tenant_id: Uuid,
        event_id: Uuid,
    ) -> Result<ReplayEventResult, CoreError> {
        let event_repo = EventRepository::new(&self.pool);
        let event = event_repo
            .find_by_tenant_and_id(tenant_id, event_id)
            .await?
            .ok_or_else(|| CoreError::NotFound(format!("Event '{event_id}' not found")))?;

        let sub_repo = SubscriptionRepository::new(&self.pool);
        let subscriptions = sub_repo.list_by_source(event.source_id).await?;

        let delivery_repo = DeliveryRepository::new(&self.pool);
        let dest_repo = DestinationRepository::new(&self.pool);

        let mut deliveries_created = 0;
        let mut deliveries_reset = 0;

        for sub in subscriptions {
            // Strictly enforce tenant isolation
            if sub.tenant_id != tenant_id {
                continue;
            }

            // Match event type:
            // 1. If sub.event_types is empty or contains "*", match all events
            // 2. Exact match
            // 3. Prefix wildcard match, e.g. "payment.*"
            let matches = if sub.event_types.is_empty() {
                true
            } else {
                sub.event_types.iter().any(|pattern| {
                    if pattern == "*" || pattern == &event.event_type {
                        true
                    } else if let Some(prefix) = pattern.strip_suffix(".*") {
                        event.event_type.starts_with(prefix)
                    } else if let Some(prefix) = pattern.strip_suffix('*') {
                        event.event_type.starts_with(prefix)
                    } else {
                        false
                    }
                })
            };

            if !matches {
                continue;
            }

            // Check if delivery already exists for this event and destination
            if let Some(existing) = delivery_repo
                .find_by_event_and_destination(tenant_id, event.id, sub.destination_id)
                .await?
            {
                delivery_repo
                    .update_status(existing.id, "pending", 0, Some(chrono::Utc::now()))
                    .await?;
                deliveries_reset += 1;
            } else {
                let max_attempts = dest_repo
                    .find_by_tenant_and_id(tenant_id, sub.destination_id)
                    .await?
                    .map(|d| d.max_retries)
                    .unwrap_or(5);

                delivery_repo
                    .create(
                        tenant_id,
                        event.id,
                        sub.id,
                        sub.destination_id,
                        max_attempts,
                    )
                    .await?;
                deliveries_created += 1;
            }
        }

        Ok(ReplayEventResult {
            event_id,
            deliveries_created,
            deliveries_reset,
            total_deliveries: deliveries_created + deliveries_reset,
        })
    }
}
