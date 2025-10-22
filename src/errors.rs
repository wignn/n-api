#[derive(Debug)]
pub enum ConfigError {
    MissingVar(String),
    ParseError(String, std::num::ParseIntError),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::MissingVar(var) => write!(f, "Environment variable {} not configured", var),
            ConfigError::ParseError(var, err) => write!(f, "Failed to parse {}: {}", var, err),
        }
    }
}

impl std::error::Error for ConfigError {}

use thiserror::Error;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use axum::extract::rejection::JsonRejection;
use serde_json::json;
use validator::ValidationErrors;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("JWT error: {0}")]
    Jwt(#[from] jsonwebtoken::errors::Error),

    #[error("Password hashing error: {0}")]
    PasswordHash(String),

    #[error("Validation error")]
    ValidationError(#[from] ValidationErrors),

    #[error("JSON parsing error: {0}")]
    JsonRejection(#[from] JsonRejection),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Unauthorized")]
    Unauthorized,

    #[error("Forbidden")]
    Forbidden,

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Internal server error")]
    InternalServer,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message, details) = match self {
            AppError::Database(ref e) => {
                tracing::error!("Database error: {:?}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string(), None)
            }
            AppError::ValidationError(ref errors) => {
                let error_messages: Vec<String> = errors
                    .field_errors()
                    .into_iter()
                    .flat_map(|(field, errors)| {
                        errors.iter().map(move |error| {
                            let message = error.message
                                .as_ref()
                                .map(|m| m.to_string())
                                .unwrap_or_else(|| "Invalid value".to_string());
                            format!("{}: {}", field, message)
                        })
                    })
                    .collect();

                (
                    StatusCode::BAD_REQUEST,
                    "Validation failed".to_string(),
                    Some(json!({"messages": error_messages}))
                )
            }
            AppError::JsonRejection(ref rejection) => {
                (
                    StatusCode::BAD_REQUEST,
                    "Invalid JSON format".to_string(),
                    Some(json!({"details": rejection.to_string()}))
                )
            }
            AppError::Jwt(_) => (StatusCode::UNAUTHORIZED, "Invalid token".to_string(), None),
            AppError::PasswordHash(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string(), None),
            AppError::Validation(ref msg) => (StatusCode::BAD_REQUEST, msg.clone(), None),
            AppError::Unauthorized => (StatusCode::UNAUTHORIZED, "Unauthorized".to_string(), None),
            AppError::Forbidden => (StatusCode::FORBIDDEN, "Forbidden".to_string(), None),
            AppError::NotFound(ref msg) => (StatusCode::NOT_FOUND, msg.clone(), None),
            AppError::Conflict(ref msg) => (StatusCode::CONFLICT, msg.clone(), None),
            AppError::BadRequest(ref msg) => (StatusCode::BAD_REQUEST, msg.clone(), None),
            AppError::InternalServer => (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string(), None),
        };

        let mut body = json!({
            "error": error_message,
            "status": status.as_u16()
        });

        if let Some(details) = details {
            if let serde_json::Value::Object(ref mut map) = body {
                if let serde_json::Value::Object(details_map) = details {
                    for (key, value) in details_map {
                        map.insert(key, value);
                    }
                }
            }
        }

        (status, Json(body)).into_response()
    }
}

pub type AppResult<T> = Result<T, AppError>;