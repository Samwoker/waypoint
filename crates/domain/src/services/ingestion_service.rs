use std::sync::Arc;
use base64::Engine;
use chrono::{DateTime, Utc};
use data::repositories::{AuditLogRepository, EventRepository, SourceRepository};
use sqlx::PgPool;
use uuid::Uuid;

use relay_core::error::CoreError;
use data::queue::RedisQueue;
use crate::dto::{
    CreateEventInput, DeliverySummary, EventDeliveryView, EventDetailView, EventMetadataView,
    EventView, IngestWebhookInput, PaginatedEventsView, RawEventPayloadView,
};

#[derive(Clone)]
pub struct IngestionService {
    pub pool: Arc<PgPool>,
    pub queue: Arc<tokio::sync::Mutex<RedisQueue>>,
}

impl IngestionService {
    pub fn new(pool: Arc<PgPool>, queue: Arc<tokio::sync::Mutex<RedisQueue>>) -> Self {
        Self { pool, queue }
    }

    pub async fn create_event(
        &self,
        tenant_id: Uuid,
        input: CreateEventInput,
    ) -> Result<EventView, CoreError> {
        let event_type = input.event_type.trim();
        if event_type.is_empty() {
            return Err(CoreError::Validation("Event type cannot be empty".to_string()));
        }

        // Validate source
        let source_repo = SourceRepository::new(&self.pool);
        let source_id = if let Some(src_id) = input.source_id {
            match source_repo.find_by_tenant_and_id(tenant_id, src_id).await? {
                Some(s) => s.id,
                None => {
                    if source_repo.find_by_id(src_id).await?.is_some() {
                        return Err(CoreError::Forbidden("Source belongs to another tenant".to_string()));
                    } else {
                        return Err(CoreError::NotFound(format!("Source '{src_id}' not found")));
                    }
                }
            }
        } else {
            let sources = source_repo.list_by_tenant(tenant_id, 1, 0).await?;
            if let Some(first_source) = sources.first() {
                first_source.id
            } else {
                return Err(CoreError::Validation(
                    "No event source found for tenant. Please create a source or supply source_id.".to_string(),
                ));
            }
        };

        let idempotency_key = input.idempotency_key.as_deref().map(|k| k.trim()).filter(|k| !k.is_empty());
        let event_repo = EventRepository::new(&self.pool);

        // Check for existing event if idempotency key is provided
        if let Some(ikey) = idempotency_key {
            if let Some(existing) = event_repo.find_by_idempotency_key(tenant_id, ikey).await? {
                return Ok(EventView {
                    id: existing.id,
                    tenant_id: existing.tenant_id,
                    source_id: existing.source_id,
                    event_type: existing.event_type,
                    idempotency_key: existing.idempotency_key,
                    headers: existing.headers,
                    payload: existing.payload,
                    status: existing.status,
                    created_at: existing.created_at,
                });
            }
        }

        let headers = input.headers.unwrap_or_else(|| serde_json::json!({}));

        // Persist event in PostgreSQL
        let event = match event_repo
            .create(
                tenant_id,
                source_id,
                event_type,
                idempotency_key,
                headers,
                input.payload,
            )
            .await
        {
            Ok(ev) => ev,
            Err(CoreError::Conflict(_)) if idempotency_key.is_some() => {
                let existing = event_repo
                    .find_by_idempotency_key(tenant_id, idempotency_key.unwrap())
                    .await?
                    .ok_or_else(|| CoreError::Conflict("Duplicate idempotency key conflict".to_string()))?;
                return Ok(EventView {
                    id: existing.id,
                    tenant_id: existing.tenant_id,
                    source_id: existing.source_id,
                    event_type: existing.event_type,
                    idempotency_key: existing.idempotency_key,
                    headers: existing.headers,
                    payload: existing.payload,
                    status: existing.status,
                    created_at: existing.created_at,
                });
            }
            Err(e) => return Err(e),
        };

        // Publish event to Redis Queue for async delivery processing
        {
            let mut q = self.queue.lock().await;
            let event_id_str = event.id.to_string();
            let tenant_id_str = event.tenant_id.to_string();
            let source_id_str = event.source_id.to_string();

            let stream_payload = [
                ("event_id", event_id_str.as_str()),
                ("tenant_id", tenant_id_str.as_str()),
                ("source_id", source_id_str.as_str()),
                ("event_type", event.event_type.as_str()),
            ];

            q.push_event("events:incoming", &stream_payload).await?;
        }

        Ok(EventView {
            id: event.id,
            tenant_id: event.tenant_id,
            source_id: event.source_id,
            event_type: event.event_type,
            idempotency_key: event.idempotency_key,
            headers: event.headers,
            payload: event.payload,
            status: event.status,
            created_at: event.created_at,
        })
    }

