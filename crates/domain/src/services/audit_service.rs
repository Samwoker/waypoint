use std::sync::Arc;
use sqlx::PgPool;
use uuid::Uuid;

use data::repositories::AuditLogRepository;
use relay_core::error::CoreError;
use crate::dto::AuditLogView;

#[derive(Clone)]
pub struct AuditService {
    pub pool: Arc<PgPool>,
}

impl AuditService {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }

    pub async fn create_audit_log(
        &self,
        tenant_id: Uuid,
        user_id: Option<Uuid>,
        action: &str,
        resource_type: Option<&str>,
        resource_id: Option<Uuid>,
        metadata: serde_json::Value,
    ) -> Result<AuditLogView, CoreError> {
        let repo = AuditLogRepository::new(&self.pool);
        let log = repo
            .create(tenant_id, user_id, action, resource_type, resource_id, metadata)
            .await?;

        Ok(AuditLogView {
            id: log.id,
            tenant_id: log.tenant_id,
            user_id: log.user_id,
            action: log.action,
            resource_type: log.resource_type,
            resource_id: log.resource_id,
            metadata: log.metadata,
            created_at: log.created_at,
        })
    }

    pub async fn list_audit_logs(
        &self,
        tenant_id: Uuid,
        action: Option<&str>,
        resource_type: Option<&str>,
        resource_id: Option<Uuid>,
        user_id: Option<Uuid>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<AuditLogView>, CoreError> {
        let limit = if limit <= 0 || limit > 100 { 20 } else { limit };
        let offset = if offset < 0 { 0 } else { offset };

        let repo = AuditLogRepository::new(&self.pool);
        let logs = repo
            .list_by_tenant(tenant_id, action, resource_type, resource_id, user_id, limit, offset)
            .await?;

        Ok(logs
            .into_iter()
            .map(|l| AuditLogView {
                id: l.id,
                tenant_id: l.tenant_id,
                user_id: l.user_id,
                action: l.action,
                resource_type: l.resource_type,
                resource_id: l.resource_id,
                metadata: l.metadata,
                created_at: l.created_at,
            })
            .collect())
    }
}
