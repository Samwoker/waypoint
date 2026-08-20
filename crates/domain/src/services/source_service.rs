use std::sync::Arc;
use data::repositories::SourceRepository;
use sqlx::PgPool;
use uuid::Uuid;

use relay_core::crypto::{encrypt_secret, generate_secret_base64};
use relay_core::error::CoreError;
use crate::dto::{CreateSourceInput, SourceView, UpdateSourceInput};

#[derive(Clone)]
pub struct SourceService {
    pub pool: Arc<PgPool>,
    pub encryption_key: [u8; 32],
}

impl SourceService {
    pub fn new(pool: Arc<PgPool>, encryption_key: [u8; 32]) -> Self {
        Self { pool, encryption_key }
    }

    pub async fn create_source(
        &self,
        tenant_id: Uuid,
        input: CreateSourceInput,
    ) -> Result<SourceView, CoreError> {
        let name = input.name.trim();
        if name.is_empty() {
            return Err(CoreError::Validation("Source name cannot be empty".to_string()));
        }
        if name.len() > 255 {
            return Err(CoreError::Validation(
                "Source name cannot exceed 255 characters".to_string(),
            ));
        }

        let slug = input.slug.trim();
        if slug.is_empty() {
            return Err(CoreError::Validation("Source slug cannot be empty".to_string()));
        }
        if slug.len() > 100 {
            return Err(CoreError::Validation(
                "Source slug cannot exceed 100 characters".to_string(),
            ));
        }

        // Validate slug characters: lowercase alphanumeric, hyphens, and underscores
        if !slug.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '_') {
            return Err(CoreError::Validation(
                format!("Invalid slug '{slug}': slug must contain only lowercase alphanumeric characters, dashes, and underscores")
            ));
        }
        if slug.starts_with('-') || slug.starts_with('_') || slug.ends_with('-') || slug.ends_with('_') {
            return Err(CoreError::Validation(
                "Source slug cannot start or end with a hyphen or underscore".to_string(),
            ));
        }

        let provider = if input.provider.trim().is_empty() {
            "generic".to_string()
        } else {
            input.provider.trim().to_string()
        };

        let verification_type = if input.verification_type.trim().is_empty() {
            "none".to_string()
        } else {
            input.verification_type.trim().to_string()
        };

        let plaintext_secret = match input.secret.as_deref().map(|s| s.trim()).filter(|s| !s.is_empty()) {
            Some(s) => s.to_string(),
            None => generate_secret_base64(32),
        };

        let encrypted_secret = encrypt_secret(plaintext_secret.as_bytes(), &self.encryption_key)?;

        let repo = SourceRepository::new(&self.pool);
        let source = repo
            .create(
                tenant_id,
                name,
                slug,
                input.description.as_deref().map(|d| d.trim()),
                &provider,
                &verification_type,
                Some(&encrypted_secret),
            )
            .await?;

