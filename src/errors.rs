use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),

    #[error("Password error: {0}")]
    PasswordError(#[from] bcrypt::BcryptError),

    #[error("JWT error: {0}")]
    JwtError(#[from] jsonwebtoken::errors::Error),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Bad Request: {0}")]
    BadRequest(String),

    #[error("Not Found: {0}")]
    NotFound(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Too Many Requests: {0}")]
    TooManyRequests(String),

    #[allow(dead_code)]
    #[error("Internal Server Error: {0}")]
    InternalServerError(String),

    #[error("Task Join Error: {0}")]
    TaskJoinError(#[from] tokio::task::JoinError),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            AppError::DatabaseError(err) => {
                tracing::error!("Database error occurred: {:?}", err);
                (StatusCode::INTERNAL_SERVER_ERROR, "A database error occurred".to_string())
            }
            AppError::PasswordError(err) => {
                tracing::error!("Password hashing error occurred: {:?}", err);
                (StatusCode::INTERNAL_SERVER_ERROR, "An internal authentication error occurred".to_string())
            }
            AppError::JwtError(err) => {
                tracing::warn!("JWT verification error: {:?}", err);
                (StatusCode::UNAUTHORIZED, "Invalid or expired authorization token".to_string())
            }
            AppError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg.clone()),
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg.clone()),
            AppError::Conflict(msg) => (StatusCode::CONFLICT, msg.clone()),
            AppError::TooManyRequests(msg) => (StatusCode::TOO_MANY_REQUESTS, msg.clone()),
            AppError::InternalServerError(msg) => {
                tracing::error!("Internal error: {}", msg);
                (StatusCode::INTERNAL_SERVER_ERROR, msg.clone())
            }
            AppError::TaskJoinError(err) => {
                tracing::error!("Task join error: {:?}", err);
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal processing error".to_string())
            }
        };

        let body = Json(json!({
            "error": message
        }));

        (status, body).into_response()
    }
}
