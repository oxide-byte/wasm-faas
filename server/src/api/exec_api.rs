use crate::error::AppError;
use crate::tools::s3::S3;
use axum::extract::State;
use axum::Json;
use std::sync::Arc;
use std::time::Instant;
use wasmtime::{Config, Engine, Instance, Module, OptLevel, Store};

pub async fn exec_wasm(State(s3): State<Arc<S3>>) -> Result<Json<i32>, AppError> {
    let start = Instant::now();

    let engine = Engine::new(
        Config::new()
            .debug_info(true)
            .cranelift_opt_level(OptLevel::None),
    )?;

    let mut store = Store::new(&engine, ());
    let wasm_stream = s3.download_file("faas-modules", "fibonacci_faas.wasm").await?;
    let wasm_bytes = wasm_stream
        .collect()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .to_vec();

    let module = Module::from_binary(&engine, &wasm_bytes)?;
    let instance = Instance::new(&mut store, &module, &[])?;

    let fib = instance.get_typed_func::<i32, i32>(&mut store, "faas_exec")?;
    let result = fib.call(&mut store, 6);
    println!("Time {}ms", start.elapsed().as_millis());

    Ok(Json::from(result?))
}