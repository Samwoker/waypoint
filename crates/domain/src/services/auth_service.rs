use std::sync::Arc;
use data::repositories::{ApiKeyRepository, AuditLogRepository};
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use uuid::Uuid;

use relay_core::crypto::generate_secret;
use relay_core::error::CoreError;
use crate::dto::{ApiKeyCreatedView, ApiKeyView, AuthTokenView, CreateApiKeyInput, LoginInput};

#[derive(Clone)]
pub struct AuthService {
    pub pool: Arc<PgPool>,
    pub jwt_secret: String,
}

impl AuthService {
    pub fn new(pool: Arc<PgPool>, jwt_secret: String) -> Self {
        Self { pool, jwt_secret }
    }

    pub async fn login(&self, _input: LoginInput) -> Result<AuthTokenView, CoreError> {
        Err(CoreError::Validation("Password login not configured in this environment".to_string()))
    }

    pub async fn validate_api_key(&self, raw_key: &str) -> Result<Uuid, CoreError> {
        let (tenant_id, _) = self.validate_api_key_with_scope(raw_key).await?;
        Ok(tenant_id)
    }

    pub async fn validate_api_key_with_scope(&self, raw_key: &str) -> Result<(Uuid, String), CoreError> {
        let trimmed = raw_key.trim();
        if trimmed.is_empty() {
            return Err(CoreError::Unauthorized("API key cannot be empty".to_string()));
        }

        let mut hasher = Sha256::new();
        hasher.update(trimmed.as_bytes());
        let key_hash = hex::encode(hasher.finalize());

        let repo = ApiKeyRepository::new(&self.pool);
        let api_key = repo
            .find_by_key_hash(&key_hash)
            .await?
            .ok_or_else(|| CoreError::Unauthorized("Invalid or expired API key".to_string()))?;

        // Update last_used_at in the background / asynchronously
        let _ = repo.update_last_used(api_key.id).await;

        let scope = if api_key.name.to_lowercase().contains("read_only")
            || api_key.key_prefix.contains("_ro_")
            || trimmed.starts_with("rc_ro_")
            || trimmed.contains("_ro_")
        {
            "read_only".to_string()
        } else {
            "full".to_string()
        };

        Ok((api_key.tenant_id, scope))
    }

    pub async fn create_api_key(
        &self,
        tenant_id: Uuid,
        input: CreateApiKeyInput,
    ) -> Result<ApiKeyCreatedView, CoreError> {
        let name = input.name.trim();
        if name.is_empty() {
            return Err(CoreError::Validation("API key name cannot be empty".to_string()));
        }

        let random_part = generate_secret(24);
        let raw_key = format!("rc_live_{random_part}");
        let key_prefix = raw_key[..12].to_string();

        let mut hasher = Sha256::new();
        hasher.update(raw_key.as_bytes());
        let key_hash = hex::encode(hasher.finalize());

        let repo = ApiKeyRepository::new(&self.pool);
        let api_key = repo
            .create(tenant_id, name, &key_prefix, &key_hash, input.expires_at)
            .await?;

        // Create audit log entry
        let audit_repo = AuditLogRepository::new(&self.pool);
        let _ = audit_repo
            .create(
                tenant_id,
                None,
                "api_key.created",
                Some("api_key"),
                Some(api_key.id),
                serde_json::json!({ "name": name, "key_prefix": key_prefix }),
            )
            .await;

        Ok(ApiKeyCreatedView {
            id: api_key.id,
            name: api_key.name,
            raw_key,
            key_prefix: api_key.key_prefix,
            expires_at: api_key.expires_at,
            created_at: api_key.created_at,
        })
    }

    pub async fn list_api_keys(
        &self,
        tenant_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<ApiKeyView>, CoreError> {
        let limit = if limit <= 0 || limit > 100 { 20 } else { limit };
        let offset = if offset < 0 { 0 } else { offset };

        let repo = ApiKeyRepository::new(&self.pool);
        let keys = repo.list_by_tenant(tenant_id, limit, offset).await?;

        Ok(keys
            .into_iter()
            .map(|k| ApiKeyView {
                id: k.id,
                tenant_id: k.tenant_id,
                name: k.name,
                key_prefix: k.key_prefix,
                expires_at: k.expires_at,
                last_used_at: k.last_used_at,
                created_at: k.created_at,
            })
            .collect())
    }

    pub async fn revoke_api_key(&self, tenant_id: Uuid, key_id: Uuid) -> Result<(), CoreError> {
        let repo = ApiKeyRepository::new(&self.pool);
        repo.revoke(tenant_id, key_id).await?;

        // Create audit log entry for revocation
        let audit_repo = AuditLogRepository::new(&self.pool);
        let _ = audit_repo
            .create(
                tenant_id,
                None,
                "api_key.revoked",
                Some("api_key"),
                Some(key_id),
                serde_json::json!({}),
            )
            .await;

        Ok(())
    }
}
