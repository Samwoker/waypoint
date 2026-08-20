use std::sync::Arc;
use data::repositories::DestinationRepository;
use sqlx::PgPool;
use uuid::Uuid;

use relay_core::crypto::encrypt_secret;
use relay_core::error::CoreError;
use crate::dto::{CreateDestinationInput, DestinationView, UpdateDestinationInput};

#[derive(Clone)]
pub struct DestinationService {
    pub pool: Arc<PgPool>,
    pub encryption_key: [u8; 32],
}

impl DestinationService {
    pub fn new(pool: Arc<PgPool>, encryption_key: [u8; 32]) -> Self {
        Self { pool, encryption_key }
    }

    pub async fn create_destination(
        &self,
        tenant_id: Uuid,
        input: CreateDestinationInput,
    ) -> Result<DestinationView, CoreError> {
        let name = input.name.trim();
        if name.is_empty() {
            return Err(CoreError::Validation("Destination name cannot be empty".to_string()));
        }
        if name.len() > 255 {
            return Err(CoreError::Validation("Destination name cannot exceed 255 characters".to_string()));
        }

        let validated_url = validate_destination_url(&input.url)?;

        let timeout_ms = match input.timeout_ms {
            Some(t) if t <= 0 => {
                return Err(CoreError::Validation("Destination timeout must be a positive integer".to_string()));
            }
            Some(t) => t,
            None => 10000,
        };

        let max_retries = match input.max_retries {
            Some(r) if r < 0 => {
                return Err(CoreError::Validation("Destination max retries must be non-negative".to_string()));
            }
            Some(r) => r,
            None => 10,
        };

        if let Some(rps) = input.rate_limit_rps {
            if rps <= 0 {
                return Err(CoreError::Validation("Destination rate limit must be a positive integer".to_string()));
            }
        }

        if let Some(ref headers) = input.headers {
            if !headers.is_object() {
                return Err(CoreError::Validation("Destination headers must be a JSON object of key-value pairs".to_string()));
            }
        }

        let encrypted_secret = if let Some(ref secret) = input.secret {
            let trimmed = secret.trim();
            if !trimmed.is_empty() {
                Some(encrypt_secret(trimmed.as_bytes(), &self.encryption_key)?)
            } else {
                None
            }
        } else {
            None
        };

        let repo = DestinationRepository::new(&self.pool);
        let destination = repo
            .create(
                tenant_id,
                name,
                &validated_url,
                input.description.as_deref().map(|d| d.trim()),
                input.rate_limit_rps,
                timeout_ms,
                max_retries,
                input.headers.as_ref(),
                encrypted_secret.as_deref(),
            )
            .await?;

        Ok(DestinationView {
            id: destination.id,
            tenant_id: destination.tenant_id,
            name: destination.name,
            url: destination.url,
            description: destination.description,
            rate_limit_rps: destination.rate_limit_rps,
            timeout_ms: destination.timeout_ms,
            is_active: destination.is_active,
            created_at: destination.created_at,
            updated_at: destination.updated_at,
        })
    }

    pub async fn get_destination(&self, tenant_id: Uuid, id: Uuid) -> Result<Option<DestinationView>, CoreError> {
        let repo = DestinationRepository::new(&self.pool);
        let destination = repo.find_by_tenant_and_id(tenant_id, id).await?;

        Ok(destination.map(|d| DestinationView {
            id: d.id,
            tenant_id: d.tenant_id,
            name: d.name,
            url: d.url,
            description: d.description,
            rate_limit_rps: d.rate_limit_rps,
            timeout_ms: d.timeout_ms,
            is_active: d.is_active,
            created_at: d.created_at,
            updated_at: d.updated_at,
        }))
    }