    pub async fn get_event(&self, tenant_id: Uuid, id: Uuid) -> Result<Option<EventView>, CoreError> {
        let repo = EventRepository::new(&self.pool);
        let event = repo.find_by_tenant_and_id(tenant_id, id).await?;

        Ok(event.map(|e| EventView {
            id: e.id,
            tenant_id: e.tenant_id,
            source_id: e.source_id,
            event_type: e.event_type,
            idempotency_key: e.idempotency_key,
            headers: e.headers,
            payload: e.payload,
            status: e.status,
            created_at: e.created_at,
        }))
    }

    pub async fn list_events_paginated(
        &self,
        tenant_id: Uuid,
        source_id: Option<Uuid>,
        status: Option<&str>,
        from: Option<DateTime<Utc>>,
        to: Option<DateTime<Utc>>,
        limit: i64,
        cursor: Option<&str>,
    ) -> Result<PaginatedEventsView, CoreError> {
        let limit = if limit <= 0 || limit > 100 { 20 } else { limit };

        if let Some(st) = status {
            if !["delivered", "failed", "pending", "no_subscriptions"].contains(&st) {
                return Err(CoreError::Validation(format!(
                    "Invalid status filter '{st}'. Valid values are: delivered, failed, pending, no_subscriptions"
                )));
            }
        }

        let (cursor_received_at, cursor_id) = if let Some(c) = cursor {
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

        let repo = EventRepository::new(&self.pool);
        // Query limit + 1 to check if there is a next page
        let mut events = repo
            .list_paginated_with_status(
                tenant_id,
                source_id,
                status,
                from,
                to,
                cursor_received_at,
                cursor_id,
                limit + 1,
            )
            .await?;

        let has_more = events.len() > limit as usize;
        if has_more {
            events.truncate(limit as usize);
        }

        let next_cursor = if has_more {
            events.last().map(|last| {
                let payload = format!("{}_{}", last.received_at.to_rfc3339(), last.id);
                base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(payload)
            })
        } else {
            None
        };

        let event_views = events
            .into_iter()
            .map(|e| EventMetadataView {
                id: e.id,
                tenant_id: e.tenant_id,
                source_id: e.source_id,
                event_type: e.event_type,
                idempotency_key: e.idempotency_key,
                status: e.status,
                received_at: e.received_at,
                created_at: e.created_at,
            })
            .collect();

        Ok(PaginatedEventsView {
            events: event_views,
            next_cursor,
            has_more,
        })
    }

    pub async fn get_event_detail(
        &self,
        tenant_id: Uuid,
        id: Uuid,
    ) -> Result<EventDetailView, CoreError> {
        let repo = EventRepository::new(&self.pool);
        let event = repo
            .find_by_tenant_and_id(tenant_id, id)
            .await?
            .ok_or_else(|| CoreError::NotFound(format!("Event '{id}' not found")))?;

        let summary = repo.get_delivery_summary(tenant_id, id).await?;
        let computed_status = repo.get_computed_status(tenant_id, id).await?;

        Ok(EventDetailView {
            id: event.id,
            tenant_id: event.tenant_id,
            source_id: event.source_id,
            event_type: event.event_type,
            idempotency_key: event.idempotency_key,
            status: computed_status,
            delivery_summary: DeliverySummary {
                total: summary.total,
                delivered: summary.delivered,
                failed: summary.failed,
                pending: summary.pending,
            },
            received_at: event.received_at,
            created_at: event.created_at,
        })
    }

    pub async fn get_event_raw(
        &self,
        tenant_id: Uuid,
        id: Uuid,
        scope: &str,
    ) -> Result<RawEventPayloadView, CoreError> {
        if scope != "full" {
            return Err(CoreError::Forbidden(
                "API key must have 'full' scope to access raw event payload".to_string(),
            ));
        }

        let repo = EventRepository::new(&self.pool);
        let event = repo
            .find_by_tenant_and_id(tenant_id, id)
            .await?
            .ok_or_else(|| CoreError::NotFound(format!("Event '{id}' not found")))?;

        // Audit log entry for sensitive raw payload access
        let audit_repo = AuditLogRepository::new(&self.pool);
        let _ = audit_repo
            .create(
                tenant_id,
                None,
                "event.raw_accessed",
                Some("event"),
                Some(id),
                serde_json::json!({
                    "event_id": id,
                    "event_type": event.event_type,
                }),
            )
            .await;

        let payload_str = match &event.payload {
            serde_json::Value::String(s) => s.clone(),
            other => serde_json::to_string(other).unwrap_or_default(),
        };

        Ok(RawEventPayloadView {
            event_id: event.id,
            headers: event.headers,
            payload: payload_str,
            is_binary: false,
        })
    }

    pub async fn delete_event_compliance(
        &self,
        tenant_id: Uuid,
        id: Uuid,
    ) -> Result<(), CoreError> {
        let repo = EventRepository::new(&self.pool);
        let event = repo
            .find_by_tenant_and_id(tenant_id, id)
            .await?
            .ok_or_else(|| CoreError::NotFound(format!("Event '{id}' not found")))?;

        // Audit log entry BEFORE deletion (do NOT copy payload into audit log)
        let audit_repo = AuditLogRepository::new(&self.pool);
        let _ = audit_repo
            .create(
                tenant_id,
                None,
                "event.compliance_deleted",
                Some("event"),
                Some(id),
                serde_json::json!({
                    "event_id": id,
                    "event_type": event.event_type,
                    "source_id": event.source_id,
                    "deleted_reason": "compliance",
                }),
            )
            .await;

        repo.delete_compliance(tenant_id, id).await?;

        Ok(())
    }

    pub async fn get_event_deliveries(
        &self,
        tenant_id: Uuid,
        id: Uuid,
    ) -> Result<Vec<EventDeliveryView>, CoreError> {
        let repo = EventRepository::new(&self.pool);
        if repo.find_by_tenant_and_id(tenant_id, id).await?.is_none() {
            return Err(CoreError::NotFound(format!("Event '{id}' not found")));
        }

        let deliveries = repo.get_deliveries(tenant_id, id).await?;

        Ok(deliveries
            .into_iter()
            .map(|d| EventDeliveryView {
                id: d.id,
                destination_id: d.destination_id,
                destination_name: d.destination_name,
                status: d.status,
                attempt_count: d.attempt_count,
                next_attempt_at: d.next_attempt_at,
                delivered_at: d.delivered_at,
                created_at: d.created_at,
            })
            .collect())
    }

    pub async fn list_events(
        &self,
        tenant_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<EventView>, CoreError> {
        let limit = if limit <= 0 || limit > 100 { 20 } else { limit };
        let offset = if offset < 0 { 0 } else { offset };

        let repo = EventRepository::new(&self.pool);
        let events = repo.list_by_tenant(tenant_id, limit, offset).await?;

        Ok(events
            .into_iter()
            .map(|e| EventView {
                id: e.id,
                tenant_id: e.tenant_id,
                source_id: e.source_id,
                event_type: e.event_type,
                idempotency_key: e.idempotency_key,
                headers: e.headers,
                payload: e.payload,
                status: e.status,
                created_at: e.created_at,
            })
            .collect())
    }

    pub async fn ingest(
        &self,
        tenant_id: Uuid,
        source_slug: &str,
        headers: serde_json::Value,
        input: IngestWebhookInput,
    ) -> Result<EventView, CoreError> {
        let source_repo = SourceRepository::new(&self.pool);
        let source = source_repo
            .find_by_tenant_and_slug(tenant_id, source_slug)
            .await?
            .ok_or_else(|| CoreError::NotFound(format!("Source with slug '{source_slug}' not found")))?;

        self.create_event(
            tenant_id,
            CreateEventInput {
                source_id: Some(source.id),
                event_type: input.event_type,
                payload: input.payload,
                idempotency_key: input.idempotency_key,
                headers: Some(headers),
            },
        )
        .await
    }
}
