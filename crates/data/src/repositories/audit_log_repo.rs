use sqlx::PgPool;
use uuid::Uuid;

use relay_core::error::CoreError;
use crate::models::AuditLog;

#[derive(Clone, Debug)]
pub struct AuditLogRepository<'a> {
    pub pool: &'a PgPool,
}

impl<'a> AuditLogRepository<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }

    pub async fn create(
        &self,
        tenant_id: Uuid,
        user_id: Option<Uuid>,
        action: &str,
        resource_type: Option<&str>,
        resource_id: Option<Uuid>,
        metadata: serde_json::Value,
    ) -> Result<AuditLog, CoreError> {
        let id = Uuid::new_v4();

        let row = sqlx::query_as::<_, AuditLog>(
            r#"
            INSERT INTO audit_logs (
                id, tenant_id, user_id, action, resource_type, resource_id, metadata, created_at
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, NOW()
            )
            RETURNING
                id,
                tenant_id,
                user_id,
                action,
                resource_type,
                resource_id,
                metadata,
                created_at
            "#,
        )
        .bind(id)
        .bind(tenant_id)
        .bind(user_id)
        .bind(action)
        .bind(resource_type)
        .bind(resource_id)
        .bind(metadata)
        .fetch_one(self.pool)
        .await
        .map_err(|e| CoreError::Internal(format!("Database error creating audit log: {e}")))?;

        Ok(row)
    }

    pub async fn list_by_tenant(
        &self,
        tenant_id: Uuid,
        action: Option<&str>,
        resource_type: Option<&str>,
        resource_id: Option<Uuid>,
        user_id: Option<Uuid>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<AuditLog>, CoreError> {
        let rows = sqlx::query_as::<_, AuditLog>(
            r#"
            SELECT
                id,
                tenant_id,
                user_id,
                action,
                resource_type,
                resource_id,
                metadata,
                created_at
            FROM audit_logs
            WHERE tenant_id = $1
              AND ($2::TEXT IS NULL OR action = $2)
              AND ($3::TEXT IS NULL OR resource_type = $3)
              AND ($4::UUID IS NULL OR resource_id = $4)
              AND ($5::UUID IS NULL OR user_id = $5)
            ORDER BY created_at DESC
            LIMIT $6 OFFSET $7
            "#,
        )
        .bind(tenant_id)
        .bind(action)
        .bind(resource_type)
        .bind(resource_id)
        .bind(user_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(self.pool)
        .await
        .map_err(|e| CoreError::Internal(format!("Database error listing audit logs: {e}")))?;

        Ok(rows)
    }
}
