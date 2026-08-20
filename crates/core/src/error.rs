use thiserror::Error;

#[derive(Debug, Error)]
pub enum CoreError {
    #[error("Resource not found: {0}")]
    NotFound(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Validation failed: {0}")]
    Validation(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Forbidden: {0}")]
    Forbidden(String),

    #[error("Cryptographic error: {0}")]
    Crypto(String),

    #[error("Invalid signature")]
    SignatureInvalid,

    #[error("Internal error: {0}")]
    Internal(String),
}
