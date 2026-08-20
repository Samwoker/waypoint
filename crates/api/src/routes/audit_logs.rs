use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::error::ApiError;
use crate::middleware::auth::AuthenticatedTenant;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct ListAuditLogsQuery {
    pub action: Option<String>,
    pub resource_type: Option<String>,
    pub resource_id: Option<Uuid>,
    pub user_id: Option<Uuid>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

pub async fn list_audit_logs(
    tenant: AuthenticatedTenant,
    State(state): State<AppState>,
    Query(query): Query<ListAuditLogsQuery>,
) -> Result<impl IntoResponse, ApiError> {
    let limit = query.limit.unwrap_or(20);
    let offset = query.offset.unwrap_or(0);

    let logs = state
        .audit_service
        .list_audit_logs(
            tenant.tenant_id,
            query.action.as_deref(),
            query.resource_type.as_deref(),
            query.resource_id,
            query.user_id,
            limit,
            offset,
        )
        .await
        .map_err(ApiError)?;

    Ok((StatusCode::OK, Json(logs)))
}
