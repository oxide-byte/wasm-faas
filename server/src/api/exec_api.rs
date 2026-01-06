use crate::error::AppError;
use axum::Json;
use std::time::Instant;
use wasmtime::{Config, Engine, Instance, Module, OptLevel, Store};

pub async fn exec_wasm() -> Result<Json<i32>, AppError> {
    let start = Instant::now();

    let engine = Engine::new(
        Config::new()
            .debug_info(true)
            .cranelift_opt_level(OptLevel::None),
    )?;

    let mut store = Store::new(&engine, ());
    let module = Module::from_file(&engine, "../target/wasm32-wasip1/release/faas_exec.wasm")?;
    let instance = Instance::new(&mut store, &module, &[])?;

    let fib = instance.get_typed_func::<i32, i32>(&mut store, "faas_exec")?;
    let result = fib.call(&mut store, 6);
    println!("Time {}ms", start.elapsed().as_millis());

    Ok(Json::from(result?))
}