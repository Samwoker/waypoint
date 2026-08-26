use std::sync::Arc;
use base64::Engine;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use data::repositories::DeliveryRepository;
use relay_core::error::CoreError;
use crate::dto::{
    DeliveryAttemptView, DeliveryDetailAttemptView, DeliveryDetailView, DeliveryView,
    PaginatedDeliveriesView, ReplayBatchInput, ReplayBatchResult,
};

#[derive(Clone)]
pub struct DeliveryService {
    pub pool: Arc<PgPool>,
    pub http_client: reqwest::Client,
}

impl DeliveryService {
    pub fn new(pool: Arc<PgPool>, http_client: reqwest::Client) -> Self {
        Self { pool, http_client }
    }

    pub async fn get_delivery(
        &self,
        tenant_id: Uuid,
        delivery_id: Uuid,
    ) -> Result<Option<DeliveryView>, CoreError> {
        let repo = DeliveryRepository::new(&self.pool);
        let delivery = repo.find_by_tenant_and_id(tenant_id, delivery_id).await?;

        Ok(delivery.map(|d| DeliveryView {
            id: d.id,
            tenant_id: d.tenant_id,
            event_id: d.event_id,
            subscription_id: d.subscription_id,
            destination_id: d.destination_id,
            status: d.status,
            attempt_count: d.attempt_count,
            max_attempts: d.max_attempts,
            next_retry_at: d.next_retry_at,
            created_at: d.created_at,
            updated_at: d.updated_at,
        }))
    }

    pub async fn get_delivery_detail(
        &self,
        tenant_id: Uuid,
        delivery_id: Uuid,
    ) -> Result<DeliveryDetailView, CoreError> {
        let repo = DeliveryRepository::new(&self.pool);
        let delivery = repo
            .find_by_tenant_and_id(tenant_id, delivery_id)
            .await?
            .ok_or_else(|| CoreError::NotFound(format!("Delivery '{delivery_id}' not found")))?;

        let attempts = repo.list_attempts_by_delivery_id(delivery.id).await?;

        let attempt_views = attempts
            .into_iter()
            .map(|a| {
                let snippet = a.response_body.map(|body| {
                    if body.len() > 500 {
                        format!("{}...", &body[..500])
                    } else {
                        body
                    }
                });

                DeliveryDetailAttemptView {
                    id: a.id,
                    attempt_number: a.attempt_number,
                    http_status: a.status_code,
                    response_body_snippet: snippet,
                    latency_ms: a.execution_duration_ms,
                    error_message: a.error_message,
                    created_at: a.created_at,
                }
            })
            .collect();

        Ok(DeliveryDetailView {
            id: delivery.id,
            tenant_id: delivery.tenant_id,
            event_id: delivery.event_id,
            subscription_id: delivery.subscription_id,
            destination_id: delivery.destination_id,
            status: delivery.status,
            attempt_count: delivery.attempt_count,
            max_attempts: delivery.max_attempts,
            next_retry_at: delivery.next_retry_at,
            attempts: attempt_views,
            created_at: delivery.created_at,
            updated_at: delivery.updated_at,
        })
    }

