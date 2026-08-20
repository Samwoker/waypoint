use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use serde_json::Value;

use crate::state::AppState;

pub async fn receive_webhook(
    State(_state): State<AppState>,
    Path(slug): Path<String>,
    _headers: HeaderMap,
    Json(_payload): Json<Value>,
) -> impl IntoResponse {
    let _ = slug;
    StatusCode::NOT_IMPLEMENTED
}
