use std::sync::Arc;
use std::time::Duration;
use data::models::Destination;
use data::repositories::DestinationRepository;
use sqlx::PgPool;
use uuid::Uuid;

use relay_core::crypto::{decrypt_secret, encrypt_secret, generate_secret_base64, sign_hmac_sha256};
use relay_core::error::CoreError;
use crate::dto::{
    CreateDestinationInput, DestinationHealthView, DestinationView, TestDestinationResponse,
    UpdateDestinationInput,
};

#[derive(Clone)]
pub struct DestinationService {
    pub pool: Arc<PgPool>,
    pub encryption_key: [u8; 32],
    pub environment: String,
    pub http_client: reqwest::Client,
}

impl DestinationService {
    pub fn new(pool: Arc<PgPool>, encryption_key: [u8; 32]) -> Self {
        Self {
            pool,
            encryption_key,
            environment: "development".to_string(),
            http_client: reqwest::Client::new(),
        }
    }

    pub fn with_config(
        pool: Arc<PgPool>,
        encryption_key: [u8; 32],
        environment: String,
        http_client: reqwest::Client,
    ) -> Self {
        Self {
            pool,
            encryption_key,
            environment,
            http_client,
        }
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

        let validated_url = validate_destination_url_with_mode(&input.url, &self.environment)?;

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

        let plaintext_secret = match input.secret {
            Some(ref s) if !s.trim().is_empty() => s.trim().to_string(),
            _ => generate_secret_base64(32),
        };

        let encrypted_secret = encrypt_secret(plaintext_secret.as_bytes(), &self.encryption_key)?;

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
                Some(&encrypted_secret),
            )
            .await?;

