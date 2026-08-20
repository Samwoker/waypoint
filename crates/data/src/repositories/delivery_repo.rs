use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use relay_core::error::CoreError;
use crate::models::{Delivery, DeliveryAttempt};

#[derive(Clone, Debug)]
pub struct DeliveryRepository<'a> {
    pub pool: &'a PgPool,
}

impl<'a> DeliveryRepository<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }

    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<Delivery>, CoreError> {
        let row = sqlx::query_as::<_, Delivery>(
            r#"
            SELECT
                id,
                tenant_id,
                event_id,
                COALESCE(subscription_id, '00000000-0000-0000-0000-000000000000'::uuid) AS subscription_id,
                destination_id,
                status::text AS status,
                attempt_count,
                max_attempts,
                next_attempt_at AS next_retry_at,
                created_at,
                updated_at
            FROM deliveries
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(self.pool)
        .await
        .map_err(|e| CoreError::Internal(format!("Database error fetching delivery by id: {e}")))?;

        Ok(row)
    }

    pub async fn find_by_tenant_and_id(&self, tenant_id: Uuid, id: Uuid) -> Result<Option<Delivery>, CoreError> {
        let row = sqlx::query_as::<_, Delivery>(
            r#"
            SELECT
                id,
                tenant_id,
                event_id,
                COALESCE(subscription_id, '00000000-0000-0000-0000-000000000000'::uuid) AS subscription_id,
                destination_id,
                status::text AS status,
                attempt_count,
                max_attempts,
                next_attempt_at AS next_retry_at,
                created_at,
                updated_at
            FROM deliveries
            WHERE tenant_id = $1 AND id = $2
            "#,
        )
        .bind(tenant_id)
        .bind(id)
        .fetch_optional(self.pool)
        .await
        .map_err(|e| CoreError::Internal(format!("Database error fetching delivery: {e}")))?;

        Ok(row)
    }

    pub async fn find_by_event_and_destination(
        &self,
        tenant_id: Uuid,
        event_id: Uuid,
        destination_id: Uuid,
    ) -> Result<Option<Delivery>, CoreError> {
        let row = sqlx::query_as::<_, Delivery>(
            r#"
            SELECT
                id,
                tenant_id,
                event_id,
                COALESCE(subscription_id, '00000000-0000-0000-0000-000000000000'::uuid) AS subscription_id,
                destination_id,
                status::text AS status,
                attempt_count,
                max_attempts,
                next_attempt_at AS next_retry_at,
                created_at,
                updated_at
            FROM deliveries
            WHERE tenant_id = $1 AND event_id = $2 AND destination_id = $3
            "#,
        )
        .bind(tenant_id)
        .bind(event_id)
        .bind(destination_id)
        .fetch_optional(self.pool)
        .await
        .map_err(|e| CoreError::Internal(format!("Database error fetching delivery by event and destination: {e}")))?;

        Ok(row)
    }

    pub async fn list_by_tenant(
        &self,
        tenant_id: Uuid,
        status: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Delivery>, CoreError> {
        let rows = if let Some(s) = status {
            sqlx::query_as::<_, Delivery>(
                r#"
                SELECT
                    id,
                    tenant_id,
                    event_id,
                    COALESCE(subscription_id, '00000000-0000-0000-0000-000000000000'::uuid) AS subscription_id,
                    destination_id,
                    status::text AS status,
                    attempt_count,
                    max_attempts,
                    next_attempt_at AS next_retry_at,
                    created_at,
                    updated_at
                FROM deliveries
                WHERE tenant_id = $1 AND status = $2::delivery_status
                ORDER BY created_at DESC
                LIMIT $3 OFFSET $4
                "#,
            )
            .bind(tenant_id)
            .bind(s)
            .bind(limit)
            .bind(offset)
            .fetch_all(self.pool)
            .await
        } else {
            sqlx::query_as::<_, Delivery>(
                r#"
                SELECT
                    id,
                    tenant_id,
                    event_id,
                    COALESCE(subscription_id, '00000000-0000-0000-0000-000000000000'::uuid) AS subscription_id,
                    destination_id,
                    status::text AS status,
                    attempt_count,
                    max_attempts,
                    next_attempt_at AS next_retry_at,
                    created_at,
                    updated_at
                FROM deliveries
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
        }
        .map_err(|e| CoreError::Internal(format!("Database error listing deliveries: {e}")))?;

        Ok(rows)
    }

    pub async fn list_paginated(
        &self,
        tenant_id: Uuid,
        destination_id: Option<Uuid>,
        status: Option<&str>,
        from: Option<DateTime<Utc>>,
        to: Option<DateTime<Utc>>,
        cursor_created_at: Option<DateTime<Utc>>,
        cursor_id: Option<Uuid>,
        limit: i64,
    ) -> Result<Vec<Delivery>, CoreError> {
        let rows = sqlx::query_as::<_, Delivery>(
            r#"
            SELECT
                id,
                tenant_id,
                event_id,
                COALESCE(subscription_id, '00000000-0000-0000-0000-000000000000'::uuid) AS subscription_id,
                destination_id,
                status::text AS status,
                attempt_count,
                max_attempts,
                next_attempt_at AS next_retry_at,
                created_at,
                updated_at
            FROM deliveries
            WHERE tenant_id = $1
              AND ($2::uuid IS NULL OR destination_id = $2)
              AND ($3::delivery_status IS NULL OR status = $3::delivery_status)
              AND ($4::timestamptz IS NULL OR created_at >= $4)
              AND ($5::timestamptz IS NULL OR created_at <= $5)
              AND ($6::timestamptz IS NULL OR (created_at, id) < ($6, $7))
            ORDER BY created_at DESC, id DESC
            LIMIT $8
            "#,
        )
        .bind(tenant_id)
        .bind(destination_id)
        .bind(status)
        .bind(from)
        .bind(to)
        .bind(cursor_created_at)
        .bind(cursor_id)
        .bind(limit)
        .fetch_all(self.pool)
        .await
        .map_err(|e| CoreError::Internal(format!("Database error querying paginated deliveries: {e}")))?;

        Ok(rows)
    }

    pub async fn list_attempts_by_delivery_id(
        &self,
        delivery_id: Uuid,
    ) -> Result<Vec<DeliveryAttempt>, CoreError> {
        let rows = sqlx::query_as::<_, DeliveryAttempt>(
            r#"
            SELECT
                id,
                delivery_id,
                attempt_number,
                response_status,
                request_headers,
                request_body,
                response_headers,
                response_body,
                error_message,
                duration_ms,
                created_at
            FROM delivery_attempts
            WHERE delivery_id = $1
            ORDER BY attempt_number ASC
            "#,
        )
        .bind(delivery_id)
        .fetch_all(self.pool)
        .await
        .map_err(|e| CoreError::Internal(format!("Database error listing delivery attempts: {e}")))?;

        Ok(rows)
    }

    pub async fn list_due_deliveries(&self, limit: i64) -> Result<Vec<Delivery>, CoreError> {
        let rows = sqlx::query_as::<_, Delivery>(
            r#"
            SELECT
                id,
                tenant_id,
                event_id,
                COALESCE(subscription_id, '00000000-0000-0000-0000-000000000000'::uuid) AS subscription_id,
                destination_id,
                status::text AS status,
                attempt_count,
                max_attempts,
                next_attempt_at AS next_retry_at,
                created_at,
                updated_at
            FROM deliveries
            WHERE status = 'pending' AND (next_attempt_at IS NULL OR next_attempt_at <= NOW())
            ORDER BY next_attempt_at ASC NULLS FIRST
            LIMIT $1
            "#,
        )
        .bind(limit)
        .fetch_all(self.pool)
        .await
        .map_err(|e| CoreError::Internal(format!("Database error listing due deliveries: {e}")))?;

        Ok(rows)
    }

    pub async fn list_dlq_by_tenant(
        &self,
        tenant_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Delivery>, CoreError> {
        self.list_by_tenant(tenant_id, Some("dead_letter"), limit, offset).await
    }

    pub async fn create(
        &self,
        tenant_id: Uuid,
        event_id: Uuid,
        subscription_id: Uuid,
        destination_id: Uuid,
        max_attempts: i32,
    ) -> Result<Delivery, CoreError> {
        let id = Uuid::new_v4();

        let row = sqlx::query_as::<_, Delivery>(
            r#"
            INSERT INTO deliveries (
                id, tenant_id, event_id, subscription_id, destination_id, status, attempt_count, max_attempts, next_attempt_at, created_at, updated_at
            ) VALUES (
                $1, $2, $3, $4, $5, 'pending', 0, $6, NOW(), NOW(), NOW()
            )
            RETURNING
                id,
                tenant_id,
                event_id,
                COALESCE(subscription_id, '00000000-0000-0000-0000-000000000000'::uuid) AS subscription_id,
                destination_id,
                status::text AS status,
                attempt_count,
                max_attempts,
                next_attempt_at AS next_retry_at,
                created_at,
                updated_at
            "#,
        )
        .bind(id)
        .bind(tenant_id)
        .bind(event_id)
        .bind(subscription_id)
        .bind(destination_id)
        .bind(max_attempts)
        .fetch_one(self.pool)
        .await
        .map_err(|e| CoreError::Internal(format!("Database error creating delivery: {e}")))?;

        Ok(row)
    }

    pub async fn record_attempt(
        &self,
        delivery_id: Uuid,
        attempt_number: i32,
        status_code: Option<i32>,
        request_headers: Option<serde_json::Value>,
        request_body: Option<&str>,
        response_headers: Option<serde_json::Value>,
        response_body: Option<&str>,
        error_message: Option<&str>,
        duration_ms: Option<i32>,
    ) -> Result<DeliveryAttempt, CoreError> {
        let id = Uuid::new_v4();
        let status = if status_code.map(|c| c >= 200 && c < 300).unwrap_or(false) {
            "success"
        } else {
            "failed"
        };

        let row = sqlx::query_as::<_, DeliveryAttempt>(
            r#"
            INSERT INTO delivery_attempts (
                id, delivery_id, attempt_number, status, response_status, request_headers, request_body, response_headers, response_body, error_message, duration_ms, created_at
            ) VALUES (
                $1, $2, $3, $4::attempt_status, $5, $6, $7, $8, $9, $10, $11, NOW()
            )
            RETURNING
                id,
                delivery_id,
                attempt_number,
                response_status,
                request_headers,
                request_body,
                response_headers,
                response_body,
                error_message,
                duration_ms,
                created_at
            "#,
        )
        .bind(id)
        .bind(delivery_id)
        .bind(attempt_number)
        .bind(status)
        .bind(status_code)
        .bind(request_headers)
        .bind(request_body.and_then(|b| serde_json::from_str::<serde_json::Value>(b).ok()))
        .bind(response_headers)
        .bind(response_body)
        .bind(error_message)
        .bind(duration_ms)
        .fetch_one(self.pool)
        .await
        .map_err(|e| CoreError::Internal(format!("Database error recording delivery attempt: {e}")))?;

        Ok(row)
    }

    pub async fn update_status(
        &self,
        id: Uuid,
        status: &str,
        attempt_count: i32,
        next_retry_at: Option<DateTime<Utc>>,
    ) -> Result<(), CoreError> {
        sqlx::query(
            r#"
            UPDATE deliveries
            SET
                status = $1::delivery_status,
                attempt_count = $2,
                next_attempt_at = $3,
                updated_at = NOW()
            WHERE id = $4
            "#,
        )
        .bind(status)
        .bind(attempt_count)
        .bind(next_retry_at)
        .bind(id)
        .execute(self.pool)
        .await
        .map_err(|e| CoreError::Internal(format!("Database error updating delivery status: {e}")))?;

        Ok(())
    }

    pub async fn replay_delivery(
        &self,
        tenant_id: Uuid,
        id: Uuid,
        reset_attempt_count: bool,
    ) -> Result<Delivery, CoreError> {
        let row = sqlx::query_as::<_, Delivery>(
            r#"
            UPDATE deliveries
            SET
                status = 'pending',
                next_attempt_at = NOW(),
                attempt_count = CASE WHEN $3 = TRUE THEN 0 ELSE attempt_count END,
                updated_at = NOW()
            WHERE tenant_id = $1 AND id = $2
            RETURNING
                id,
                tenant_id,
                event_id,
                COALESCE(subscription_id, '00000000-0000-0000-0000-000000000000'::uuid) AS subscription_id,
                destination_id,
                status::text AS status,
                attempt_count,
                max_attempts,
                next_attempt_at AS next_retry_at,
                created_at,
                updated_at
            "#,
        )
        .bind(tenant_id)
        .bind(id)
        .bind(reset_attempt_count)
        .fetch_optional(self.pool)
        .await
        .map_err(|e| CoreError::Internal(format!("Database error replaying delivery: {e}")))?
        .ok_or_else(|| CoreError::NotFound(format!("Delivery '{id}' not found")))?;

        Ok(row)
    }

    pub async fn replay_batch(
        &self,
        tenant_id: Uuid,
        destination_id: Option<Uuid>,
        source_id: Option<Uuid>,
        from: Option<DateTime<Utc>>,
        to: Option<DateTime<Utc>>,
        status_filter: Option<&str>,
        limit: i64,
    ) -> Result<(i64, bool), CoreError> {
        // Query matching deliveries limit + 1
        let ids = sqlx::query_scalar::<_, Uuid>(
            r#"
            SELECT d.id
            FROM deliveries d
            JOIN events e ON e.id = d.event_id
            WHERE d.tenant_id = $1
              AND ($2::uuid IS NULL OR d.destination_id = $2)
              AND ($3::uuid IS NULL OR e.source_id = $3)
              AND ($4::timestamptz IS NULL OR d.created_at >= $4)
              AND ($5::timestamptz IS NULL OR d.created_at <= $5)
              AND ($6::delivery_status IS NULL OR d.status = $6::delivery_status)
            ORDER BY d.created_at DESC
            LIMIT $7
            "#,
        )
        .bind(tenant_id)
        .bind(destination_id)
        .bind(source_id)
        .bind(from)
        .bind(to)
        .bind(status_filter)
        .bind(limit + 1)
        .fetch_all(self.pool)
        .await
        .map_err(|e| CoreError::Internal(format!("Database error finding matching deliveries for batch replay: {e}")))?;

        let has_more = ids.len() > limit as usize;
        let update_ids: Vec<Uuid> = ids.into_iter().take(limit as usize).collect();

        if update_ids.is_empty() {
            return Ok((0, false));
        }

        let updated = sqlx::query(
            r#"
            UPDATE deliveries
            SET
                status = 'pending',
                next_attempt_at = NOW(),
                updated_at = NOW()
            WHERE id = ANY($1)
            "#,
        )
        .bind(&update_ids)
        .execute(self.pool)
        .await
        .map_err(|e| CoreError::Internal(format!("Database error updating batch deliveries: {e}")))?;

        Ok((updated.rows_affected() as i64, has_more))
    }

    pub async fn list_dlq_paginated(
        &self,
        tenant_id: Uuid,
        cursor_created_at: Option<DateTime<Utc>>,
        cursor_id: Option<Uuid>,
        limit: i64,
    ) -> Result<Vec<crate::models::DlqRecord>, CoreError> {
        let rows = sqlx::query_as::<_, crate::models::DlqRecord>(
            r#"
            SELECT
                d.id AS delivery_id,
                d.tenant_id,
                d.event_id,
                e.event_type,
                d.destination_id,
                dest.name AS destination_name,
                dest.url AS destination_url,
                d.status::text AS status,
                d.attempt_count,
                d.max_attempts,
                d.last_error,
                d.created_at,
                d.updated_at
            FROM deliveries d
            JOIN events e ON e.id = d.event_id
            JOIN destinations dest ON dest.id = d.destination_id
            WHERE d.tenant_id = $1
              AND (d.status = 'dead_letter'::delivery_status OR d.status = 'dead_lettered'::delivery_status)
              AND ($2::timestamptz IS NULL OR (d.created_at, d.id) < ($2, $3))
            ORDER BY d.created_at DESC, d.id DESC
            LIMIT $4
            "#,
        )
        .bind(tenant_id)
        .bind(cursor_created_at)
        .bind(cursor_id)
        .bind(limit)
        .fetch_all(self.pool)
        .await
        .map_err(|e| CoreError::Internal(format!("Database error listing DLQ records: {e}")))?;

        Ok(rows)
    }

    pub async fn requeue_dlq(
        &self,
        tenant_id: Uuid,
        event_id: Uuid,
        destination_id: Uuid,
    ) -> Result<Delivery, CoreError> {
        let existing = self
            .find_by_event_and_destination(tenant_id, event_id, destination_id)
            .await?
            .ok_or_else(|| CoreError::NotFound(format!("Delivery for event '{event_id}' and destination '{destination_id}' not found")))?;

        if existing.status != "dead_letter" && existing.status != "dead_lettered" {
            return Err(CoreError::Validation(format!(
                "Delivery is not dead-lettered (current status: {})",
                existing.status
            )));
        }

        let row = sqlx::query_as::<_, Delivery>(
            r#"
            UPDATE deliveries
            SET
                status = 'pending',
                attempt_count = 0,
                next_attempt_at = NOW(),
                updated_at = NOW()
            WHERE tenant_id = $1 AND id = $2
            RETURNING
                id,
                tenant_id,
                event_id,
                COALESCE(subscription_id, '00000000-0000-0000-0000-000000000000'::uuid) AS subscription_id,
                destination_id,
                status::text AS status,
                attempt_count,
                max_attempts,
                next_attempt_at AS next_retry_at,
                created_at,
                updated_at
            "#,
        )
        .bind(tenant_id)
        .bind(existing.id)
        .fetch_one(self.pool)
        .await
        .map_err(|e| CoreError::Internal(format!("Database error requeuing DLQ delivery: {e}")))?;

        Ok(row)
    }

    pub async fn discard_dlq(
        &self,
        tenant_id: Uuid,
        event_id: Uuid,
        destination_id: Uuid,
    ) -> Result<(), CoreError> {
        let existing = self
            .find_by_event_and_destination(tenant_id, event_id, destination_id)
            .await?
            .ok_or_else(|| CoreError::NotFound(format!("Delivery for event '{event_id}' and destination '{destination_id}' not found")))?;

        if existing.status == "discarded" {
            return Err(CoreError::Conflict("Delivery is already discarded".to_string()));
        }

        if existing.status != "dead_letter" && existing.status != "dead_lettered" {
            return Err(CoreError::Validation(format!(
                "Delivery is not in dead letter queue (current status: {})",
                existing.status
            )));
        }

        sqlx::query(
            r#"
            UPDATE deliveries
            SET
                status = 'discarded',
                updated_at = NOW()
            WHERE tenant_id = $1 AND id = $2
            "#,
        )
        .bind(tenant_id)
        .bind(existing.id)
        .execute(self.pool)
        .await
        .map_err(|e| CoreError::Internal(format!("Database error discarding DLQ delivery: {e}")))?;

        Ok(())
    }
}