    pub async fn list_deliveries(
        &self,
        tenant_id: Uuid,
        status: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<DeliveryView>, CoreError> {
        let limit = if limit <= 0 || limit > 100 { 20 } else { limit };
        let offset = if offset < 0 { 0 } else { offset };

        let repo = DeliveryRepository::new(&self.pool);
        let deliveries = repo.list_by_tenant(tenant_id, status, limit, offset).await?;

        Ok(deliveries
            .into_iter()
            .map(|d| DeliveryView {
                id: d.id,
                tenant_id: d.tenant_id,
                event_id: d.event_id,
                subscription_id: d.subscription_id,
                destination_id: d.destination_id,
                status: d.status,
                attempt_count: d.attempt_count,
                max_attempts: d.max_attempts,
                next_retry_at: d.next_retry_at,
                created_at: d.created_at,
                updated_at: d.updated_at,
            })
            .collect())
    }

    pub async fn list_deliveries_paginated(
        &self,
        tenant_id: Uuid,
        destination_id: Option<Uuid>,
        status: Option<&str>,
        from: Option<DateTime<Utc>>,
        to: Option<DateTime<Utc>>,
        limit: i64,
        cursor: Option<&str>,
    ) -> Result<PaginatedDeliveriesView, CoreError> {
        let limit = if limit <= 0 || limit > 100 { 20 } else { limit };

        if let Some(st) = status {
            if !["pending", "running", "delivered", "failed", "dead_letter"].contains(&st) {
                return Err(CoreError::Validation(format!(
                    "Invalid status filter '{st}'. Valid values: pending, running, delivered, failed, dead_letter"
                )));
            }
        }

        let (cursor_created_at, cursor_id) = if let Some(c) = cursor {
            let decoded = base64::engine::general_purpose::URL_SAFE_NO_PAD
                .decode(c.trim())
                .or_else(|_| base64::engine::general_purpose::STANDARD.decode(c.trim()))
                .map_err(|_| CoreError::Validation("Invalid cursor encoding".to_string()))?;

            let decoded_str = String::from_utf8(decoded)
                .map_err(|_| CoreError::Validation("Invalid cursor text".to_string()))?;

            let parts: Vec<&str> = decoded_str.split('_').collect();
            if parts.len() < 2 {
                return Err(CoreError::Validation("Invalid cursor structure".to_string()));
            }

            let ts_str = parts[..parts.len() - 1].join("_");
            let id_str = parts[parts.len() - 1];

            let ts = DateTime::parse_from_rfc3339(&ts_str)
                .map_err(|_| CoreError::Validation("Invalid cursor timestamp".to_string()))?
                .with_timezone(&Utc);

            let id = Uuid::parse_str(id_str)
                .map_err(|_| CoreError::Validation("Invalid cursor UUID".to_string()))?;

            (Some(ts), Some(id))
        } else {
            (None, None)
        };

        let repo = DeliveryRepository::new(&self.pool);
        let mut deliveries = repo
            .list_paginated(
                tenant_id,
                destination_id,
                status,
                from,
                to,
                cursor_created_at,
                cursor_id,
                limit + 1,
            )
            .await?;

        let has_more = deliveries.len() > limit as usize;
        if has_more {
            deliveries.truncate(limit as usize);
        }

        let next_cursor = if has_more {
            deliveries.last().map(|last| {
                let payload = format!("{}_{}", last.created_at.to_rfc3339(), last.id);
                base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(payload)
            })
        } else {
            None
        };

        let delivery_views = deliveries
            .into_iter()
            .map(|d| DeliveryView {
                id: d.id,
                tenant_id: d.tenant_id,
                event_id: d.event_id,
                subscription_id: d.subscription_id,
                destination_id: d.destination_id,
                status: d.status,
                attempt_count: d.attempt_count,
                max_attempts: d.max_attempts,
                next_retry_at: d.next_retry_at,
                created_at: d.created_at,
                updated_at: d.updated_at,
            })
            .collect();

        Ok(PaginatedDeliveriesView {
            deliveries: delivery_views,
            next_cursor,
            has_more,
        })
    }

    pub async fn replay_delivery(
        &self,
        tenant_id: Uuid,
        delivery_id: Uuid,
        reset_attempt_count: bool,
    ) -> Result<DeliveryView, CoreError> {
        let repo = DeliveryRepository::new(&self.pool);
        let delivery = repo.replay_delivery(tenant_id, delivery_id, reset_attempt_count).await?;

        Ok(DeliveryView {
            id: delivery.id,
            tenant_id: delivery.tenant_id,
            event_id: delivery.event_id,
            subscription_id: delivery.subscription_id,
            destination_id: delivery.destination_id,
            status: delivery.status,
            attempt_count: delivery.attempt_count,
            max_attempts: delivery.max_attempts,
            next_retry_at: delivery.next_retry_at,
            created_at: delivery.created_at,
            updated_at: delivery.updated_at,
        })
    }

    pub async fn replay_batch(
        &self,
        tenant_id: Uuid,
        input: ReplayBatchInput,
    ) -> Result<ReplayBatchResult, CoreError> {
        if let Some(ref st) = input.status_filter {
            if !["pending", "running", "delivered", "failed", "dead_letter"].contains(&st.as_str()) {
                return Err(CoreError::Validation(format!(
                    "Invalid status filter '{st}'. Valid values: pending, running, delivered, failed, dead_letter"
                )));
            }
        }

        let limit = 1000;
        let repo = DeliveryRepository::new(&self.pool);
        let (replayed_count, has_more) = repo
            .replay_batch(
                tenant_id,
                input.destination_id,
                input.source_id,
                input.from,
                input.to,
                input.status_filter.as_deref(),
                limit,
            )
            .await?;

        Ok(ReplayBatchResult {
            replayed_count,
            has_more,
        })
    }

    pub async fn retry_delivery(
        &self,
        tenant_id: Uuid,
        delivery_id: Uuid,
    ) -> Result<DeliveryView, CoreError> {
        self.replay_delivery(tenant_id, delivery_id, false).await
    }

    pub async fn list_dlq(
        &self,
        tenant_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<DeliveryView>, CoreError> {
        let limit = if limit <= 0 || limit > 100 { 20 } else { limit };
        let offset = if offset < 0 { 0 } else { offset };

        let repo = DeliveryRepository::new(&self.pool);
        let deliveries = repo.list_dlq_by_tenant(tenant_id, limit, offset).await?;

        Ok(deliveries
            .into_iter()
            .map(|d| DeliveryView {
                id: d.id,
                tenant_id: d.tenant_id,
                event_id: d.event_id,
                subscription_id: d.subscription_id,
                destination_id: d.destination_id,
                status: d.status,
                attempt_count: d.attempt_count,
                max_attempts: d.max_attempts,
                next_retry_at: d.next_retry_at,
                created_at: d.created_at,
                updated_at: d.updated_at,
            })
            .collect())
    }

    pub async fn list_dlq_paginated(
        &self,
        tenant_id: Uuid,
        limit: i64,
        cursor: Option<&str>,
    ) -> Result<crate::dto::PaginatedDlqView, CoreError> {
        let limit = if limit <= 0 || limit > 100 { 20 } else { limit };

        let (cursor_created_at, cursor_id) = if let Some(c) = cursor {
            let decoded = base64::engine::general_purpose::URL_SAFE_NO_PAD
                .decode(c.trim())
                .or_else(|_| base64::engine::general_purpose::STANDARD.decode(c.trim()))
                .map_err(|_| CoreError::Validation("Invalid cursor encoding".to_string()))?;

            let decoded_str = String::from_utf8(decoded)
                .map_err(|_| CoreError::Validation("Invalid cursor text".to_string()))?;

            let parts: Vec<&str> = decoded_str.split('_').collect();
            if parts.len() < 2 {
                return Err(CoreError::Validation("Invalid cursor structure".to_string()));
            }

            let ts_str = parts[..parts.len() - 1].join("_");
            let id_str = parts[parts.len() - 1];

            let ts = DateTime::parse_from_rfc3339(&ts_str)
                .map_err(|_| CoreError::Validation("Invalid cursor timestamp".to_string()))?
                .with_timezone(&Utc);

            let id = Uuid::parse_str(id_str)
                .map_err(|_| CoreError::Validation("Invalid cursor UUID".to_string()))?;

            (Some(ts), Some(id))
        } else {
            (None, None)
        };

        let repo = DeliveryRepository::new(&self.pool);
        let mut items = repo
            .list_dlq_paginated(tenant_id, cursor_created_at, cursor_id, limit + 1)
            .await?;

        let has_more = items.len() > limit as usize;
        if has_more {
            items.truncate(limit as usize);
        }

        let next_cursor = if has_more {
            items.last().map(|last| {
                let payload = format!("{}_{}", last.created_at.to_rfc3339(), last.delivery_id);
                base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(payload)
            })
        } else {
            None
        };

        let item_views = items
            .into_iter()
            .map(|d| crate::dto::DlqItemView {
                delivery_id: d.delivery_id,
                tenant_id: d.tenant_id,
                event_id: d.event_id,
                event_type: d.event_type,
                destination_id: d.destination_id,
                destination_name: d.destination_name,
                destination_url: d.destination_url,
                status: d.status,
                attempt_count: d.attempt_count,
                max_attempts: d.max_attempts,
                last_error: d.last_error,
                created_at: d.created_at,
                updated_at: d.updated_at,
            })
            .collect();

        Ok(crate::dto::PaginatedDlqView {
            items: item_views,
            next_cursor,
            has_more,
        })
    }

    pub async fn requeue_dlq_item(
        &self,
        tenant_id: Uuid,
        event_id: Uuid,
        destination_id: Uuid,
    ) -> Result<DeliveryView, CoreError> {
        let event_repo = data::repositories::EventRepository::new(&self.pool);
        if event_repo.find_by_tenant_and_id(tenant_id, event_id).await?.is_none() {
            return Err(CoreError::NotFound(format!("Event '{event_id}' not found")));
        }

        let dest_repo = data::repositories::DestinationRepository::new(&self.pool);
        if dest_repo.find_by_tenant_and_id(tenant_id, destination_id).await?.is_none() {
            return Err(CoreError::NotFound(format!("Destination '{destination_id}' not found")));
        }

        let repo = DeliveryRepository::new(&self.pool);
        let delivery = repo.requeue_dlq(tenant_id, event_id, destination_id).await?;

        Ok(DeliveryView {
            id: delivery.id,
            tenant_id: delivery.tenant_id,
            event_id: delivery.event_id,
            subscription_id: delivery.subscription_id,
            destination_id: delivery.destination_id,
            status: delivery.status,
            attempt_count: delivery.attempt_count,
            max_attempts: delivery.max_attempts,
            next_retry_at: delivery.next_retry_at,
            created_at: delivery.created_at,
            updated_at: delivery.updated_at,
        })
    }

    pub async fn discard_dlq_item(
        &self,
        tenant_id: Uuid,
        event_id: Uuid,
        destination_id: Uuid,
    ) -> Result<(), CoreError> {
        let event_repo = data::repositories::EventRepository::new(&self.pool);
        if event_repo.find_by_tenant_and_id(tenant_id, event_id).await?.is_none() {
            return Err(CoreError::NotFound(format!("Event '{event_id}' not found")));
        }

        let dest_repo = data::repositories::DestinationRepository::new(&self.pool);
        if dest_repo.find_by_tenant_and_id(tenant_id, destination_id).await?.is_none() {
            return Err(CoreError::NotFound(format!("Destination '{destination_id}' not found")));
        }

        let repo = DeliveryRepository::new(&self.pool);
        repo.discard_dlq(tenant_id, event_id, destination_id).await
    }

    pub async fn discard_dlq_by_id(&self, tenant_id: Uuid, delivery_id: Uuid) -> Result<(), CoreError> {
        let repo = DeliveryRepository::new(&self.pool);
        repo.discard_dlq_by_id(tenant_id, delivery_id).await
    }

    pub async fn retry_all_dlq(&self, tenant_id: Uuid) -> Result<i64, CoreError> {
        let repo = DeliveryRepository::new(&self.pool);
        repo.retry_all_dlq(tenant_id).await
    }

    pub async fn replay_dlq(&self, tenant_id: Uuid, delivery_id: Uuid) -> Result<(), CoreError> {
        self.retry_delivery(tenant_id, delivery_id).await?;
        Ok(())
    }

    pub async fn attempt_delivery(&self, _delivery_id: Uuid) -> Result<DeliveryAttemptView, CoreError> {
        todo!()
    }
}
