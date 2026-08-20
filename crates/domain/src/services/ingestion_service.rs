use std::sync::Arc;
use data::repositories::{EventRepository, SourceRepository};
use sqlx::PgPool;
use uuid::Uuid;

use relay_core::error::CoreError;
use data::queue::RedisQueue;
use crate::dto::{CreateEventInput, EventView, IngestWebhookInput};

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
                // Handled concurrent insertion with same idempotency key
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
