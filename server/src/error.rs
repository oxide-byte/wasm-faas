use aws_sdk_s3::error::ProvideErrorMetadata;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("S3 error: ({0}) {1}")]
    S3Error(String, String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Internal server error")]
    Internal(String),

    #[error("Wasmtime error: {0}")]
    WasmtimeError(#[from] wasmtime::Error),
}

impl AppError {
    pub fn from_s3<E>(err: E) -> Self
    where
        E: ProvideErrorMetadata,
    {
        Self::S3Error(
            err.code().unwrap_or_default().to_string(),
            err.message().unwrap_or_default().to_string(),
        )
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        println!("{:?}", self);
        let (status, error_message) = match self {
            AppError::S3Error(code, msg) => {
                (StatusCode::INTERNAL_SERVER_ERROR, format!("{}: {}", code, msg))
            }
            AppError::IoError(err) => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()),
            AppError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            AppError::WasmtimeError(err) => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()),
        };

        let body = Json(json!({
            "error": error_message,
        }));

        (status, body).into_response()
    }
}