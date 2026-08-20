use std::sync::Arc;
use sqlx::PgPool;
use uuid::Uuid;

use data::repositories::DeliveryRepository;
use relay_core::error::CoreError;
use crate::dto::{DeliveryAttemptView, DeliveryView};

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

    pub async fn retry_delivery(
        &self,
        tenant_id: Uuid,
        delivery_id: Uuid,
    ) -> Result<DeliveryView, CoreError> {
        let repo = DeliveryRepository::new(&self.pool);
        let delivery = repo
            .find_by_tenant_and_id(tenant_id, delivery_id)
            .await?
            .ok_or_else(|| CoreError::NotFound(format!("Delivery '{delivery_id}' not found")))?;

        repo.update_status(delivery.id, "pending", 0, Some(chrono::Utc::now())).await?;

        self.get_delivery(tenant_id, delivery_id)
            .await?
            .ok_or_else(|| CoreError::NotFound(format!("Delivery '{delivery_id}' not found")))
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

    pub async fn replay_dlq(&self, tenant_id: Uuid, delivery_id: Uuid) -> Result<(), CoreError> {
        self.retry_delivery(tenant_id, delivery_id).await?;
        Ok(())
    }

    pub async fn attempt_delivery(&self, _delivery_id: Uuid) -> Result<DeliveryAttemptView, CoreError> {
        todo!()
    }
}
