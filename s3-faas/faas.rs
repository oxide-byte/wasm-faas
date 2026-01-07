wit_bindgen::generate!({
    world: "faas-exec",
    path: "../wit",
    generate_all,
});

struct GuestImpl;

impl Guest for GuestImpl {
    fn exec(input: String) -> String {
        let input: Input = serde_json::from_str(&input).unwrap();

        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_time()
            .build()
            .expect("Failed to generate runtime");

        let output = rt.block_on(async {
            match list_objects(&input.bucket).await {
                Ok(files) => Output {
                    success: true,
                    files,
                },
                Err(_) => Output {
                    success: false,
                    files: vec![],
                },
            }
        });

        serde_json::to_string(&output).unwrap()
    }
}

export!(GuestImpl);

use serde::{Deserialize, Serialize};
use aws_config::Region;
use aws_sdk_s3 as s3;
use aws_sdk_s3::Client;
use aws_sdk_s3::config::Credentials;
use aws_smithy_async::rt::sleep::TokioSleep;
use aws_smithy_wasm::wasi::WasiHttpClientBuilder;

#[derive(Deserialize)]
struct Input {
    bucket: String,
}

#[derive(Serialize)]
struct Output {
    success: bool,
    files: Vec<File>,
}

#[derive(Serialize)]
struct File {
    name: String,
    size: u64,
}

// Based on: https://github.com/smithy-lang/smithy-rs/blob/main/tools/ci-cdk/canary-wasm/src/lib.rs
async fn list_objects(bucket: &str) -> Result<Vec<File>, String> {
    let http_client = WasiHttpClientBuilder::new().build();

    let sleep = TokioSleep::new();

    let credentials = Credentials::new(
        "rustfsadmin".to_string(),
        "rustfsadmin".to_string(),
        None,
        None,
        "rustfs",
    );

    let config = aws_config::from_env()
        .region(Region::new("eu-west-1"))
        .credentials_provider(credentials)
        .http_client(http_client)
        .sleep_impl(sleep)
        .endpoint_url("http://localhost:9000".to_string())
        .load()
        .await;

    let client = Client::from_conf(
        aws_sdk_s3::config::Builder::from(&config)
            .force_path_style(true)
            .build(),
    );

    let result = client
        .list_objects_v2()
        .bucket(bucket)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let files = result
        .contents
        .unwrap_or_default()
        .into_iter()
        .map(|obj| File {
            name: obj.key().unwrap_or_default().to_string(),
            size: obj.size().unwrap_or_default() as u64,
        })
        .collect();

    Ok(files)
}