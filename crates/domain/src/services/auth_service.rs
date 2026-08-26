use std::sync::Arc;
use data::repositories::{ApiKeyRepository, AuditLogRepository, TenantRepository, UserRepository};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use uuid::Uuid;

use relay_core::crypto::{generate_secret, hash_password, verify_password};
use relay_core::error::CoreError;
use crate::dto::{ApiKeyCreatedView, ApiKeyView, AuthTokenView, CreateApiKeyInput, LoginInput, RegisterInput, UserView};

#[derive(Debug, Serialize, Deserialize)]
struct JwtClaims {
    pub sub: String,
    pub tenant_id: Uuid,
    pub role: Option<String>,
    pub is_admin: Option<bool>,
    pub scope: Option<String>,
    pub exp: usize,
}

#[derive(Clone)]
pub struct AuthService {
    pub pool: Arc<PgPool>,
    pub jwt_secret: String,
}

impl AuthService {
    pub fn new(pool: Arc<PgPool>, jwt_secret: String) -> Self {
        Self { pool, jwt_secret }
    }

    pub async fn register(&self, input: RegisterInput) -> Result<AuthTokenView, CoreError> {
        let email = input.email.trim();
        if email.is_empty() || !email.contains('@') {
            return Err(CoreError::Validation("A valid email is required".to_string()));
        }
        if input.password.len() < 6 {
            return Err(CoreError::Validation("Password must be at least 6 characters".to_string()));
        }
        let tenant_name = input.tenant_name.trim();
        if tenant_name.is_empty() {
            return Err(CoreError::Validation("Organization/Tenant name cannot be empty".to_string()));
        }

        let user_repo = UserRepository::new(&self.pool);
        if let Some(_) = user_repo.find_by_email(email).await? {
            return Err(CoreError::Conflict(format!("User with email '{email}' already exists")));
        }

        // Generate URL slug from tenant name
        let clean_slug = tenant_name
            .to_lowercase()
            .replace(|c: char| !c.is_alphanumeric(), "-");
        let slug_trimmed = clean_slug.trim_matches('-');
        let random_suffix = generate_secret(4);
        let slug = if slug_trimmed.is_empty() {
            format!("tenant-{random_suffix}")
        } else {
            format!("{slug_trimmed}-{random_suffix}")
        };

        // Create Tenant
        let tenant_repo = TenantRepository::new(&self.pool);
        let tenant = tenant_repo.create(tenant_name, &slug).await?;

        // Hash password and create User with "owner" / "admin" role
        let password_hash = hash_password(&input.password)?;
        let user = user_repo.create(tenant.id, email, &password_hash, "owner").await?;

        let is_admin = true;
        let now = chrono::Utc::now().timestamp() as usize;
        let expires_in: i64 = 86400; // 24 hours

        let access_claims = JwtClaims {
            sub: user.id.to_string(),
            tenant_id: tenant.id,
            role: Some("owner".to_string()),
            is_admin: Some(is_admin),
            scope: Some("full".to_string()),
            exp: now + (expires_in as usize),
        };

        let access_token = encode(
            &Header::default(),
            &access_claims,
            &EncodingKey::from_secret(self.jwt_secret.as_bytes()),
        )
        .map_err(|e| CoreError::Internal(format!("Failed to generate JWT: {e}")))?;

        let refresh_claims = JwtClaims {
            sub: format!("refresh:{}", user.id),
            tenant_id: tenant.id,
            role: Some("owner".to_string()),
            is_admin: Some(is_admin),
            scope: Some("refresh".to_string()),
            exp: now + 30 * 86400, // 30 days
        };

        let refresh_token = encode(
            &Header::default(),
            &refresh_claims,
            &EncodingKey::from_secret(self.jwt_secret.as_bytes()),
        )
        .map_err(|e| CoreError::Internal(format!("Failed to generate refresh JWT: {e}")))?;

        Ok(AuthTokenView {
            access_token,
            token_type: "Bearer".to_string(),
            expires_in,
            refresh_token: Some(refresh_token),
        })
    }

