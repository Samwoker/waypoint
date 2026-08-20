use std::sync::Arc;
use data::models::Subscription;
use data::repositories::{AuditLogRepository, DestinationRepository, SourceRepository, SubscriptionRepository};
use sqlx::PgPool;
use uuid::Uuid;

use relay_core::error::CoreError;
use crate::dto::{CreateSubscriptionInput, SubscriptionView, UpdateSubscriptionInput};

#[derive(Clone)]
pub struct SubscriptionService {
    pub pool: Arc<PgPool>,
}

impl SubscriptionService {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }

    pub async fn create_subscription(
        &self,
        tenant_id: Uuid,
        input: CreateSubscriptionInput,
    ) -> Result<SubscriptionView, CoreError> {
        if input.event_types.is_empty() {
            return Err(CoreError::Validation("At least one event type must be specified".to_string()));
        }

        let mut cleaned_event_types = Vec::with_capacity(input.event_types.len());
        for et in &input.event_types {
            let trimmed = et.trim();
            if trimmed.is_empty() {
                return Err(CoreError::Validation("Event type cannot be empty".to_string()));
            }
            cleaned_event_types.push(trimmed.to_string());
        }

        // Validate filter rules if provided
        if let Some(ref filter) = input.filter_rules {
            if !filter.is_object() {
                return Err(CoreError::Validation("Filter rules must be a JSON object".to_string()));
            }
        }

        // Validate source exists and belongs to the authenticated tenant
        let source_repo = SourceRepository::new(&self.pool);
        let source = match source_repo.find_by_tenant_and_id(tenant_id, input.source_id).await? {
            Some(s) => s,
            None => {
                if source_repo.find_by_id(input.source_id).await?.is_some() {
                    return Err(CoreError::NotFound(format!("Source '{}' not found", input.source_id)));
                } else {
                    return Err(CoreError::NotFound(format!("Source '{}' not found", input.source_id)));
                }
            }
        };

        // Validate destination exists and belongs to the authenticated tenant
        let dest_repo = DestinationRepository::new(&self.pool);
        let destination = match dest_repo.find_by_tenant_and_id(tenant_id, input.destination_id).await? {
            Some(d) => d,
            None => {
                if dest_repo.find_by_id(input.destination_id).await?.is_some() {
                    return Err(CoreError::NotFound(format!("Destination '{}' not found", input.destination_id)));
                } else {
                    return Err(CoreError::NotFound(format!("Destination '{}' not found", input.destination_id)));
                }
            }
        };

        let repo = SubscriptionRepository::new(&self.pool);

        // Check for duplicate binding
        if let Some(_existing) = repo.find_by_source_and_destination(tenant_id, input.source_id, input.destination_id).await? {
            return Err(CoreError::Conflict(format!(
                "Subscription binding for source '{}' and destination '{}' already exists",
                input.source_id, input.destination_id
            )));
        }

        let sub = repo
            .create(
                tenant_id,
                input.source_id,
                input.destination_id,
                cleaned_event_types,
                input.filter_rules,
                input.transformation_template.as_deref(),
                Some(source.name),
                Some(destination.name),
            )
            .await?;

        Ok(to_subscription_view(sub))
    }

    pub async fn get_subscription(&self, tenant_id: Uuid, id: Uuid) -> Result<Option<SubscriptionView>, CoreError> {
        let repo = SubscriptionRepository::new(&self.pool);
        let sub = repo.find_by_tenant_and_id(tenant_id, id).await?;

        Ok(sub.map(to_subscription_view))
    }

    pub async fn list_subscriptions(
        &self,
        tenant_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<SubscriptionView>, CoreError> {
        let limit = if limit <= 0 || limit > 100 { 20 } else { limit };
        let offset = if offset < 0 { 0 } else { offset };

        let repo = SubscriptionRepository::new(&self.pool);
        let subs = repo.list_by_tenant(tenant_id, limit, offset).await?;

        Ok(subs.into_iter().map(to_subscription_view).collect())
    }

    pub async fn update_subscription(
        &self,
        tenant_id: Uuid,
        id: Uuid,
        input: UpdateSubscriptionInput,
    ) -> Result<SubscriptionView, CoreError> {
        let event_types = input.event_types;
        if let Some(ref et_list) = event_types {
            if et_list.is_empty() {
                return Err(CoreError::Validation("At least one event type must be specified".to_string()));
            }
            for et in et_list {
                if et.trim().is_empty() {
                    return Err(CoreError::Validation("Event type cannot be empty".to_string()));
                }
            }
        }

        if let Some(ref filter) = input.filter_rules {
            if !filter.is_object() {
                return Err(CoreError::Validation("Filter rules must be a JSON object".to_string()));
            }
        }

        let repo = SubscriptionRepository::new(&self.pool);
        let sub = repo
            .update(
                tenant_id,
                id,
                event_types,
                input.filter_rules,
                input.transformation_template.as_deref(),
                input.is_active,
            )
            .await?;

        Ok(to_subscription_view(sub))
    }

    pub async fn delete_subscription(&self, tenant_id: Uuid, id: Uuid) -> Result<(), CoreError> {
        let repo = SubscriptionRepository::new(&self.pool);
        let existing = repo
            .find_by_tenant_and_id(tenant_id, id)
            .await?
            .ok_or_else(|| CoreError::NotFound(format!("Subscription '{id}' not found")))?;

        repo.delete(tenant_id, id).await?;

        let audit_repo = AuditLogRepository::new(&self.pool);
        let _ = audit_repo
            .create(
                tenant_id,
                None,
                "subscription.deleted",
                Some("subscription"),
                Some(id),
                serde_json::json!({
                    "subscription_id": id,
                    "source_id": existing.source_id,
                    "destination_id": existing.destination_id,
                }),
            )
            .await;

        Ok(())
    }
}

fn to_subscription_view(s: Subscription) -> SubscriptionView {
    SubscriptionView {
        id: s.id,
        tenant_id: s.tenant_id,
        source_id: s.source_id,
        destination_id: s.destination_id,
        source_name: s.source_name,
        destination_name: s.destination_name,
        event_types: s.event_types,
        filter_rules: s.filter_rules,
        transformation_template: s.transformation_template,
        is_active: s.is_active,
        created_at: s.created_at,
        updated_at: s.updated_at,
    }
}
