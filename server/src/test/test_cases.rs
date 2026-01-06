#![allow(unused_imports)]
use wasmtime::component::{Component, Linker, ResourceTable};
use wasmtime::{Config, Engine, Store};
use wasmtime_wasi::{WasiCtx, WasiCtxBuilder, WasiView, WasiCtxView};
use std::fs;
use crate::tools::s3::S3;
use crate::error::AppError;
use serde_json::json;

#[allow(dead_code)]
struct ServerState {
    ctx: WasiCtx,
    table: ResourceTable,
}

impl WasiView for ServerState {
    fn ctx(&mut self) -> WasiCtxView<'_> {
        WasiCtxView {
            ctx: &mut self.ctx,
            table: &mut self.table,
        }
    }
}

wasmtime::component::bindgen!({
    world: "faas-exec",
    path: "../wit",
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
    let mut config = Config::new();
    config.wasm_component_model(true);
    config.async_support(false);
    let engine = Engine::new(&config)?;

    let wasm_path = "../target/wasm32-wasip1/release/hello_faas.wasm";
    let wasm_bytes = fs::read(wasm_path)?;
    
    let component = Component::from_binary(&engine, &wasm_bytes)?;

    let mut linker = Linker::new(&engine);
    wasmtime_wasi::p2::add_to_linker_sync(&mut linker)?;

    let mut store = Store::new(
        &engine,
        ServerState {
            ctx: WasiCtxBuilder::new().inherit_stdout().build(),
            table: ResourceTable::new(),
        },
    );

    let bindings = FaasExec::instantiate(&mut store, &component, &linker)?;

    let input = json!({ "name": "James" });
    let input_str = serde_json::to_string(&input)?;

    let output_str = tokio::task::spawn_blocking(move || {
        bindings.call_exec(&mut store, &input_str)
    })
    .await??;
    let output: serde_json::Value = serde_json::from_str(&output_str)?;

    println!("Output: {}", output);

    assert_eq!(output["result"], "Hello James, how are you?");

    Ok(())
}

#[tokio::test]
async fn test_exec_remote_valid() -> Result<(), Box<dyn std::error::Error>> {
    let s3 = S3::new().await;
    let key = "fibonacci_faas.wasm";
    let wasm_stream = s3.download_file("faas-modules", key).await?;

    let mut config = Config::new();
    config.wasm_component_model(true);
    config.async_support(false);
    let engine = Engine::new(&config)?;

    let wasm_bytes = wasm_stream
        .collect()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .to_vec();

    let component = Component::from_binary(&engine, &wasm_bytes)?;

    let mut linker = Linker::new(&engine);
    wasmtime_wasi::p2::add_to_linker_sync(&mut linker)?;

    let mut store = Store::new(
        &engine,
        ServerState {
            ctx: WasiCtxBuilder::new().inherit_stdout().build(),
            table: ResourceTable::new(),
        },
    );

    let bindings = FaasExec::instantiate(&mut store, &component, &linker)?;

    let input = json!({ "n": 10 });
    let input_str = serde_json::to_string(&input)?;

    let output_str = tokio::task::spawn_blocking(move || {
        bindings.call_exec(&mut store, &input_str)
    })
    .await??;
    let output: serde_json::Value = serde_json::from_str(&output_str)?;

    println!("Output: {}", output);

    assert_eq!(output["result"], 144);
    
    Ok(())
}