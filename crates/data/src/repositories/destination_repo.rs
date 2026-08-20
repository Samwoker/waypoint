use sqlx::PgPool;
use uuid::Uuid;

use relay_core::error::CoreError;
use crate::models::Destination;

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
                (metadata->>'rate_limit_rps')::INTEGER AS rate_limit_rps,
                timeout_ms,
                (status = 'active') AS is_active,
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
                (metadata->>'rate_limit_rps')::INTEGER AS rate_limit_rps,
                timeout_ms,
                (status = 'active') AS is_active,
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
                (metadata->>'rate_limit_rps')::INTEGER AS rate_limit_rps,
                timeout_ms,
                (status = 'active') AS is_active,
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

        let metadata = if let Some(rps) = rate_limit_rps {
            serde_json::json!({ "rate_limit_rps": rps })
        } else {
            serde_json::json!({})
        };

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
                (metadata->>'rate_limit_rps')::INTEGER AS rate_limit_rps,
                timeout_ms,
                (status = 'active') AS is_active,
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
        is_active: Option<bool>,
    ) -> Result<Destination, CoreError> {
        let existing = self
            .find_by_tenant_and_id(tenant_id, id)
            .await?
            .ok_or_else(|| CoreError::NotFound(format!("Destination '{id}' not found")))?;

        let new_name = name.unwrap_or(&existing.name);
        let new_url = url.unwrap_or(&existing.url);
        let new_description = description.or(existing.description.as_deref());
        let new_timeout_ms = timeout_ms.unwrap_or(existing.timeout_ms);
        let new_status = match is_active {
            Some(true) => "active",
            Some(false) => "inactive",
            None => {
                if existing.is_active {
                    "active"
                } else {
                    "inactive"
                }
            }
        };

        let new_metadata = if let Some(rps) = rate_limit_rps.or(existing.rate_limit_rps) {
            serde_json::json!({ "rate_limit_rps": rps })
        } else {
            serde_json::json!({})
        };

        let row = sqlx::query_as::<_, Destination>(
            r#"
            UPDATE destinations
            SET
                name = $1,
                url = $2,
                description = $3,
                timeout_ms = $4,
                status = $5::destination_status,
                metadata = $6,
                updated_at = NOW()
            WHERE tenant_id = $7 AND id = $8 AND status != 'deleted'
            RETURNING
                id,
                tenant_id,
                name,
                url,
                description,
                (metadata->>'rate_limit_rps')::INTEGER AS rate_limit_rps,
                timeout_ms,
                (status = 'active') AS is_active,
                created_at,
                updated_at
            "#,
        )
        .bind(new_name)
        .bind(new_url)
        .bind(new_description)
        .bind(new_timeout_ms)
        .bind(new_status)
        .bind(new_metadata)
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
}
