use crate::error::AppError;
use crate::tools::s3::S3;
use axum::extract::{Multipart, Path, State};
use axum::http::header::{CONTENT_DISPOSITION, CONTENT_TYPE};
use axum::http::HeaderMap;
use axum::response::IntoResponse;
use axum::Json;
use std::sync::Arc;
use tokio_util::io::ReaderStream;

pub async fn upload_file(
    State(s3): State<Arc<S3>>,
    Path((bucket, key)): Path<(String, String)>,
    mut multipart: Multipart,
) -> Result<Json<String>, AppError> {
    while let Some(field) = multipart.next_field().await.map_err(|e| AppError::Internal(e.to_string()))? {
        let name = field.name().unwrap_or_default().to_string();
        if name == "file" {

            // Set a reasonable size limit (50MB) to prevent memory issues
            // TODO: handle larger files already > 1 MB (see S3 FAAS)
            let data = field.bytes().await.map_err(|e| AppError::Internal(format!("Failed to read file data: {}", e)))?;
            
            // Check file size to prevent overly large uploads
            if data.len() > 50 * 1024 * 1024 {
                return Err(AppError::Internal("File too large. Maximum size is 50MB".to_string()));
            }
            
            let body = aws_sdk_s3::primitives::ByteStream::from(data.to_vec());
            s3.upload_file(&bucket, &key, body).await?;
            return Ok(Json(format!("File {} uploaded to {}", key, bucket)));
        }
    }

    Err(AppError::Internal("Missing file in multipart".to_string()))
}

pub async fn download_file(
    State(s3): State<Arc<S3>>,
    Path((bucket, key)): Path<(String, String)>,
) -> Result<impl IntoResponse, AppError> {
    let body: aws_sdk_s3::primitives::ByteStream = s3.download_file(&bucket, &key).await?;

    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, "application/octet-stream".parse().unwrap());
    headers.insert(
        CONTENT_DISPOSITION,
        format!("attachment; filename=\"{}\"", key).parse().unwrap(),
    );

    let stream = ReaderStream::new(body.into_async_read());
    Ok((headers, axum::body::Body::from_stream(stream)))
}

pub async fn delete_file(
    State(s3): State<Arc<S3>>,
    Path((bucket, key)): Path<(String, String)>,
) -> Result<Json<String>, AppError> {
    s3.delete_file(&bucket, &key).await?;
    Ok(Json(format!("File {} deleted from {}", key, bucket)))
}