use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use relay_core::error::CoreError;
use crate::models::{Event, EventDeliveryRecord, EventDeliverySummary, EventWithComputedStatus, VerificationLogRecord};

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
                received_at,
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
                received_at,
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
                received_at,
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
                received_at,
                created_at
            FROM events
            WHERE tenant_id = $1
            ORDER BY received_at DESC, created_at DESC
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

    pub async fn list_paginated_with_status(
        &self,
        tenant_id: Uuid,
        source_id: Option<Uuid>,
        status: Option<&str>,
        from: Option<DateTime<Utc>>,
        to: Option<DateTime<Utc>>,
        cursor_received_at: Option<DateTime<Utc>>,
        cursor_id: Option<Uuid>,
        limit: i64,
    ) -> Result<Vec<EventWithComputedStatus>, CoreError> {
        let rows = sqlx::query_as::<_, EventWithComputedStatus>(
            r#"
            WITH event_delivery_stats AS (
                SELECT
                    d.event_id,
                    count(*) AS total_count,
                    count(*) FILTER (WHERE d.status = 'delivered') AS delivered_count,
                    count(*) FILTER (WHERE d.status = 'failed') AS failed_count,
                    count(*) FILTER (WHERE d.status = 'pending') AS pending_count
                FROM deliveries d
                WHERE d.tenant_id = $1
                GROUP BY d.event_id
            ),
            events_with_status AS (
                SELECT
                    e.id,
                    e.tenant_id,
                    e.source_id,
                    e.event_type,
                    e.idempotency_key,
                    CASE
                        WHEN eds.total_count IS NULL OR eds.total_count = 0 THEN 'no_subscriptions'
                        WHEN eds.delivered_count = eds.total_count THEN 'delivered'
                        WHEN eds.failed_count > 0 AND eds.pending_count = 0 THEN 'failed'
                        ELSE 'pending'
                    END AS status,
                    e.received_at,
                    e.created_at
                FROM events e
                LEFT JOIN event_delivery_stats eds ON eds.event_id = e.id
                WHERE e.tenant_id = $1
                  AND ($2::uuid IS NULL OR e.source_id = $2)
                  AND ($3::timestamptz IS NULL OR e.received_at >= $3)
                  AND ($4::timestamptz IS NULL OR e.received_at <= $4)
            )
            SELECT
                id,
                tenant_id,
                source_id,
                event_type,
                idempotency_key,
                status,
                received_at,
                created_at
            FROM events_with_status
            WHERE ($5::text IS NULL OR status = $5)
              AND ($6::timestamptz IS NULL OR (received_at, id) < ($6, $7))
            ORDER BY received_at DESC, id DESC
            LIMIT $8
            "#,
        )
        .bind(tenant_id)
        .bind(source_id)
        .bind(from)
        .bind(to)
        .bind(status)
        .bind(cursor_received_at)
        .bind(cursor_id)
        .bind(limit)
        .fetch_all(self.pool)
        .await
        .map_err(|e| CoreError::Internal(format!("Database error querying paginated events: {e}")))?;

        Ok(rows)
    }

    pub async fn get_delivery_summary(
        &self,
        tenant_id: Uuid,
        event_id: Uuid,
    ) -> Result<EventDeliverySummary, CoreError> {
        let row = sqlx::query_as::<_, (Option<i64>, Option<i64>, Option<i64>, Option<i64>)>(
            r#"
            SELECT
                count(*) AS total,
                count(*) FILTER (WHERE status = 'delivered') AS delivered,
                count(*) FILTER (WHERE status = 'failed') AS failed,
                count(*) FILTER (WHERE status = 'pending') AS pending
            FROM deliveries
            WHERE tenant_id = $1 AND event_id = $2
            "#,
        )
        .bind(tenant_id)
        .bind(event_id)
        .fetch_one(self.pool)
        .await
        .map_err(|e| CoreError::Internal(format!("Database error calculating event delivery summary: {e}")))?;

        Ok(EventDeliverySummary {
            total: row.0.unwrap_or(0),
            delivered: row.1.unwrap_or(0),
            failed: row.2.unwrap_or(0),
            pending: row.3.unwrap_or(0),
        })
    }

    pub async fn get_computed_status(
        &self,
        tenant_id: Uuid,
        event_id: Uuid,
    ) -> Result<String, CoreError> {
        let status = sqlx::query_scalar::<_, String>(
            r#"
            SELECT
                CASE
                    WHEN count(*) = 0 THEN 'no_subscriptions'
                    WHEN count(*) FILTER (WHERE status = 'delivered') = count(*) THEN 'delivered'
                    WHEN count(*) FILTER (WHERE status = 'failed') > 0 AND count(*) FILTER (WHERE status = 'pending') = 0 THEN 'failed'
                    ELSE 'pending'
                END AS status
            FROM deliveries
            WHERE tenant_id = $1 AND event_id = $2
            "#,
        )
        .bind(tenant_id)
        .bind(event_id)
        .fetch_one(self.pool)
        .await
        .map_err(|e| CoreError::Internal(format!("Database error calculating event status: {e}")))?;

        Ok(status)
    }

    pub async fn get_deliveries(
        &self,
        tenant_id: Uuid,
        event_id: Uuid,
    ) -> Result<Vec<EventDeliveryRecord>, CoreError> {
        let rows = sqlx::query_as::<_, EventDeliveryRecord>(
            r#"
            SELECT
                d.id,
                d.destination_id,
                dest.name AS destination_name,
                d.status::text AS status,
                d.attempt_count,
                d.next_attempt_at,
                d.delivered_at,
                d.created_at
            FROM deliveries d
            JOIN destinations dest ON dest.id = d.destination_id
            WHERE d.tenant_id = $1 AND d.event_id = $2
            ORDER BY d.created_at ASC
            "#,
        )
        .bind(tenant_id)
        .bind(event_id)
        .fetch_all(self.pool)
        .await
        .map_err(|e| CoreError::Internal(format!("Database error fetching event deliveries: {e}")))?;

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
                id, tenant_id, source_id, event_type, idempotency_key, headers, payload, status, received_at, created_at
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, 'received', NOW(), NOW()
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
                received_at,
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

    pub async fn delete_compliance(&self, tenant_id: Uuid, id: Uuid) -> Result<(), CoreError> {
        let result = sqlx::query(
            r#"
            DELETE FROM events
            WHERE tenant_id = $1 AND id = $2
            "#,
        )
        .bind(tenant_id)
        .bind(id)
        .execute(self.pool)
        .await
        .map_err(|e| CoreError::Internal(format!("Database error deleting event: {e}")))?;

        if result.rows_affected() == 0 {
            return Err(CoreError::NotFound(format!("Event '{id}' not found")));
        }

        Ok(())
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
