use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json
};
use thiserror::Error;
use serde_json::json;


#[derive(Error, Debug)]
pub enum AppError {
    // CLIENT ERRORS
    #[error("The requested image does not exist")]
    NotFound,
    
    #[error("Error de validación: {0}")]
    ValidationError(String),
    
    // SERVER ERRORS
    #[error("The server is under to much load.")]
    ServerBusy,

    #[error("Reading from disk failed: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Error processing image: {0}")]
    VipsError(#[from] libvips::error::Error),
    
    #[error("Unexpected Error: {0}")]
    UnexpectedError(#[from] anyhow::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::NotFound => (StatusCode::NOT_FOUND, "Image not found".to_string()),
            AppError::ValidationError(msg) => (StatusCode::BAD_REQUEST, msg),
            AppError::ServerBusy => (StatusCode::SERVICE_UNAVAILABLE, "Server is busy (Semaphore timeout)".to_string()),
            // Internal errors
            // Log the technical details, but to the user we give a generic message
            AppError::IoError(e) => {
                tracing::error!("Disk I/O Error: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "Storage error".to_string())
            },
            AppError::VipsError(e) => {
                tracing::error!("Libvips Conversion Error: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "Image Processing Failed".to_string())
            },
            AppError::UnexpectedError(error) => {
                tracing::error!("Unexpected Error: {:?}", error);
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error".to_string())
            }
        };

        // Build Json Body
        let body = Json(json!({
            "error" : {
                "code" : status.as_u16(),
                "type": status.canonical_reason().unwrap_or("Unknown"),
                "message" : error_message
            }
        }));
    
        (status, body).into_response()
    }
}