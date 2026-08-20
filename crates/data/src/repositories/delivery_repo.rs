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
}
