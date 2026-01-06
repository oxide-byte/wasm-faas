# WASM-FAAS

## Introduction

This is a simple POC demonstrating WASM functions executed by a common server.

Directory wit contains a global WASM Interface that all WASM Modules need to implement.

## Description

## Build

```shell
cargo build -p fibonacci-faas --target wasm32-wasip1 --release
```