use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use serde_json::{json, Value};
use data::repositories::SourceRepository;
use relay_core::crypto::{decrypt_secret, verify_hmac_sha256, verify_hmac_sha256_base64};
use relay_core::error::CoreError;
use domain::dto::CreateEventInput;

use crate::error::ApiError;
use crate::state::AppState;

pub async fn receive_webhook(
    State(state): State<AppState>,
    Path(slug): Path<String>,
    headers: HeaderMap,
    Json(payload): Json<Value>,
) -> Result<impl IntoResponse, ApiError> {
    let source_repo = SourceRepository::new(&state.pool);
    let source = source_repo
        .find_by_slug(&slug)
        .await
        .map_err(ApiError)?
        .ok_or_else(|| ApiError(CoreError::NotFound(format!("Source with slug '{slug}' not found"))))?;

    // Signature verification
    let verification_type = source.verification_type.to_lowercase();
    if verification_type != "none" && !verification_type.is_empty() {
        if let Some(encrypted_secret) = source.encrypted_secret.filter(|s| !s.is_empty()) {
            let secret_bytes = decrypt_secret(&encrypted_secret, &state.source_service.encryption_key)
                .map_err(ApiError)?;
            let raw_body = serde_json::to_vec(&payload)
                .map_err(|e| ApiError(CoreError::Validation(format!("Invalid JSON body: {e}"))))?;

            let is_valid = match verification_type.as_str() {
                "hmac_sha256" | "generic" => {
                    let sig = headers
                        .get("x-signature")
                        .or_else(|| headers.get("x-hub-signature-256"))
                        .or_else(|| headers.get("webhook-signature"))
                        .and_then(|v| v.to_str().ok())
                        .map(|s| s.strip_prefix("sha256=").unwrap_or(s).trim());

                    if let Some(sig_hex) = sig {
                        verify_hmac_sha256(&secret_bytes, &raw_body, sig_hex).unwrap_or(false)
                    } else {
                        false
                    }
                }
                "stripe" => {
                    let sig_header = headers
                        .get("stripe-signature")
                        .and_then(|v| v.to_str().ok());

                    if let Some(sig_str) = sig_header {
                        let mut timestamp = "";
                        let mut signature = "";
                        for part in sig_str.split(',') {
                            let mut kv = part.splitn(2, '=');
                            let k = kv.next().unwrap_or("").trim();
                            let v = kv.next().unwrap_or("").trim();
                            if k == "t" {
                                timestamp = v;
                            } else if k == "v1" {
                                signature = v;
                            }
                        }

                        if !timestamp.is_empty() && !signature.is_empty() {
                            let signed_payload = format!("{timestamp}.{}", String::from_utf8_lossy(&raw_body));
                            verify_hmac_sha256(&secret_bytes, signed_payload.as_bytes(), signature).unwrap_or(false)
                        } else {
                            verify_hmac_sha256(&secret_bytes, &raw_body, sig_str).unwrap_or(false)
                        }
                    } else {
                        false
                    }
                }
                "github" => {
                    let sig = headers
                        .get("x-hub-signature-256")
                        .or_else(|| headers.get("x-hub-signature"))
                        .and_then(|v| v.to_str().ok())
                        .map(|s| s.strip_prefix("sha256=").unwrap_or(s).trim());

                    if let Some(sig_hex) = sig {
                        verify_hmac_sha256(&secret_bytes, &raw_body, sig_hex).unwrap_or(false)
                    } else {
                        false
                    }
                }
                "shopify" => {
                    let sig = headers
                        .get("x-shopify-hmac-sha256")
                        .and_then(|v| v.to_str().ok())
                        .map(|s| s.trim());

                    if let Some(sig_b64) = sig {
                        verify_hmac_sha256_base64(&secret_bytes, &raw_body, sig_b64).unwrap_or(false)
                    } else {
                        false
                    }
                }
                _ => true,
            };

            if !is_valid {
                return Err(ApiError(CoreError::Unauthorized("Invalid webhook signature".to_string())));
            }
        }
    }

    // Determine event_type
    let event_type = headers
        .get("x-event-type")
        .or_else(|| headers.get("x-github-event"))
        .or_else(|| headers.get("x-shopify-topic"))
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
        .or_else(|| {
            payload.get("type").or_else(|| payload.get("event")).and_then(|v| v.as_str()).map(|s| s.to_string())
        })
        .unwrap_or_else(|| format!("{slug}.event"));

    // Determine idempotency key
    let idempotency_key = headers
        .get("idempotency-key")
        .or_else(|| headers.get("x-idempotency-key"))
        .or_else(|| headers.get("x-github-delivery"))
        .or_else(|| headers.get("x-shopify-webhook-id"))
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
        .or_else(|| {
            payload.get("id").and_then(|v| v.as_str()).map(|s| s.to_string())
        });

    // Build headers JSON
    let mut headers_map = serde_json::Map::new();
    for (name, value) in &headers {
        if let Ok(val_str) = value.to_str() {
            headers_map.insert(name.as_str().to_string(), Value::String(val_str.to_string()));
        }
    }
    headers_map.insert("signature_valid".to_string(), Value::Bool(true));

    let event = state
        .ingestion_service
        .create_event(
            source.tenant_id,
            CreateEventInput {
                source_id: Some(source.id),
                event_type,
                payload,
                idempotency_key,
                headers: Some(Value::Object(headers_map)),
            },
        )
        .await
        .map_err(ApiError)?;

    Ok((
        StatusCode::ACCEPTED,
        Json(json!({
            "id": event.id,
            "status": "received",
            "event_type": event.event_type,
            "created_at": event.created_at,
        })),
    ))
}
