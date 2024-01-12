[![Documentation](https://docs.rs/ic-cdk/badge.svg)](https://docs.rs/ic-cdk/)
[![Crates.io](https://img.shields.io/crates/v/ic-cdk.svg)](https://crates.io/crates/ic-cdk)
[![License](https://img.shields.io/crates/l/ic-cdk.svg)](https://github.com/dfinity/cdk-rs/blob/main/src/ic-cdk/LICENSE)
[![Downloads](https://img.shields.io/crates/d/ic-cdk.svg)](https://crates.io/crates/ic-cdk)
[![CI](https://github.com/dfinity/cdk-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/dfinity/cdk-rs/actions/workflows/ci.yml)

# ic-cdk

Canister Developer Kit for the Internet Computer.

## Background

On the Internet Computer, smart contracts come in the form of canisters which are WebAssembly modules.

Canisters expose entry points which can be called both by other canisters and by parties external to the IC.

This library aims to provide a Rust-ergonomic abstraction to implement Canister entry points.

## Using `ic-cdk`

In Cargo.toml:

```toml
[lib]
crate-type = ["cdylib"]

[dependencies]
ic-cdk = "0.12"
# Only necessary if you want to define Candid data types
candid = "0.10" 
```

Then in Rust source code:

```rust
#[ic_cdk::query]
fn hello() -> String{
    "world".to_string()
}
```

This will register a **query** entry point named `hello`.

## More examples

* [Basic examples](https://github.com/dfinity/cdk-rs/tree/main/examples): Demonstrate usage of `ic-cdk` API.
* [Comprehensive examples](https://github.com/dfinity/examples/tree/master/rust): Illustrate how to build useful Rust canisters.

## Manage Data Structure in Stable Memory

Using `ic_cdk::storage::{stable_save, stable_restore}` API is easy but it doesn't scale well.

[`ic-stable-structures`](https://crates.io/crates/ic-stable-structures) is recommended when you are dealing with multiple data structures with larger datasets.
