# Rust Canister Development Kit

[![Documentation](https://docs.rs/ic-cdk/badge.svg)](https://docs.rs/ic-cdk/)
[![Crates.io](https://img.shields.io/crates/v/ic-cdk.svg)](https://crates.io/crates/ic-cdk)
[![License](https://img.shields.io/crates/l/ic-cdk.svg)](https://github.com/dfinity/cdk-rs/blob/main/src/ic-cdk/LICENSE)
[![Downloads](https://img.shields.io/crates/d/ic-cdk.svg)](https://crates.io/crates/ic-cdk)
[![Test Status](https://github.com/dfinity/cdk-rs/actions/workflows/test.yml/badge.svg)](https://github.com/dfinity/cdk-rs/actions)

**Rust CDK provides tools for building Canisters on Internet Computer (IC).**

You may be looking for:

- [Documentation Site of the Internet Computer](https://smartcontracts.org/)
- [Tutorials of Rust CDK](https://smartcontracts.org/docs/rust-guide/rust-intro.html)
- [Examples](https://github.com/dfinity/cdk-rs/tree/main/examples)
- [`dfx` for managing IC projects](https://github.com/dfinity/sdk)

If you are looking for a crate to communicate with existing canisters on IC,
you may want to check [agent-rs](https://github.com/dfinity/agent-rs).

# Introduction

A `canister` is a WebAssembly (wasm) module that can run on the Internet Computer.

To be a `canister`, a wasm module should communicate with the execution environment using [Canister interfaces (System API)](https://sdk.dfinity.org/docs/interface-spec/index.html#system-api).

This repo provides libraries and tools to facilitate developing canisters in Rust.

- [`ic-cdk`](https://github.com/dfinity/cdk-rs/tree/main/src/ic-cdk):
Bindings of the System API.
- [`ic-cdk-macros`](https://github.com/dfinity/cdk-rs/tree/main/src/ic-cdk-macros):
Annotate functions with attribute macros to make them exposed public interfaces.
- [`ic-cdk-optimizer`](https://github.com/dfinity/cdk-rs/tree/main/src/ic-cdk-optimizer):
A binary tool to reduing size of the compiled wasm module.
- [`ic-certified-map`](https://github.com/dfinity/cdk-rs/tree/main/src/ic-certified-map): An implementation of map which support *certified queries*.

## Rust CDK in Action

In Cargo.toml:

```toml
[lib]
crate-type = ["cdylib"]

[dependencies]
candid = "0.7.4" # this is required if you want to use the `#[import]` macro
ic-cdk = "0.4"
ic-cdk-macros = "0.4"
```

Then in your rust source code:

```rust
#[ic_cdk_macros::query]
fn print() {
    ic_cdk::print("Hello World from DFINITY!");
}
```

Check [tutorial](https://sdk.dfinity.org/docs/rust-guide/rust-quickstart.html) for a detailed guidance.