        Ok(DestinationView {
            id: destination.id,
            tenant_id: destination.tenant_id,
            name: destination.name,
            url: destination.url,
            description: destination.description,
            status: destination.status,
            is_active: destination.is_active,
            consecutive_failures: destination.consecutive_failures,
            circuit_opened_at: destination.circuit_opened_at,
            max_retries: destination.max_retries,
            timeout_ms: destination.timeout_ms,
            retry_backoff_strategy: destination.retry_backoff_strategy,
            rate_limit_rps: destination.rate_limit_rps,
            secret: Some(plaintext_secret),
            created_at: destination.created_at,
            updated_at: destination.updated_at,
        })
    }

    pub async fn get_destination(&self, tenant_id: Uuid, id: Uuid) -> Result<Option<DestinationView>, CoreError> {
        let repo = DestinationRepository::new(&self.pool);
        let destination = repo.find_by_tenant_and_id(tenant_id, id).await?;

        Ok(destination.map(to_destination_view_redacted))
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

        Ok(destinations.into_iter().map(to_destination_view_redacted).collect())
    }

    pub async fn update_destination(
        &self,
        tenant_id: Uuid,
        id: Uuid,
        input: UpdateDestinationInput,
    ) -> Result<DestinationView, CoreError> {
        if input.secret.is_some() {
            return Err(CoreError::Validation("Direct mutation of destination signing secret is not permitted".to_string()));
        }

        if let Some(ref name) = input.name {
            if name.trim().is_empty() {
                return Err(CoreError::Validation("Destination name cannot be empty".to_string()));
            }
        }

        let validated_url = if let Some(ref u) = input.url {
            Some(validate_destination_url_with_mode(u, &self.environment)?)
        } else {
            None
        };

        if let Some(timeout) = input.timeout_ms {
            if timeout <= 0 {
                return Err(CoreError::Validation("Destination timeout must be a positive integer".to_string()));
            }
        }

        if let Some(retries) = input.max_retries {
            if retries < 0 {
                return Err(CoreError::Validation("Destination max retries must be non-negative".to_string()));
            }
        }

        if let Some(rps) = input.rate_limit_rps {
            if rps <= 0 {
                return Err(CoreError::Validation("Destination rate limit must be a positive integer".to_string()));
            }
        }

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
                input.max_retries,
                input.retry_backoff_strategy.as_deref().map(|s| s.trim()),
                input.is_active,
            )
            .await?;

        Ok(to_destination_view_redacted(destination))
    }

    pub async fn delete_destination(&self, tenant_id: Uuid, id: Uuid) -> Result<(), CoreError> {
        let repo = DestinationRepository::new(&self.pool);
        repo.delete(tenant_id, id).await
    }

    pub async fn pause_destination(&self, tenant_id: Uuid, id: Uuid) -> Result<DestinationView, CoreError> {
        let repo = DestinationRepository::new(&self.pool);
        let destination = repo.pause(tenant_id, id).await?;
        Ok(to_destination_view_redacted(destination))
    }

    pub async fn resume_destination(&self, tenant_id: Uuid, id: Uuid) -> Result<DestinationView, CoreError> {
        let repo = DestinationRepository::new(&self.pool);
        let destination = repo.resume(tenant_id, id).await?;
        Ok(to_destination_view_redacted(destination))
    }

    pub async fn test_destination(&self, tenant_id: Uuid, id: Uuid) -> Result<TestDestinationResponse, CoreError> {
        let repo = DestinationRepository::new(&self.pool);
        let destination = repo
            .find_by_tenant_and_id(tenant_id, id)
            .await?
            .ok_or_else(|| CoreError::NotFound(format!("Destination '{id}' not found")))?;

        let plaintext_secret = if let Some(ref enc) = destination.secret_encrypted {
            let dec = decrypt_secret(enc, &self.encryption_key)?;
            Some(String::from_utf8_lossy(&dec).to_string())
        } else {
            None
        };

        let now_ts = chrono::Utc::now().timestamp();
        let payload = serde_json::json!({
            "type": "relay.test",
            "timestamp": now_ts,
        });
        let payload_str = payload.to_string();

        let mut req_builder = self
            .http_client
            .post(&destination.url)
            .header("Content-Type", "application/json")
            .header("User-Agent", "RelayCore-Webhook/1.0")
            .header("X-Relay-Timestamp", now_ts.to_string())
            .timeout(Duration::from_millis(destination.timeout_ms.max(1) as u64))
            .body(payload_str.clone());

        if let Some(ref sec) = plaintext_secret {
            if let Ok(sig) = sign_hmac_sha256(sec.as_bytes(), payload_str.as_bytes()) {
                req_builder = req_builder.header("X-Relay-Signature", format!("sha256={sig}"));
            }
        }

        let start = std::time::Instant::now();
        match req_builder.send().await {
            Ok(response) => {
                let latency_ms = start.elapsed().as_millis() as i64;
                let status = response.status();
                let status_code = status.as_u16() as i32;
                let success = status.is_success();
                let error = if !success {
                    Some(format!("HTTP error status: {status_code}"))
                } else {
                    None
                };

                Ok(TestDestinationResponse {
                    success,
                    http_status: Some(status_code),
                    latency_ms,
                    error,
                })
            }
            Err(e) => {
                let latency_ms = start.elapsed().as_millis() as i64;
                let status_code = e.status().map(|s| s.as_u16() as i32);
                let err_msg = if e.is_timeout() {
                    "Destination request timed out".to_string()
                } else {
                    format!("Connection error: {e}")
                };

                Ok(TestDestinationResponse {
                    success: false,
                    http_status: status_code,
                    latency_ms,
                    error: Some(err_msg),
                })
            }
        }
    }

    pub async fn get_destination_health(&self, tenant_id: Uuid, id: Uuid) -> Result<DestinationHealthView, CoreError> {
        let repo = DestinationRepository::new(&self.pool);
        let stats = repo
            .get_health_stats(tenant_id, id)
            .await?
            .ok_or_else(|| CoreError::NotFound(format!("Destination '{id}' not found")))?;

        let success_rate = if stats.total == 0 {
            1.0
        } else {
            stats.successes as f64 / stats.total as f64
        };

        Ok(DestinationHealthView {
            status: stats.status,
            consecutive_failures: stats.consecutive_failures,
            circuit_opened_at: stats.circuit_opened_at,
            success_rate,
            total_attempts: stats.total,
            successful_attempts: stats.successes,
        })
    }
}

fn to_destination_view_redacted(d: Destination) -> DestinationView {
    DestinationView {
        id: d.id,
        tenant_id: d.tenant_id,
        name: d.name,
        url: d.url,
        description: d.description,
        status: d.status,
        is_active: d.is_active,
        consecutive_failures: d.consecutive_failures,
        circuit_opened_at: d.circuit_opened_at,
        max_retries: d.max_retries,
        timeout_ms: d.timeout_ms,
        retry_backoff_strategy: d.retry_backoff_strategy,
        rate_limit_rps: d.rate_limit_rps,
        secret: None,
        created_at: d.created_at,
        updated_at: d.updated_at,
    }
}

/// Validates that a destination URL is present, well-formed, uses HTTP(S), and has a valid host.
pub fn validate_destination_url(raw_url: &str) -> Result<String, CoreError> {
    validate_destination_url_with_mode(raw_url, "development")
}

pub fn validate_destination_url_with_mode(raw_url: &str, environment: &str) -> Result<String, CoreError> {
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

    if environment == "production" && parsed.scheme() != "https" {
        return Err(CoreError::Validation(
            "HTTPS is required for destination URLs in production mode".to_string(),
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

    #[test]
    fn test_production_mode_https_enforcement() {
        assert!(validate_destination_url_with_mode("http://example.com/webhook", "production").is_err());
        assert!(validate_destination_url_with_mode("https://example.com/webhook", "production").is_ok());
        assert!(validate_destination_url_with_mode("http://example.com/webhook", "development").is_ok());
    }
}
