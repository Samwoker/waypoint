use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use domain::dto::{LoginInput, RefreshTokenInput, RegisterInput};

use crate::error::ApiError;
use crate::middleware::auth::AuthenticatedTenant;
use crate::state::AppState;

pub async fn register(
    State(state): State<AppState>,
    Json(input): Json<RegisterInput>,
) -> Result<impl IntoResponse, ApiError> {
    let token_view = state.auth_service.register(input).await.map_err(ApiError)?;
    Ok((StatusCode::CREATED, Json(token_view)))
}

pub async fn login(
    State(state): State<AppState>,
    Json(input): Json<LoginInput>,
) -> Result<impl IntoResponse, ApiError> {
    let token_view = state.auth_service.login(input).await.map_err(ApiError)?;
    Ok((StatusCode::OK, Json(token_view)))
}

pub async fn refresh_token(
    State(state): State<AppState>,
    headers: HeaderMap,
    payload: Option<Json<RefreshTokenInput>>,
) -> Result<impl IntoResponse, ApiError> {
    let refresh_str = if let Some(Json(body)) = payload {
        body.refresh_token
    } else {
        None
    };

    let token = if let Some(r) = refresh_str {
        r
    } else if let Some(auth_val) = headers.get("authorization") {
        let s = auth_val.to_str().map_err(|_| {
            ApiError(relay_core::error::CoreError::Unauthorized(
                "Invalid header encoding".to_string(),
            ))
        })?;
        if let Some(stripped) = s.strip_prefix("Bearer ") {
            stripped.trim().to_string()
        } else {
            s.trim().to_string()
        }
    } else {
        return Err(ApiError(relay_core::error::CoreError::Unauthorized(
            "Missing refresh token".to_string(),
        )));
    };

    let token_view = state
        .auth_service
        .refresh_token(&token)
        .await
        .map_err(ApiError)?;

    Ok((StatusCode::OK, Json(token_view)))
}

pub async fn me(
    tenant: AuthenticatedTenant,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, ApiError> {
    // If tenant context has a user sub, fetch user profile, otherwise return tenant info
    let tenant_view = state
        .tenant_service
        .get_tenant(tenant.tenant_id)
        .await
        .map_err(ApiError)?
        .ok_or_else(|| {
            ApiError(relay_core::error::CoreError::NotFound(
                "Tenant not found".to_string(),
            ))
        })?;

    Ok((
        StatusCode::OK,
        Json(serde_json::json!({
            "tenant_id": tenant.tenant_id,
            "is_admin": tenant.is_admin,
            "scope": tenant.scope,
            "tenant": tenant_view,
        })),
    ))
}
