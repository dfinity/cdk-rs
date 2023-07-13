# Rust Canister Development Kit

[![Documentation](https://docs.rs/ic-cdk/badge.svg)](https://docs.rs/ic-cdk/)
[![Crates.io](https://img.shields.io/crates/v/ic-cdk.svg)](https://crates.io/crates/ic-cdk)
[![License](https://img.shields.io/crates/l/ic-cdk.svg)](https://github.com/dfinity/cdk-rs/blob/main/src/ic-cdk/LICENSE)
[![Downloads](https://img.shields.io/crates/d/ic-cdk.svg)](https://crates.io/crates/ic-cdk)
[![CI](https://github.com/dfinity/cdk-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/dfinity/cdk-rs/actions/workflows/ci.yml)

**Rust CDK provides tools for building Canisters on Internet Computer (IC).**

You may be looking for:

- [Documentation Site of the Internet Computer](https://internetcomputer.org/docs)
- [Tutorials of Rust CDK](https://internetcomputer.org/docs/current/developer-docs/build/cdks/cdk-rs-dfinity/)
- [Examples](https://github.com/dfinity/cdk-rs/tree/main/examples)
- [`dfx` for managing IC projects](https://github.com/dfinity/sdk)

If you are looking for a crate to communicate with existing canisters on IC,
you may want to check [agent-rs](https://github.com/dfinity/agent-rs).

# Introduction

A `canister` is a WebAssembly (wasm) module that can run on the Internet Computer.

To be a `canister`, a wasm module should communicate with the execution environment using [Canister interfaces (System API)](https://internetcomputer.org/docs/current/references/ic-interface-spec/#system-api).

This repo provides libraries and tools to facilitate developing canisters in Rust.

- [`ic0`](src/ic0):
Internet Computer System API binding.
- [`ic-cdk`](src/ic-cdk):
Internet Computer Canister Development Kit
- [`ic-cdk-bindgen`](src/ic-cdk-bindgen):
Generate Rust bindings from Candid to make inter-canister calls.
- [`ic-cdk-macros`](src/ic-cdk-macros):
Annotate functions with attribute macros to make them exposed public interfaces.
- [`ic-cdk-timers`](src/ic-cdk-timers):
The library implements multiple and periodic timers.
- [`ic-certified-map`](library/ic-certified-map):
An implementation of map which support *certified queries*.
- [`ic-ledger-types`](library/ic-ledger-types):
Type definitions to communicate with the ICP ledger canister.

## Rust CDK in Action

In Cargo.toml:

```toml
[lib]
crate-type = ["cdylib"]

[dependencies]
candid = "0.9" # this version is required if you want to define Candid data types
ic-cdk = "0.10"
```

Then in your rust source code:

```rust
#[ic_cdk::query]
fn print() {
    ic_cdk::print("Hello World from DFINITY!");
}
```

Check [tutorial](https://internetcomputer.org/docs/current/developer-docs/build/cdks/cdk-rs-dfinity/rust-quickstart) for a detailed guidance.
