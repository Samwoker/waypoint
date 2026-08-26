use sqlx::PgPool;
use uuid::Uuid;

use relay_core::error::CoreError;
use crate::models::Source;

#[derive(Clone, Debug)]
pub struct SourceRepository<'a> {
    pub pool: &'a PgPool,
}

impl<'a> SourceRepository<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }

    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<Source>, CoreError> {
        let row = sqlx::query_as::<_, Source>(
            r#"
            SELECT
                id,
                tenant_id,
                name,
                slug,
                description,
                source_type AS provider,
                COALESCE(metadata->>'verification_type', 'none') AS verification_type,
                signing_secret_encrypted AS encrypted_secret,
                (status = 'active') AS is_active,
                (signing_secret_encrypted IS NOT NULL AND signing_secret_encrypted != '') AS has_secret,
                (metadata->>'timestamp_tolerance_secs')::integer AS timestamp_tolerance_secs,
                created_at,
                updated_at
            FROM sources
            WHERE id = $1 AND status != 'deleted'
            "#,
        )
        .bind(id)
        .fetch_optional(self.pool)
        .await
        .map_err(|e| CoreError::Internal(format!("Database error fetching source by id: {e}")))?;

        Ok(row)
    }

    pub async fn find_by_tenant_and_id(&self, tenant_id: Uuid, id: Uuid) -> Result<Option<Source>, CoreError> {
        let row = sqlx::query_as::<_, Source>(
            r#"
            SELECT
                id,
                tenant_id,
                name,
                slug,
                description,
                source_type AS provider,
                COALESCE(metadata->>'verification_type', 'none') AS verification_type,
                signing_secret_encrypted AS encrypted_secret,
                (status = 'active') AS is_active,
                (signing_secret_encrypted IS NOT NULL AND signing_secret_encrypted != '') AS has_secret,
                (metadata->>'timestamp_tolerance_secs')::integer AS timestamp_tolerance_secs,
                created_at,
                updated_at
            FROM sources
            WHERE tenant_id = $1 AND id = $2 AND status != 'deleted'
            "#,
        )
        .bind(tenant_id)
        .bind(id)
        .fetch_optional(self.pool)
        .await
        .map_err(|e| CoreError::Internal(format!("Database error fetching source: {e}")))?;

        Ok(row)
    }

    pub async fn find_by_tenant_and_slug(&self, tenant_id: Uuid, slug: &str) -> Result<Option<Source>, CoreError> {
        let row = sqlx::query_as::<_, Source>(
            r#"
            SELECT
                id,
                tenant_id,
                name,
                slug,
                description,
                source_type AS provider,
                COALESCE(metadata->>'verification_type', 'none') AS verification_type,
                signing_secret_encrypted AS encrypted_secret,
                (status = 'active') AS is_active,
                (signing_secret_encrypted IS NOT NULL AND signing_secret_encrypted != '') AS has_secret,
                (metadata->>'timestamp_tolerance_secs')::integer AS timestamp_tolerance_secs,
                created_at,
                updated_at
            FROM sources
            WHERE tenant_id = $1 AND slug = $2 AND status != 'deleted'
            "#,
        )
        .bind(tenant_id)
        .bind(slug)
        .fetch_optional(self.pool)
        .await
        .map_err(|e| CoreError::Internal(format!("Database error fetching source by slug: {e}")))?;

        Ok(row)
    }

    pub async fn find_by_slug(&self, slug: &str) -> Result<Option<Source>, CoreError> {
        let row = sqlx::query_as::<_, Source>(
            r#"
            SELECT
                id,
                tenant_id,
                name,
                slug,
                description,
                source_type AS provider,
                COALESCE(metadata->>'verification_type', 'none') AS verification_type,
                signing_secret_encrypted AS encrypted_secret,
                (status = 'active') AS is_active,
                (signing_secret_encrypted IS NOT NULL AND signing_secret_encrypted != '') AS has_secret,
                (metadata->>'timestamp_tolerance_secs')::integer AS timestamp_tolerance_secs,
                created_at,
                updated_at
            FROM sources
            WHERE slug = $1 AND status = 'active'
            LIMIT 1
            "#,
        )
        .bind(slug)
        .fetch_optional(self.pool)
        .await
        .map_err(|e| CoreError::Internal(format!("Database error fetching source by slug: {e}")))?;

        Ok(row)
    }

    pub async fn list_by_tenant(&self, tenant_id: Uuid, limit: i64, offset: i64) -> Result<Vec<Source>, CoreError> {
        let rows = sqlx::query_as::<_, Source>(
            r#"
            SELECT
                id,
                tenant_id,
                name,
                slug,
                description,
                source_type AS provider,
                COALESCE(metadata->>'verification_type', 'none') AS verification_type,
                NULL::text AS encrypted_secret,
                (status = 'active') AS is_active,
                (signing_secret_encrypted IS NOT NULL AND signing_secret_encrypted != '') AS has_secret,
                (metadata->>'timestamp_tolerance_secs')::integer AS timestamp_tolerance_secs,
                created_at,
                updated_at
            FROM sources
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
        .map_err(|e| CoreError::Internal(format!("Database error listing sources: {e}")))?;

        Ok(rows)
    }

    pub async fn create(
        &self,
        tenant_id: Uuid,
        name: &str,
        slug: &str,
        description: Option<&str>,
        provider: &str,
        verification_type: &str,
        encrypted_secret: Option<&str>,
    ) -> Result<Source, CoreError> {
        let id = Uuid::new_v4();
        let metadata = serde_json::json!({
            "verification_type": verification_type
        });

        let row = sqlx::query_as::<_, Source>(
            r#"
            INSERT INTO sources (
                id, tenant_id, name, slug, description, source_type, status, signing_secret_encrypted, metadata, created_at, updated_at
            ) VALUES (
                $1, $2, $3, $4, $5, $6, 'active', $7, $8, NOW(), NOW()
            )
            RETURNING
                id,
                tenant_id,
                name,
                slug,
                description,
                source_type AS provider,
                COALESCE(metadata->>'verification_type', 'none') AS verification_type,
                signing_secret_encrypted AS encrypted_secret,
                (status = 'active') AS is_active,
                (signing_secret_encrypted IS NOT NULL AND signing_secret_encrypted != '') AS has_secret,
                (metadata->>'timestamp_tolerance_secs')::integer AS timestamp_tolerance_secs,
                created_at,
                updated_at
            "#,
        )
        .bind(id)
        .bind(tenant_id)
        .bind(name)
        .bind(slug)
        .bind(description)
        .bind(provider)
        .bind(encrypted_secret)
        .bind(metadata)
        .fetch_one(self.pool)
        .await
        .map_err(|e| {
            if let Some(db_err) = e.as_database_error() {
                if db_err.code().as_deref() == Some("23505") {
                    return CoreError::Conflict(format!("Source with slug '{slug}' already exists for this tenant"));
                }
                if db_err.code().as_deref() == Some("23503") {
                    return CoreError::NotFound(format!("Tenant '{tenant_id}' does not exist"));
                }
            }
            CoreError::Internal(format!("Database error creating source: {e}"))
        })?;

        Ok(row)
    }

    pub async fn update(
        &self,
        tenant_id: Uuid,
        id: Uuid,
        name: Option<&str>,
        description: Option<&str>,
        is_active: Option<bool>,
        timestamp_tolerance_secs: Option<i32>,
    ) -> Result<Source, CoreError> {
        let existing = self
            .find_by_tenant_and_id(tenant_id, id)
            .await?
            .ok_or_else(|| CoreError::NotFound(format!("Source '{id}' not found")))?;

        let new_name = name.unwrap_or(&existing.name);
        let new_description = description.or(existing.description.as_deref());
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

        let metadata_update = if let Some(tol) = timestamp_tolerance_secs {
            serde_json::json!({ "timestamp_tolerance_secs": tol })
        } else {
            serde_json::json!({})
        };

        let row = sqlx::query_as::<_, Source>(
            r#"
            UPDATE sources
            SET
                name = $1,
                description = $2,
                status = $3::source_status,
                metadata = COALESCE(metadata, '{}'::jsonb) || $4::jsonb,
                updated_at = NOW()
            WHERE tenant_id = $5 AND id = $6 AND status != 'deleted'
            RETURNING
                id,
                tenant_id,
                name,
                slug,
                description,
                source_type AS provider,
                COALESCE(metadata->>'verification_type', 'none') AS verification_type,
                signing_secret_encrypted AS encrypted_secret,
                (status = 'active') AS is_active,
                (signing_secret_encrypted IS NOT NULL AND signing_secret_encrypted != '') AS has_secret,
                (metadata->>'timestamp_tolerance_secs')::integer AS timestamp_tolerance_secs,
                created_at,
                updated_at
            "#,
        )
        .bind(new_name)
        .bind(new_description)
        .bind(new_status)
        .bind(metadata_update)
        .bind(tenant_id)
        .bind(id)
        .fetch_one(self.pool)
        .await
        .map_err(|e| CoreError::Internal(format!("Database error updating source: {e}")))?;

        Ok(row)
    }

    pub async fn delete(&self, tenant_id: Uuid, id: Uuid) -> Result<(), CoreError> {
        let result = sqlx::query(
            r#"
            UPDATE sources
            SET status = 'deleted', updated_at = NOW()
            WHERE tenant_id = $1 AND id = $2 AND status != 'deleted'
            "#,
        )
        .bind(tenant_id)
        .bind(id)
        .execute(self.pool)
        .await
        .map_err(|e| CoreError::Internal(format!("Database error deleting source: {e}")))?;

        if result.rows_affected() == 0 {
            return Err(CoreError::NotFound(format!("Source '{id}' not found")));
        }

        Ok(())
    }

    pub async fn rotate_secret(
        &self,
        tenant_id: Uuid,
        id: Uuid,
        encrypted_secret: &str,
    ) -> Result<Source, CoreError> {
        let row = sqlx::query_as::<_, Source>(
            r#"
            UPDATE sources
            SET
                signing_secret_encrypted = $1,
                updated_at = NOW()
            WHERE tenant_id = $2 AND id = $3 AND status != 'deleted'
            RETURNING
                id,
                tenant_id,
                name,
                slug,
                description,
                source_type AS provider,
                COALESCE(metadata->>'verification_type', 'none') AS verification_type,
                signing_secret_encrypted AS encrypted_secret,
                (status = 'active') AS is_active,
                (signing_secret_encrypted IS NOT NULL AND signing_secret_encrypted != '') AS has_secret,
                (metadata->>'timestamp_tolerance_secs')::integer AS timestamp_tolerance_secs,
                created_at,
                updated_at
            "#,
        )
        .bind(encrypted_secret)
        .bind(tenant_id)
        .bind(id)
        .fetch_optional(self.pool)
        .await
        .map_err(|e| CoreError::Internal(format!("Database error rotating source secret: {e}")))?
        .ok_or_else(|| CoreError::NotFound(format!("Source '{id}' not found")))?;

        Ok(row)
    }
}
