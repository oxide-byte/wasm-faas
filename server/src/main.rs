mod tools;
mod api;
mod error;

use crate::api::bucket_api::{create_bucket, delete_bucket, list_bucket};
use crate::api::exec_api::exec_wasm;
use crate::api::file_api::{delete_file, download_file, upload_file};
use crate::tools::s3::S3;
use axum::routing::{get, post, put};
use axum::Router;
use std::sync::Arc;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
pub async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "doc_store=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let s3 = Arc::new(S3::new().await);

    let app = Router::new()
        .route("/bucket/{bucket}", put(create_bucket).delete(delete_bucket).get(list_bucket))
        .route("/file/{bucket}/{key}", post(upload_file).get(download_file).delete(delete_file))
        .route("/exec", get(exec_wasm))
        .with_state(s3);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    tracing::debug!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}