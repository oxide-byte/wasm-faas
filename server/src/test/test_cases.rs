#![allow(unused_imports)]
use wasmtime::component::{Component, Linker, ResourceTable};
use wasmtime::{Config, Engine, Store};
use wasmtime_wasi::{WasiCtx, WasiCtxBuilder, WasiView, WasiCtxView};
use wasmtime_wasi_http::{WasiHttpCtx, WasiHttpView};
use std::fs;
use crate::tools::s3::S3;
use crate::error::AppError;
use serde_json::json;

#[allow(dead_code)]
struct ServerState {
    ctx: WasiCtx,
    table: ResourceTable,
    http: WasiHttpCtx,
}

impl WasiView for ServerState {
    fn ctx(&mut self) -> WasiCtxView<'_> {
        WasiCtxView {
            ctx: &mut self.ctx,
            table: &mut self.table,
        }
    }
}

impl WasiHttpView for ServerState {
    fn ctx(&mut self) -> &mut WasiHttpCtx {
        &mut self.http
    }
    fn table(&mut self) -> &mut ResourceTable {
        &mut self.table
    }
}

wasmtime::component::bindgen!({
    world: "faas-exec",
    path: "../wit",
    exports: {
        "exec": async
    }
});

#[test]
fn test_file_valid() -> Result<(), Box<dyn std::error::Error>> {
    let mut config = Config::new();
    config.wasm_component_model(true);
    let engine = Engine::new(&config)?;

    let wasm_path = "../target/wasm32-wasip1/release/hello_faas.wasm";
    let wasm_bytes = fs::read(wasm_path)?;
    println!("Read {} bytes from {}", wasm_bytes.len(), wasm_path);

    match Component::from_binary(&engine, &wasm_bytes) {
        Ok(_) => println!("Successfully parsed component"),
        Err(e) => println!("Failed to parse component: {}", e),
    }

    Ok(())
}

#[tokio::test]
async fn test_download_valid() -> Result<(), Box<dyn std::error::Error>> {
    let s3 = S3::new().await;
    let key = "fibonacci_faas.wasm";
    let wasm_stream = s3.download_file("faas-modules", key).await?;

    let wasm_bytes = wasm_stream
        .collect()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .to_vec();
    
    let mut config = Config::new();
    config.wasm_component_model(true);
    let engine = Engine::new(&config)?;

    println!("Read {} bytes from S3 key: {}", wasm_bytes.len(), key);

    match Component::from_binary(&engine, &wasm_bytes) {
        Ok(_) => println!("Successfully parsed component"),
        Err(e) => println!("Failed to parse component: {}", e),
    }

    Ok(())
}

#[tokio::test]
async fn test_exec_local_valid() -> Result<(), Box<dyn std::error::Error>> {
    let wasm_file = "s3_faas.wasm";
    let input = json!({ "bucket": "faas-modules" });

    let mut config = Config::new();
    config.wasm_component_model(true);
    config.async_support(true);
    let engine = Engine::new(&config)?;

    let wasm_path = format!("../target/wasm32-wasip1/release/{wasm_file}");
    let wasm_bytes = fs::read(wasm_path)?;
    
    let component = Component::from_binary(&engine, &wasm_bytes)?;

    let mut linker = Linker::new(&engine);
    wasmtime_wasi::p2::add_to_linker_async(&mut linker)?;
    wasmtime_wasi_http::add_only_http_to_linker_async(&mut linker)?;

    let mut store = Store::new(
        &engine,
        ServerState {
            ctx: WasiCtxBuilder::new().inherit_stdout().build(),
            table: ResourceTable::new(),
            http: WasiHttpCtx::new(),
        },
    );

    let bindings = FaasExec::instantiate_async(&mut store, &component, &linker).await?;

    let input_str = serde_json::to_string(&input)?;

    let output_str = bindings.call_exec(&mut store, &input_str).await?;
    let output: serde_json::Value = serde_json::from_str(&output_str)?;

    println!("Output: {}", output);

    Ok(())
}

#[tokio::test]
async fn test_exec_remote_valid() -> Result<(), Box<dyn std::error::Error>> {
    let s3 = S3::new().await;
    let wasm_file = "s3_faas.wasm";
    let input = json!({ "bucket": "faas-modules" });

    let wasm_stream = s3.download_file("faas-modules", wasm_file).await?;

    let mut config = Config::new();
    config.wasm_component_model(true);
    config.async_support(true);
    let engine = Engine::new(&config)?;

    let wasm_bytes = wasm_stream
        .collect()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .to_vec();

    let component = Component::from_binary(&engine, &wasm_bytes)?;

    let mut linker = Linker::new(&engine);
    wasmtime_wasi::p2::add_to_linker_async(&mut linker)?;
    wasmtime_wasi_http::add_only_http_to_linker_async(&mut linker)?;

    let mut store = Store::new(
        &engine,
        ServerState {
            ctx: WasiCtxBuilder::new().inherit_stdout().build(),
            table: ResourceTable::new(),
            http: WasiHttpCtx::new(),
        },
    );

    let bindings = FaasExec::instantiate_async(&mut store, &component, &linker).await?;

    let input_str = serde_json::to_string(&input)?;

    let output_str = bindings.call_exec(&mut store, &input_str).await?;
    let output: serde_json::Value = serde_json::from_str(&output_str)?;

    println!("Output: {}", output);

    Ok(())
}