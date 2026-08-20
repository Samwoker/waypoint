use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use relay_core::error::CoreError;
use crate::models::ApiKey;

#[derive(Clone, Debug)]
pub struct ApiKeyRepository<'a> {
    pub pool: &'a PgPool,
}

impl<'a> ApiKeyRepository<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }

    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<ApiKey>, CoreError> {
        let row = sqlx::query_as::<_, ApiKey>(
            r#"
            SELECT id, tenant_id, name, key_prefix, key_hash, expires_at, last_used_at, created_at
            FROM api_keys
            WHERE id = $1 AND status = 'active'
            "#,
        )
        .bind(id)
        .fetch_optional(self.pool)
        .await
        .map_err(|e| CoreError::Internal(format!("Database error fetching api key: {e}")))?;

        Ok(row)
    }

    pub async fn find_by_tenant_and_id(&self, tenant_id: Uuid, id: Uuid) -> Result<Option<ApiKey>, CoreError> {
        let row = sqlx::query_as::<_, ApiKey>(
            r#"
            SELECT id, tenant_id, name, key_prefix, key_hash, expires_at, last_used_at, created_at
            FROM api_keys
            WHERE tenant_id = $1 AND id = $2 AND status = 'active'
            "#,
        )
        .bind(tenant_id)
        .bind(id)
        .fetch_optional(self.pool)
        .await
        .map_err(|e| CoreError::Internal(format!("Database error fetching api key: {e}")))?;

        Ok(row)
    }

    pub async fn find_by_key_hash(&self, key_hash: &str) -> Result<Option<ApiKey>, CoreError> {
        let row = sqlx::query_as::<_, ApiKey>(
            r#"
            SELECT id, tenant_id, name, key_prefix, key_hash, expires_at, last_used_at, created_at
            FROM api_keys
            WHERE key_hash = $1 AND status = 'active' AND (expires_at IS NULL OR expires_at > NOW())
            "#,
        )
        .bind(key_hash)
        .fetch_optional(self.pool)
        .await
        .map_err(|e| CoreError::Internal(format!("Database error looking up api key: {e}")))?;

        Ok(row)
    }

    pub async fn find_by_key_prefix(&self, prefix: &str) -> Result<Option<ApiKey>, CoreError> {
        let row = sqlx::query_as::<_, ApiKey>(
            r#"
            SELECT id, tenant_id, name, key_prefix, key_hash, expires_at, last_used_at, created_at
            FROM api_keys
            WHERE key_prefix = $1 AND status = 'active'
            "#,
        )
        .bind(prefix)
        .fetch_optional(self.pool)
        .await
        .map_err(|e| CoreError::Internal(format!("Database error fetching api key by prefix: {e}")))?;

        Ok(row)
    }

    pub async fn list_by_tenant(&self, tenant_id: Uuid, limit: i64, offset: i64) -> Result<Vec<ApiKey>, CoreError> {
        let rows = sqlx::query_as::<_, ApiKey>(
            r#"
            SELECT id, tenant_id, name, key_prefix, key_hash, expires_at, last_used_at, created_at
            FROM api_keys
            WHERE tenant_id = $1 AND status = 'active'
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(tenant_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(self.pool)
        .await
        .map_err(|e| CoreError::Internal(format!("Database error listing api keys: {e}")))?;

        Ok(rows)
    }

    pub async fn create(
        &self,
        tenant_id: Uuid,
        name: &str,
        key_prefix: &str,
        key_hash: &str,
        expires_at: Option<DateTime<Utc>>,
    ) -> Result<ApiKey, CoreError> {
        let id = Uuid::new_v4();
        let row = sqlx::query_as::<_, ApiKey>(
            r#"
            INSERT INTO api_keys (id, tenant_id, name, key_prefix, key_hash, type, status, expires_at, created_at)
            VALUES ($1, $2, $3, $4, $5, 'secret', 'active', $6, NOW())
            RETURNING id, tenant_id, name, key_prefix, key_hash, expires_at, last_used_at, created_at
            "#,
        )
        .bind(id)
        .bind(tenant_id)
        .bind(name)
        .bind(key_prefix)
        .bind(key_hash)
        .bind(expires_at)
        .fetch_one(self.pool)
        .await
        .map_err(|e| {
            if let Some(db_err) = e.as_database_error() {
                if db_err.code().as_deref() == Some("23505") {
                    return CoreError::Conflict("API key already exists".to_string());
                }
                if db_err.code().as_deref() == Some("23503") {
                    return CoreError::NotFound(format!("Tenant '{tenant_id}' not found"));
                }
            }
            CoreError::Internal(format!("Database error creating api key: {e}"))
        })?;

        Ok(row)
    }

    pub async fn update_last_used(&self, id: Uuid) -> Result<(), CoreError> {
        sqlx::query(
            r#"
            UPDATE api_keys
            SET last_used_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(id)
        .execute(self.pool)
        .await
        .map_err(|e| CoreError::Internal(format!("Database error updating api key last used: {e}")))?;

        Ok(())
    }

    pub async fn revoke(&self, tenant_id: Uuid, id: Uuid) -> Result<(), CoreError> {
        let existing = sqlx::query_scalar::<_, String>(
            r#"
            SELECT status::text FROM api_keys
            WHERE tenant_id = $1 AND id = $2
            "#,
        )
        .bind(tenant_id)
        .bind(id)
        .fetch_optional(self.pool)
        .await
        .map_err(|e| CoreError::Internal(format!("Database error checking api key: {e}")))?;

        match existing {
            None => Err(CoreError::NotFound(format!("API key '{id}' not found"))),
            Some(status) if status == "revoked" => {
                Err(CoreError::Conflict(format!("API key '{id}' is already revoked")))
            }
            Some(_) => {
                sqlx::query(
                    r#"
                    UPDATE api_keys
                    SET status = 'revoked', revoked_at = NOW()
                    WHERE tenant_id = $1 AND id = $2
                    "#,
                )
                .bind(tenant_id)
                .bind(id)
                .execute(self.pool)
                .await
                .map_err(|e| CoreError::Internal(format!("Database error revoking api key: {e}")))?;

                Ok(())
            }
        }
    }

    pub async fn delete(&self, tenant_id: Uuid, id: Uuid) -> Result<(), CoreError> {
        self.revoke(tenant_id, id).await
    }
}