        Ok(SourceView {
            id: source.id,
            tenant_id: source.tenant_id,
            name: source.name,
            slug: source.slug,
            description: source.description,
            provider: source.provider,
            verification_type: source.verification_type,
            is_active: source.is_active,
            has_secret: true,
            timestamp_tolerance_secs: source.timestamp_tolerance_secs,
            secret: Some(plaintext_secret),
            created_at: source.created_at,
            updated_at: source.updated_at,
        })
    }

    pub async fn get_source(&self, tenant_id: Uuid, id: Uuid) -> Result<Option<SourceView>, CoreError> {
        let repo = SourceRepository::new(&self.pool);
        let source = repo.find_by_tenant_and_id(tenant_id, id).await?;

        Ok(source.map(|s| SourceView {
            id: s.id,
            tenant_id: s.tenant_id,
            name: s.name,
            slug: s.slug,
            description: s.description,
            provider: s.provider,
            verification_type: s.verification_type,
            is_active: s.is_active,
            has_secret: s.has_secret,
            timestamp_tolerance_secs: s.timestamp_tolerance_secs,
            secret: None,
            created_at: s.created_at,
            updated_at: s.updated_at,
        }))
    }

    pub async fn list_sources(
        &self,
        tenant_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<SourceView>, CoreError> {
        let limit = if limit <= 0 || limit > 100 { 20 } else { limit };
        let offset = if offset < 0 { 0 } else { offset };

        let repo = SourceRepository::new(&self.pool);
        let sources = repo.list_by_tenant(tenant_id, limit, offset).await?;

        Ok(sources
            .into_iter()
            .map(|s| SourceView {
                id: s.id,
                tenant_id: s.tenant_id,
                name: s.name,
                slug: s.slug,
                description: s.description,
                provider: s.provider,
                verification_type: s.verification_type,
                is_active: s.is_active,
                has_secret: s.has_secret,
                timestamp_tolerance_secs: s.timestamp_tolerance_secs,
                secret: None,
                created_at: s.created_at,
                updated_at: s.updated_at,
            })
            .collect())
    }

    pub async fn update_source(
        &self,
        tenant_id: Uuid,
        id: Uuid,
        input: UpdateSourceInput,
    ) -> Result<SourceView, CoreError> {
        if input.secret.is_some() {
            return Err(CoreError::Validation(
                "Updating secret directly is not allowed. Use the secret rotation endpoint.".to_string(),
            ));
        }

        if let Some(ref name) = input.name {
            if name.trim().is_empty() {
                return Err(CoreError::Validation("Source name cannot be empty".to_string()));
            }
        }

        if let Some(tolerance) = input.timestamp_tolerance_secs {
            if tolerance < 0 || tolerance > 86400 {
                return Err(CoreError::Validation(
                    "timestamp_tolerance_secs must be between 0 and 86400 seconds (24h)".to_string(),
                ));
            }
        }

        let repo = SourceRepository::new(&self.pool);
        let source = repo
            .update(
                tenant_id,
                id,
                input.name.as_deref().map(|n| n.trim()),
                input.description.as_deref().map(|d| d.trim()),
                input.is_active,
                input.timestamp_tolerance_secs,
            )
            .await?;

        Ok(SourceView {
            id: source.id,
            tenant_id: source.tenant_id,
            name: source.name,
            slug: source.slug,
            description: source.description,
            provider: source.provider,
            verification_type: source.verification_type,
            is_active: source.is_active,
            has_secret: source.has_secret,
            timestamp_tolerance_secs: source.timestamp_tolerance_secs,
            secret: None,
            created_at: source.created_at,
            updated_at: source.updated_at,
        })
    }

    pub async fn delete_source(&self, tenant_id: Uuid, id: Uuid) -> Result<(), CoreError> {
        let repo = SourceRepository::new(&self.pool);
        repo.delete(tenant_id, id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use relay_core::error::CoreError;

    #[tokio::test]
    async fn test_create_source_validation_rejects_empty_name() {
        let pool = Arc::new(
            sqlx::postgres::PgPoolOptions::new()
                .connect_lazy("postgres://invalid:invalid@localhost/db")
                .unwrap(),
        );
        let service = SourceService::new(pool, [0u8; 32]);

        let res = service
            .create_source(
                Uuid::new_v4(),
                CreateSourceInput {
                    name: "   ".to_string(),
                    slug: "valid-slug".to_string(),
                    description: None,
                    provider: "generic".to_string(),
                    verification_type: "none".to_string(),
                    secret: None,
                },
            )
            .await;

        assert!(matches!(res, Err(CoreError::Validation(_))));
    }

    #[tokio::test]
    async fn test_create_source_validation_rejects_invalid_slug() {
        let pool = Arc::new(
            sqlx::postgres::PgPoolOptions::new()
                .connect_lazy("postgres://invalid:invalid@localhost/db")
                .unwrap(),
        );
        let service = SourceService::new(pool, [0u8; 32]);

        let test_cases = vec![
            "",
            "   ",
            "UPPERCASE",
            "slug with space",
            "slug#with@special",
            "-leading-dash",
            "trailing-dash-",
        ];

        for slug in test_cases {
            let res = service
                .create_source(
                    Uuid::new_v4(),
                    CreateSourceInput {
                        name: "Valid Name".to_string(),
                        slug: slug.to_string(),
                        description: None,
                        provider: "generic".to_string(),
                        verification_type: "none".to_string(),
                        secret: None,
                    },
                )
                .await;

            assert!(
                matches!(res, Err(CoreError::Validation(_))),
                "Failed to reject invalid slug: '{slug}'"
            );
        }
    }
}