    pub async fn login(&self, input: LoginInput) -> Result<AuthTokenView, CoreError> {
        let email = input.email.trim();
        if email.is_empty() {
            return Err(CoreError::Validation("Email cannot be empty".to_string()));
        }
        if input.password.is_empty() {
            return Err(CoreError::Validation("Password cannot be empty".to_string()));
        }

        let user_repo = UserRepository::new(&self.pool);
        let user = user_repo
            .find_by_email(email)
            .await?
            .ok_or_else(|| CoreError::Unauthorized("Invalid email or password".to_string()))?;

        let is_valid = verify_password(&input.password, &user.password_hash)?;
        if !is_valid {
            return Err(CoreError::Unauthorized("Invalid email or password".to_string()));
        }

        // Update last login timestamp
        let _ = user_repo.update_last_login(user.id).await;

        let is_admin = user.role == "admin" || user.role == "owner";
        let now = chrono::Utc::now().timestamp() as usize;
        let expires_in: i64 = 86400; // 24 hours

        let access_claims = JwtClaims {
            sub: user.id.to_string(),
            tenant_id: user.tenant_id,
            role: Some(user.role.clone()),
            is_admin: Some(is_admin),
            scope: Some("full".to_string()),
            exp: now + (expires_in as usize),
        };

        let access_token = encode(
            &Header::default(),
            &access_claims,
            &EncodingKey::from_secret(self.jwt_secret.as_bytes()),
        )
        .map_err(|e| CoreError::Internal(format!("Failed to generate JWT: {e}")))?;

        let refresh_claims = JwtClaims {
            sub: format!("refresh:{}", user.id),
            tenant_id: user.tenant_id,
            role: Some(user.role),
            is_admin: Some(is_admin),
            scope: Some("refresh".to_string()),
            exp: now + 30 * 86400, // 30 days
        };

        let refresh_token = encode(
            &Header::default(),
            &refresh_claims,
            &EncodingKey::from_secret(self.jwt_secret.as_bytes()),
        )
        .map_err(|e| CoreError::Internal(format!("Failed to generate refresh JWT: {e}")))?;

        Ok(AuthTokenView {
            access_token,
            token_type: "Bearer".to_string(),
            expires_in,
            refresh_token: Some(refresh_token),
        })
    }

    pub async fn refresh_token(&self, refresh_token_str: &str) -> Result<AuthTokenView, CoreError> {
        let token_data = decode::<JwtClaims>(
            refresh_token_str,
            &DecodingKey::from_secret(self.jwt_secret.as_bytes()),
            &Validation::default(),
        )
        .map_err(|_| CoreError::Unauthorized("Invalid or expired refresh token".to_string()))?;

        let user_id_str = token_data
            .claims
            .sub
            .strip_prefix("refresh:")
            .unwrap_or(&token_data.claims.sub);

        let user_id = Uuid::parse_str(user_id_str)
            .map_err(|_| CoreError::Unauthorized("Invalid token subject".to_string()))?;

        let user_repo = UserRepository::new(&self.pool);
        let user = user_repo
            .find_by_id(user_id)
            .await?
            .ok_or_else(|| CoreError::Unauthorized("User not found or inactive".to_string()))?;

        let is_admin = user.role == "admin" || user.role == "owner";
        let now = chrono::Utc::now().timestamp() as usize;
        let expires_in: i64 = 86400;

        let access_claims = JwtClaims {
            sub: user.id.to_string(),
            tenant_id: user.tenant_id,
            role: Some(user.role),
            is_admin: Some(is_admin),
            scope: Some("full".to_string()),
            exp: now + (expires_in as usize),
        };

        let access_token = encode(
            &Header::default(),
            &access_claims,
            &EncodingKey::from_secret(self.jwt_secret.as_bytes()),
        )
        .map_err(|e| CoreError::Internal(format!("Failed to generate JWT: {e}")))?;

        Ok(AuthTokenView {
            access_token,
            token_type: "Bearer".to_string(),
            expires_in,
            refresh_token: Some(refresh_token_str.to_string()),
        })
    }

    pub async fn get_user_profile(&self, user_id: Uuid) -> Result<UserView, CoreError> {
        let user_repo = UserRepository::new(&self.pool);
        let user = user_repo
            .find_by_id(user_id)
            .await?
            .ok_or_else(|| CoreError::NotFound("User profile not found".to_string()))?;

        Ok(UserView {
            id: user.id,
            tenant_id: user.tenant_id,
            email: user.email,
            role: user.role,
            created_at: user.created_at,
            updated_at: user.updated_at,
        })
    }

    pub async fn get_api_key(&self, tenant_id: Uuid, key_id: Uuid) -> Result<ApiKeyView, CoreError> {
        let repo = ApiKeyRepository::new(&self.pool);
        let key = repo
            .find_by_tenant_and_id(tenant_id, key_id)
            .await?
            .ok_or_else(|| CoreError::NotFound(format!("API key '{key_id}' not found")))?;

        Ok(ApiKeyView {
            id: key.id,
            tenant_id: key.tenant_id,
            name: key.name,
            key_prefix: key.key_prefix,
            expires_at: key.expires_at,
            last_used_at: key.last_used_at,
            created_at: key.created_at,
        })
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
