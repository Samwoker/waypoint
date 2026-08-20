use axum::{
    async_trait,
    extract::FromRequestParts,
    http::request::Parts,
};
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use relay_core::error::CoreError;
use crate::error::ApiError;
use crate::state::AppState;

#[derive(Debug, Clone)]
pub struct AuthenticatedTenant {
    pub tenant_id: Uuid,
    pub is_admin: bool,
    pub scope: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub tenant_id: Uuid,
    pub role: Option<String>,
    pub is_admin: Option<bool>,
    pub scope: Option<String>,
    pub exp: usize,
}

#[async_trait]
impl FromRequestParts<AppState> for AuthenticatedTenant {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        // 1. Check Authorization header: "Bearer <token/key>" or "ApiKey <key>"
        if let Some(auth_val) = parts.headers.get("authorization") {
            let auth_str = auth_val
                .to_str()
                .map_err(|_| ApiError(CoreError::Unauthorized("Invalid authorization header encoding".to_string())))?;

            let token = if let Some(stripped) = auth_str.strip_prefix("Bearer ") {
                stripped.trim()
            } else if let Some(stripped) = auth_str.strip_prefix("bearer ") {
                stripped.trim()
            } else if let Some(stripped) = auth_str.strip_prefix("ApiKey ") {
                stripped.trim()
            } else if let Some(stripped) = auth_str.strip_prefix("apikey ") {
                stripped.trim()
            } else {
                auth_str.trim()
            };

            if token.is_empty() {
                return Err(ApiError(CoreError::Unauthorized("Empty authorization token".to_string())));
            }

            // Try JWT token decoding first
            if let Ok(token_data) = decode::<Claims>(
                token,
                &DecodingKey::from_secret(state.config.jwt_secret.as_bytes()),
                &Validation::default(),
            ) {
                let is_admin = token_data.claims.is_admin.unwrap_or(false)
                    || token_data.claims.role.as_deref() == Some("admin")
                    || token_data.claims.role.as_deref() == Some("platform_admin");
                let scope = token_data.claims.scope.unwrap_or_else(|| "full".to_string());

                return Ok(AuthenticatedTenant {
                    tenant_id: token_data.claims.tenant_id,
                    is_admin,
                    scope,
                });
            }

            // Try API Key validation against database
            match state.auth_service.validate_api_key_with_scope(token).await {
                Ok((tenant_id, scope)) => return Ok(AuthenticatedTenant { tenant_id, is_admin: false, scope }),
                Err(_) => return Err(ApiError(CoreError::Unauthorized("Invalid API key or token".to_string()))),
            }
        }

        // 2. Check X-Api-Key header: "<key>"
        if let Some(api_key_val) = parts.headers.get("x-api-key") {
            let api_key = api_key_val
                .to_str()
                .map_err(|_| ApiError(CoreError::Unauthorized("Invalid x-api-key header encoding".to_string())))?
                .trim();

            if api_key.is_empty() {
                return Err(ApiError(CoreError::Unauthorized("Empty x-api-key header".to_string())));
            }

            match state.auth_service.validate_api_key_with_scope(api_key).await {
                Ok((tenant_id, scope)) => return Ok(AuthenticatedTenant { tenant_id, is_admin: false, scope }),
                Err(_) => return Err(ApiError(CoreError::Unauthorized("Invalid API key".to_string()))),
            }
        }

        Err(ApiError(CoreError::Unauthorized("Missing authentication credentials".to_string())))
    }
}
