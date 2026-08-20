use sqlx::PgPool;
use uuid::Uuid;

use relay_core::error::CoreError;
use crate::models::{Event, VerificationLogRecord};

#[derive(Clone, Debug)]
pub struct EventRepository<'a> {
    pub pool: &'a PgPool,
}

impl<'a> EventRepository<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }

    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<Event>, CoreError> {
        let row = sqlx::query_as::<_, Event>(
            r#"
            SELECT
                id,
                tenant_id,
                source_id,
                event_type,
                idempotency_key,
                headers,
                payload,
                status::text AS status,
                created_at
            FROM events
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(self.pool)
        .await
        .map_err(|e| CoreError::Internal(format!("Database error fetching event by id: {e}")))?;

        Ok(row)
    }

    pub async fn find_by_tenant_and_id(&self, tenant_id: Uuid, id: Uuid) -> Result<Option<Event>, CoreError> {
        let row = sqlx::query_as::<_, Event>(
            r#"
            SELECT
                id,
                tenant_id,
                source_id,
                event_type,
                idempotency_key,
                headers,
                payload,
                status::text AS status,
                created_at
            FROM events
            WHERE tenant_id = $1 AND id = $2
            "#,
        )
        .bind(tenant_id)
        .bind(id)
        .fetch_optional(self.pool)
        .await
        .map_err(|e| CoreError::Internal(format!("Database error fetching event: {e}")))?;

        Ok(row)
    }

    pub async fn find_by_idempotency_key(
        &self,
        tenant_id: Uuid,
        idempotency_key: &str,
    ) -> Result<Option<Event>, CoreError> {
        let row = sqlx::query_as::<_, Event>(
            r#"
            SELECT
                id,
                tenant_id,
                source_id,
                event_type,
                idempotency_key,
                headers,
                payload,
                status::text AS status,
                created_at
            FROM events
            WHERE tenant_id = $1 AND idempotency_key = $2
            "#,
        )
        .bind(tenant_id)
        .bind(idempotency_key)
        .fetch_optional(self.pool)
        .await
        .map_err(|e| CoreError::Internal(format!("Database error fetching event by idempotency key: {e}")))?;

        Ok(row)
    }

    pub async fn list_by_tenant(
        &self,
        tenant_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Event>, CoreError> {
        let rows = sqlx::query_as::<_, Event>(
            r#"
            SELECT
                id,
                tenant_id,
                source_id,
                event_type,
                idempotency_key,
                headers,
                payload,
                status::text AS status,
                created_at
            FROM events
            WHERE tenant_id = $1
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(tenant_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(self.pool)
        .await
        .map_err(|e| CoreError::Internal(format!("Database error listing events: {e}")))?;

        Ok(rows)
    }

    pub async fn create(
        &self,
        tenant_id: Uuid,
        source_id: Uuid,
        event_type: &str,
        idempotency_key: Option<&str>,
        headers: serde_json::Value,
        payload: serde_json::Value,
    ) -> Result<Event, CoreError> {
        let id = Uuid::new_v4();

        let row = sqlx::query_as::<_, Event>(
            r#"
            INSERT INTO events (
                id, tenant_id, source_id, event_type, idempotency_key, headers, payload, status, created_at
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, 'received', NOW()
            )
            RETURNING
                id,
                tenant_id,
                source_id,
                event_type,
                idempotency_key,
                headers,
                payload,
                status::text AS status,
                created_at
            "#,
        )
        .bind(id)
        .bind(tenant_id)
        .bind(source_id)
        .bind(event_type)
        .bind(idempotency_key)
        .bind(headers)
        .bind(payload)
        .fetch_one(self.pool)
        .await
        .map_err(|e| {
            if let Some(db_err) = e.as_database_error() {
                if db_err.code().as_deref() == Some("23505") {
                    return CoreError::Conflict("Duplicate idempotency key".to_string());
                }
                if db_err.code().as_deref() == Some("23503") {
                    return CoreError::NotFound("Referenced source does not exist".to_string());
                }
            }
            CoreError::Internal(format!("Database error creating event: {e}"))
        })?;

        Ok(row)
    }

    pub async fn update_status(&self, id: Uuid, status: &str) -> Result<(), CoreError> {
        sqlx::query(
            r#"
            UPDATE events
            SET status = $1::event_status
            WHERE id = $2
            "#,
        )
        .bind(status)
        .bind(id)
        .execute(self.pool)
        .await
        .map_err(|e| CoreError::Internal(format!("Database error updating event status: {e}")))?;

        Ok(())
    }

    pub async fn get_verification_log(
        &self,
        tenant_id: Uuid,
        source_id: Uuid,
        limit: i64,
    ) -> Result<Vec<VerificationLogRecord>, CoreError> {
        let rows = sqlx::query_as::<_, VerificationLogRecord>(
            r#"
            SELECT
                received_at,
                COALESCE((headers->>'signature_valid')::boolean, (status != 'failed')) AS signature_valid,
                external_id AS external_event_id
            FROM events
            WHERE tenant_id = $1 AND source_id = $2
            ORDER BY received_at DESC
            LIMIT $3
            "#,
        )
        .bind(tenant_id)
        .bind(source_id)
        .bind(limit)
        .fetch_all(self.pool)
        .await
        .map_err(|e| CoreError::Internal(format!("Database error querying verification log: {e}")))?;

        Ok(rows)
    }
}
