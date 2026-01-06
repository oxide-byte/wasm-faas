use crate::error::AppError;
use aws_config::{BehaviorVersion, Region};
use aws_sdk_s3::config::Credentials;
use aws_sdk_s3::primitives::ByteStream;
use aws_sdk_s3::Client;

pub struct Config {
    pub region: String,
    pub access_key_id: String,
    pub secret_access_key: String,
    pub endpoint_url: String,
}

impl Config {
    pub fn docker() -> Config {
        Config {
            // rustfsadmin
            region: "eu-west-1".to_string(),
            access_key_id: "rustfsadmin".to_string(),
            secret_access_key: "rustfsadmin".to_string(),
            endpoint_url: "http://localhost:9000".to_string(),
        }
    }
}

pub struct S3 {
    pub client: aws_sdk_s3::Client,
}

impl S3 {
    pub async fn new() -> Self {
        let config = Config::docker();

        let credentials = Credentials::new(
            config.access_key_id,
            config.secret_access_key,
            None,
            None,
            "rustfs",
        );

        let region = Region::new(config.region);

        let endpoint_url = config.endpoint_url;

        let shard_config = aws_config::defaults(BehaviorVersion::latest())
            .region(region)
            .credentials_provider(credentials)
            .endpoint_url(endpoint_url)
            .load()
            .await;

        let rustfs_client = Client::from_conf(
            aws_sdk_s3::config::Builder::from(&shard_config)
                .force_path_style(true)
                .build(),
        );

        S3 {
            client: rustfs_client,
        }
    }

    pub async fn create_bucket(&self, bucket: &str) -> Result<(), AppError> {
        self.client
            .create_bucket()
            .bucket(bucket)
            .send()
            .await
            .map_err(AppError::from_s3)?;
        Ok(())
    }

    pub async fn delete_bucket(&self, bucket: &str) -> Result<(), AppError> {
        self.client
            .delete_bucket()
            .bucket(bucket)
            .send()
            .await
            .map_err(AppError::from_s3)?;
        Ok(())
    }

    pub async fn list_files(&self, bucket: &str) -> Result<Vec<String>, AppError> {
        let res = self
            .client
            .list_objects_v2()
            .bucket(bucket)
            .send()
            .await
            .map_err(AppError::from_s3)?;

        let files = res
            .contents()
            .iter()
            .filter_map(|obj| obj.key().map(|s| s.to_string()))
            .collect();

        Ok(files)
    }

    pub async fn upload_file(
        &self,
        bucket: &str,
        key: &str,
        body: ByteStream,
    ) -> Result<(), AppError> {
        self.client
            .put_object()
            .bucket(bucket)
            .key(key)
            .body(body)
            .send()
            .await
            .map_err(AppError::from_s3)?;
        Ok(())
    }

    pub async fn download_file(&self, bucket: &str, key: &str) -> Result<ByteStream, AppError> {
        let res = self
            .client
            .get_object()
            .bucket(bucket)
            .key(key)
            .send()
            .await
            .map_err(AppError::from_s3)?;

        Ok(res.body)
    }

    pub async fn delete_file(&self, bucket: &str, key: &str) -> Result<(), AppError> {
        self.client
            .delete_object()
            .bucket(bucket)
            .key(key)
            .send()
            .await
            .map_err(AppError::from_s3)?;
        Ok(())
    }
}