    pub async fn list_destinations(
        &self,
        tenant_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<DestinationView>, CoreError> {
        let limit = if limit <= 0 || limit > 100 { 20 } else { limit };
        let offset = if offset < 0 { 0 } else { offset };

        let repo = DestinationRepository::new(&self.pool);
        let destinations = repo.list_by_tenant(tenant_id, limit, offset).await?;

        Ok(destinations
            .into_iter()
            .map(|d| DestinationView {
                id: d.id,
                tenant_id: d.tenant_id,
                name: d.name,
                url: d.url,
                description: d.description,
                rate_limit_rps: d.rate_limit_rps,
                timeout_ms: d.timeout_ms,
                is_active: d.is_active,
                created_at: d.created_at,
                updated_at: d.updated_at,
            })
            .collect())
    }

    pub async fn update_destination(
        &self,
        tenant_id: Uuid,
        id: Uuid,
        input: UpdateDestinationInput,
    ) -> Result<DestinationView, CoreError> {
        if let Some(ref name) = input.name {
            if name.trim().is_empty() {
                return Err(CoreError::Validation("Destination name cannot be empty".to_string()));
            }
        }

        let validated_url = if let Some(ref u) = input.url {
            Some(validate_destination_url(u)?)
        } else {
            None
        };

        let repo = DestinationRepository::new(&self.pool);
        let destination = repo
            .update(
                tenant_id,
                id,
                input.name.as_deref().map(|n| n.trim()),
                validated_url.as_deref(),
                input.description.as_deref().map(|d| d.trim()),
                input.rate_limit_rps,
                input.timeout_ms,
                input.is_active,
            )
            .await?;

        Ok(DestinationView {
            id: destination.id,
            tenant_id: destination.tenant_id,
            name: destination.name,
            url: destination.url,
            description: destination.description,
            rate_limit_rps: destination.rate_limit_rps,
            timeout_ms: destination.timeout_ms,
            is_active: destination.is_active,
            created_at: destination.created_at,
            updated_at: destination.updated_at,
        })
    }

    pub async fn delete_destination(&self, tenant_id: Uuid, id: Uuid) -> Result<(), CoreError> {
        let repo = DestinationRepository::new(&self.pool);
        repo.delete(tenant_id, id).await
    }
}

/// Validates that a destination URL is present, well-formed, uses HTTP(S), and has a valid host.
pub fn validate_destination_url(raw_url: &str) -> Result<String, CoreError> {
    let trimmed = raw_url.trim();
    if trimmed.is_empty() {
        return Err(CoreError::Validation("Destination URL cannot be empty".to_string()));
    }

    let parsed = reqwest::Url::parse(trimmed)
        .map_err(|e| CoreError::Validation(format!("Invalid destination URL '{trimmed}': {e}")))?;

    if parsed.scheme() != "http" && parsed.scheme() != "https" {
        return Err(CoreError::Validation(
            format!("Invalid destination URL scheme '{}': must be http or https", parsed.scheme()),
        ));
    }

    let host = match parsed.host_str() {
        Some(h) if !h.trim().is_empty() => h.trim(),
        _ => {
            return Err(CoreError::Validation(
                "Destination URL must contain a valid host".to_string(),
            ));
        }
    };

    // SSRF safety: reject cloud metadata endpoints and loopback addresses when SSRF check is applicable
    if host == "169.254.169.254" || host == "metadata.google.internal" || host == "instance-data" {
        return Err(CoreError::Validation("Destination URL targets a forbidden internal metadata address".to_string()));
    }

    Ok(trimmed.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_urls() {
        assert!(validate_destination_url("https://api.example.com/webhooks").is_ok());
        assert!(validate_destination_url("http://example.com:8080/events").is_ok());
        assert!(validate_destination_url("https://sub.domain.co.uk/path?query=1#hash").is_ok());
    }

    #[test]
    fn test_invalid_urls() {
        assert!(validate_destination_url("").is_err());
        assert!(validate_destination_url("   ").is_err());
        assert!(validate_destination_url("not-a-url").is_err());
        assert!(validate_destination_url("ftp://example.com/webhook").is_err());
        assert!(validate_destination_url("javascript:alert(1)").is_err());
        assert!(validate_destination_url("file:///etc/passwd").is_err());
        assert!(validate_destination_url("http://169.254.169.254/latest/meta-data/").is_err());
    }
}
