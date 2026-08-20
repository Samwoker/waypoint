use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use relay_core::error::CoreError;
use serde_json::json;

pub struct ApiError(pub CoreError);

impl From<CoreError> for ApiError {
    fn from(err: CoreError) -> Self {
        ApiError(err)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message) = match self.0 {
            CoreError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            CoreError::Conflict(msg) => (StatusCode::CONFLICT, msg),
            CoreError::Validation(msg) => (StatusCode::UNPROCESSABLE_ENTITY, msg),
            CoreError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg),
            CoreError::Forbidden(msg) => (StatusCode::FORBIDDEN, msg),
            CoreError::Crypto(msg) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Crypto error: {msg}")),
            CoreError::SignatureInvalid => (StatusCode::UNAUTHORIZED, "Invalid signature".to_string()),
            CoreError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
        };

        let body = Json(json!({
            "error": {
                "message": message,
                "status": status.as_u16(),
            }
        }));

        (status, body).into_response()
    }
}
