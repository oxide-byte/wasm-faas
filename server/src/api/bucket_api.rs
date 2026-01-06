use crate::error::AppError;
use crate::tools::s3::S3;
use axum::extract::{Path, State};
use axum::Json;
use serde::Serialize;
use std::sync::Arc;

#[derive(Serialize)]
pub struct BucketListResponse {
    pub files: Vec<String>,
}

pub async fn create_bucket(
    State(s3): State<Arc<S3>>,
    Path(bucket): Path<String>,
) -> Result<Json<String>, AppError> {
    s3.create_bucket(&bucket).await?;
    Ok(Json(format!("Bucket {} created", bucket)))
}

pub async fn delete_bucket(
    State(s3): State<Arc<S3>>,
    Path(bucket): Path<String>,
) -> Result<Json<String>, AppError> {
    s3.delete_bucket(&bucket).await?;
    Ok(Json(format!("Bucket {} deleted", bucket)))
}

pub async fn list_bucket(
    State(s3): State<Arc<S3>>,
    Path(bucket): Path<String>,
) -> Result<Json<BucketListResponse>, AppError> {
    let files = s3.list_files(&bucket).await?;
    Ok(Json(BucketListResponse { files }))
}