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
    let mut data = Vec::new();

    while let Some(field) = multipart.next_field().await.map_err(|e| AppError::Internal(e.to_string()))? {
        let name = field.name().unwrap_or_default().to_string();
        if name == "file" {
            data = field.bytes().await.map_err(|e: axum::extract::multipart::MultipartError| AppError::Internal(e.to_string()))?.to_vec();
        }
    }

    if data.is_empty() {
        return Err(AppError::Internal("Missing file in multipart".to_string()));
    }

    s3.upload_file(&bucket, &key, data.into()).await?;
    Ok(Json(format!("File {} uploaded to {}", key, bucket)))
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