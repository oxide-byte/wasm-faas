use crate::error::AppError;
use crate::tools::s3::S3;
use axum::extract::{Path, State};
use axum::Json;
use serde_json;
use std::sync::Arc;
use std::time::Instant;
use wasmtime::component::{Component, Linker, ResourceTable};
use wasmtime::{Config, Engine, OptLevel, Store};
use wasmtime_wasi::{WasiCtx, WasiCtxBuilder, WasiView, WasiCtxView};
use wasmtime_wasi_http::{WasiHttpCtx, WasiHttpView};

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

pub async fn exec_wasm(
    State(s3): State<Arc<S3>>,
    Path((bucket, key)): Path<(String, String)>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, AppError> {
    let start = Instant::now();

    let mut config = Config::new();
    config.wasm_component_model(true);
    config.async_support(true);
    config.debug_info(true);
    config.cranelift_opt_level(OptLevel::None);

    let engine = Engine::new(&config)?;

    let wasm_stream = s3.download_file(&bucket, &key).await?;

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

    let input_json =
        serde_json::to_string(&payload).map_err(|e| AppError::Internal(e.to_string()))?;

    let output_json_str = bindings.call_exec(&mut store, &input_json).await?;

    let output_json: serde_json::Value =
        serde_json::from_str(&output_json_str).map_err(|e| AppError::Internal(e.to_string()))?;

    println!("Time {}ms", start.elapsed().as_millis());

    Ok(Json(output_json))
}