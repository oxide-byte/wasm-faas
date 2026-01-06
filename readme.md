# WASM-FAAS

## Introduction

This is a simple POC demonstrating WASM functions executed by a common server.

Directory wit contains a global WASM Interface that all WASM Modules need to implement.

## Description

## Build

To build the WASM module as a component, you need `wasm-tools`.

```shell
cargo build -p server --release
````

```shell
curl -LO https://github.com/bytecodealliance/wasmtime/releases/latest/download/wasi_snapshot_preview1.reactor.wasm
```

```shell
cargo build -p fibonacci-faas --target wasm32-wasip1 --release
wasm-tools component new ./target/wasm32-wasip1/release/faas_exec.wasm -o ./target/wasm32-wasip1/release/fibonacci_faas.wasm --adapt ./wasi_snapshot_preview1.reactor.wasm
```

Note: You need the `wasi_snapshot_preview1.reactor.wasm` adapter to convert the WASI preview1 module to a component.
You can download it from the [wasmtime releases](https://github.com/bytecodealliance/wasmtime/releases/latest/download/wasi_snapshot_preview1.reactor.wasm):