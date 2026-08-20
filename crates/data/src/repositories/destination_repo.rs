use sqlx::PgPool;
use uuid::Uuid;

use relay_core::error::CoreError;
use crate::models::{Destination, DestinationHealthStats};

#[derive(Clone, Debug)]
pub struct DestinationRepository<'a> {
    pub pool: &'a PgPool,
}

impl<'a> DestinationRepository<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }

    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<Destination>, CoreError> {
        let row = sqlx::query_as::<_, Destination>(
            r#"
            SELECT
                id,
                tenant_id,
                name,
                url,
                description,
                CASE
                    WHEN metadata->>'paused' = 'true' THEN 'paused'
                    ELSE status::text
                END AS status,
                timeout_ms,
                max_retries,
                (status = 'active' AND (metadata->>'paused' IS NULL OR metadata->>'paused' != 'true')) AS is_active,
                COALESCE((metadata->>'consecutive_failures')::INTEGER, 0) AS consecutive_failures,
                (metadata->>'circuit_opened_at')::TIMESTAMPTZ AS circuit_opened_at,
                COALESCE(metadata->>'retry_backoff_strategy', 'exponential') AS retry_backoff_strategy,
                (metadata->>'rate_limit_rps')::INTEGER AS rate_limit_rps,
                secret_encrypted,
                created_at,
                updated_at
            FROM destinations
            WHERE id = $1 AND status != 'deleted'
            "#,
        )
        .bind(id)
        .fetch_optional(self.pool)
        .await
        .map_err(|e| CoreError::Internal(format!("Database error fetching destination by id: {e}")))?;

        Ok(row)
    }

    pub async fn find_by_tenant_and_id(&self, tenant_id: Uuid, id: Uuid) -> Result<Option<Destination>, CoreError> {
        let row = sqlx::query_as::<_, Destination>(
            r#"
            SELECT
                id,
                tenant_id,
                name,
                url,
                description,
                CASE
                    WHEN metadata->>'paused' = 'true' THEN 'paused'
                    ELSE status::text
                END AS status,
                timeout_ms,
                max_retries,
                (status = 'active' AND (metadata->>'paused' IS NULL OR metadata->>'paused' != 'true')) AS is_active,
                COALESCE((metadata->>'consecutive_failures')::INTEGER, 0) AS consecutive_failures,
                (metadata->>'circuit_opened_at')::TIMESTAMPTZ AS circuit_opened_at,
                COALESCE(metadata->>'retry_backoff_strategy', 'exponential') AS retry_backoff_strategy,
                (metadata->>'rate_limit_rps')::INTEGER AS rate_limit_rps,
                secret_encrypted,
                created_at,
                updated_at
            FROM destinations
            WHERE tenant_id = $1 AND id = $2 AND status != 'deleted'
            "#,
        )
        .bind(tenant_id)
        .bind(id)
        .fetch_optional(self.pool)
        .await
        .map_err(|e| CoreError::Internal(format!("Database error fetching destination: {e}")))?;

        Ok(row)
    }

    pub async fn list_by_tenant(&self, tenant_id: Uuid, limit: i64, offset: i64) -> Result<Vec<Destination>, CoreError> {
        let rows = sqlx::query_as::<_, Destination>(
            r#"
            SELECT
                id,
                tenant_id,
                name,
                url,
                description,
                CASE
                    WHEN metadata->>'paused' = 'true' THEN 'paused'
                    ELSE status::text
                END AS status,
                timeout_ms,
                max_retries,
                (status = 'active' AND (metadata->>'paused' IS NULL OR metadata->>'paused' != 'true')) AS is_active,
                COALESCE((metadata->>'consecutive_failures')::INTEGER, 0) AS consecutive_failures,
                (metadata->>'circuit_opened_at')::TIMESTAMPTZ AS circuit_opened_at,
                COALESCE(metadata->>'retry_backoff_strategy', 'exponential') AS retry_backoff_strategy,
                (metadata->>'rate_limit_rps')::INTEGER AS rate_limit_rps,
                secret_encrypted,
                created_at,
                updated_at
            FROM destinations
            WHERE tenant_id = $1 AND status != 'deleted'
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(tenant_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(self.pool)
        .await
        .map_err(|e| CoreError::Internal(format!("Database error listing destinations: {e}")))?;

        Ok(rows)
    }

    pub async fn create(
        &self,
        tenant_id: Uuid,
        name: &str,
        url: &str,
        description: Option<&str>,
        rate_limit_rps: Option<i32>,
        timeout_ms: i32,
        max_retries: i32,
        headers: Option<&serde_json::Value>,
        secret_encrypted: Option<&str>,
    ) -> Result<Destination, CoreError> {
        let id = Uuid::new_v4();
        let default_headers = serde_json::json!({});
        let headers_val = headers.unwrap_or(&default_headers);

        let mut metadata_map = serde_json::Map::new();
        if let Some(rps) = rate_limit_rps {
            metadata_map.insert("rate_limit_rps".to_string(), serde_json::json!(rps));
        }
        metadata_map.insert("consecutive_failures".to_string(), serde_json::json!(0));
        metadata_map.insert("retry_backoff_strategy".to_string(), serde_json::json!("exponential"));
        let metadata = serde_json::Value::Object(metadata_map);

        let row = sqlx::query_as::<_, Destination>(
            r#"
            INSERT INTO destinations (
                id, tenant_id, name, url, description, status, secret_encrypted, timeout_ms, max_retries, headers, metadata, created_at, updated_at
            ) VALUES (
                $1, $2, $3, $4, $5, 'active', $6, $7, $8, $9, $10, NOW(), NOW()
            )
            RETURNING
                id,
                tenant_id,
                name,
                url,
                description,
                'active' AS status,
                timeout_ms,
                max_retries,
                true AS is_active,
                0 AS consecutive_failures,
                NULL::TIMESTAMPTZ AS circuit_opened_at,
                'exponential' AS retry_backoff_strategy,
                (metadata->>'rate_limit_rps')::INTEGER AS rate_limit_rps,
                secret_encrypted,
                created_at,
                updated_at
            "#,
        )
        .bind(id)
        .bind(tenant_id)
        .bind(name)
        .bind(url)
        .bind(description)
        .bind(secret_encrypted)
        .bind(timeout_ms)
        .bind(max_retries)
        .bind(headers_val)
        .bind(metadata)
        .fetch_one(self.pool)
        .await
        .map_err(|e| {
            if let Some(db_err) = e.as_database_error() {
                if db_err.code().as_deref() == Some("23503") {
                    return CoreError::NotFound(format!("Tenant '{tenant_id}' does not exist"));
                }
            }
            CoreError::Internal(format!("Database error creating destination: {e}"))
        })?;

        Ok(row)
    }

    pub async fn update(
        &self,
        tenant_id: Uuid,
        id: Uuid,
        name: Option<&str>,
        url: Option<&str>,
        description: Option<&str>,
        rate_limit_rps: Option<i32>,
        timeout_ms: Option<i32>,
        max_retries: Option<i32>,
        retry_backoff_strategy: Option<&str>,
        is_active: Option<bool>,
    ) -> Result<Destination, CoreError> {
        let existing = self
            .find_by_tenant_and_id(tenant_id, id)
            .await?
            .ok_or_else(|| CoreError::NotFound(format!("Destination '{id}' not found")))?;

        let new_name = name.unwrap_or(&existing.name);
        let url_changed = url.map(|u| u != existing.url).unwrap_or(false);
        let new_url = url.unwrap_or(&existing.url);
        let new_description = description.or(existing.description.as_deref());
        let new_timeout_ms = timeout_ms.unwrap_or(existing.timeout_ms);
        let new_max_retries = max_retries.unwrap_or(existing.max_retries);
        let new_strategy = retry_backoff_strategy.unwrap_or(&existing.retry_backoff_strategy);
        let new_status = match is_active {
            Some(true) => "active",
            Some(false) => "inactive",
            None => {
                if existing.is_active {
                    "active"
                } else if existing.status == "disabled" || existing.status == "paused" {
                    "disabled"
                } else {
                    "inactive"
                }
            }
        };

        let consecutive_failures = if url_changed {
            0
        } else {
            existing.consecutive_failures
        };
        let circuit_opened_at = if url_changed {
            None
        } else {
            existing.circuit_opened_at
        };

        let mut metadata_map = serde_json::Map::new();
        if let Some(rps) = rate_limit_rps.or(existing.rate_limit_rps) {
            metadata_map.insert("rate_limit_rps".to_string(), serde_json::json!(rps));
        }
        metadata_map.insert("consecutive_failures".to_string(), serde_json::json!(consecutive_failures));
        if let Some(co) = circuit_opened_at {
            metadata_map.insert("circuit_opened_at".to_string(), serde_json::json!(co));
        }
        metadata_map.insert("retry_backoff_strategy".to_string(), serde_json::json!(new_strategy));
        let metadata = serde_json::Value::Object(metadata_map);

        let row = sqlx::query_as::<_, Destination>(
            r#"
            UPDATE destinations
            SET
                name = $1,
                url = $2,
                description = $3,
                timeout_ms = $4,
                max_retries = $5,
                status = $6::destination_status,
                metadata = $7,
                updated_at = NOW()
            WHERE tenant_id = $8 AND id = $9 AND status != 'deleted'
            RETURNING
                id,
                tenant_id,
                name,
                url,
                description,
                CASE
                    WHEN metadata->>'paused' = 'true' THEN 'paused'
                    ELSE status::text
                END AS status,
                timeout_ms,
                max_retries,
                (status = 'active' AND (metadata->>'paused' IS NULL OR metadata->>'paused' != 'true')) AS is_active,
                COALESCE((metadata->>'consecutive_failures')::INTEGER, 0) AS consecutive_failures,
                (metadata->>'circuit_opened_at')::TIMESTAMPTZ AS circuit_opened_at,
                COALESCE(metadata->>'retry_backoff_strategy', 'exponential') AS retry_backoff_strategy,
                (metadata->>'rate_limit_rps')::INTEGER AS rate_limit_rps,
                secret_encrypted,
                created_at,
                updated_at
            "#,
        )
        .bind(new_name)
        .bind(new_url)
        .bind(new_description)
        .bind(new_timeout_ms)
        .bind(new_max_retries)
        .bind(new_status)
        .bind(metadata)
        .bind(tenant_id)
        .bind(id)
        .fetch_one(self.pool)
        .await
        .map_err(|e| CoreError::Internal(format!("Database error updating destination: {e}")))?;

        Ok(row)
    }

    pub async fn delete(&self, tenant_id: Uuid, id: Uuid) -> Result<(), CoreError> {
        let result = sqlx::query(
            r#"
            UPDATE destinations
            SET status = 'deleted', updated_at = NOW()
            WHERE tenant_id = $1 AND id = $2 AND status != 'deleted'
            "#,
        )
        .bind(tenant_id)
        .bind(id)
        .execute(self.pool)
        .await
        .map_err(|e| CoreError::Internal(format!("Database error deleting destination: {e}")))?;

        if result.rows_affected() == 0 {
            return Err(CoreError::NotFound(format!("Destination '{id}' not found")));
        }

        Ok(())
    }

    pub async fn pause(&self, tenant_id: Uuid, id: Uuid) -> Result<Destination, CoreError> {
        let row = sqlx::query_as::<_, Destination>(
            r#"
            UPDATE destinations
            SET
                status = 'disabled',
                metadata = jsonb_set(COALESCE(metadata, '{}'::jsonb), '{paused}', 'true'),
                updated_at = NOW()
            WHERE tenant_id = $1 AND id = $2 AND status != 'deleted'
            RETURNING
                id,
                tenant_id,
                name,
                url,
                description,
                'paused' AS status,
                timeout_ms,
                max_retries,
                false AS is_active,
                COALESCE((metadata->>'consecutive_failures')::INTEGER, 0) AS consecutive_failures,
                (metadata->>'circuit_opened_at')::TIMESTAMPTZ AS circuit_opened_at,
                COALESCE(metadata->>'retry_backoff_strategy', 'exponential') AS retry_backoff_strategy,
                (metadata->>'rate_limit_rps')::INTEGER AS rate_limit_rps,
                secret_encrypted,
                created_at,
                updated_at
            "#,
        )
        .bind(tenant_id)
        .bind(id)
        .fetch_optional(self.pool)
        .await
        .map_err(|e| CoreError::Internal(format!("Database error pausing destination: {e}")))?;

        row.ok_or_else(|| CoreError::NotFound(format!("Destination '{id}' not found")))
    }

    pub async fn resume(&self, tenant_id: Uuid, id: Uuid) -> Result<Destination, CoreError> {
        let mut tx = self.pool.begin().await.map_err(|e| CoreError::Internal(format!("Database error starting transaction: {e}")))?;

        let row = sqlx::query_as::<_, Destination>(
            r#"
            UPDATE destinations
            SET
                status = 'active',
                metadata = (COALESCE(metadata, '{}'::jsonb) - 'paused' - 'circuit_opened_at') || jsonb_build_object('consecutive_failures', 0),
                updated_at = NOW()
            WHERE tenant_id = $1 AND id = $2 AND status != 'deleted'
            RETURNING
                id,
                tenant_id,
                name,
                url,
                description,
                'active' AS status,
                timeout_ms,
                max_retries,
                true AS is_active,
                0 AS consecutive_failures,
                NULL::TIMESTAMPTZ AS circuit_opened_at,
                COALESCE(metadata->>'retry_backoff_strategy', 'exponential') AS retry_backoff_strategy,
                (metadata->>'rate_limit_rps')::INTEGER AS rate_limit_rps,
                secret_encrypted,
                created_at,
                updated_at
            "#,
        )
        .bind(tenant_id)
        .bind(id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| CoreError::Internal(format!("Database error resuming destination: {e}")))?;

        let destination = match row {
            Some(d) => d,
            None => {
                tx.rollback().await.ok();
                return Err(CoreError::NotFound(format!("Destination '{id}' not found")));
            }
        };

        // Reset next_attempt_at for pending/failed deliveries
        sqlx::query(
            r#"
            UPDATE deliveries
            SET next_attempt_at = NOW(), status = 'pending', updated_at = NOW()
            WHERE destination_id = $1 AND status IN ('pending', 'failed') AND next_attempt_at > NOW()
            "#,
        )
        .bind(id)
        .execute(&mut *tx)
        .await
        .map_err(|e| CoreError::Internal(format!("Database error updating delivery retries on resume: {e}")))?;

        tx.commit().await.map_err(|e| CoreError::Internal(format!("Database error committing resume transaction: {e}")))?;

        Ok(destination)
    }

    pub async fn get_health_stats(&self, tenant_id: Uuid, id: Uuid) -> Result<Option<DestinationHealthStats>, CoreError> {
        let dest = self.find_by_tenant_and_id(tenant_id, id).await?;
        let dest = match dest {
            Some(d) => d,
            None => return Ok(None),
        };

        let row = sqlx::query_as::<_, (Option<i64>, Option<i64>)>(
            r#"
            SELECT
                count(*) FILTER (WHERE da.response_status BETWEEN 200 AND 299) AS successes,
                count(*) AS total
            FROM delivery_attempts da
            JOIN deliveries d ON d.id = da.delivery_id
            WHERE d.destination_id = $1
              AND da.created_at > NOW() - interval '1 hour'
            "#,
        )
        .bind(id)
        .fetch_one(self.pool)
        .await
        .map_err(|e| CoreError::Internal(format!("Database error calculating destination health: {e}")))?;

        Ok(Some(DestinationHealthStats {
            status: dest.status,
            consecutive_failures: dest.consecutive_failures,
            circuit_opened_at: dest.circuit_opened_at,
            successes: row.0.unwrap_or(0),
            total: row.1.unwrap_or(0),
        }))
    }
}
