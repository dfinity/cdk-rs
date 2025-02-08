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
ic-cdk = "0.15"
candid = "0.10" # required if you want to define Candid data types
```

Then in Rust source code:

```rust
#[ic_cdk::query]
fn hello() -> String {
    "world".to_string()
}
```

This will register a **query** entry point named `hello`.

## Macros

This library re-exports macros defined in `ic-cdk-macros` crate.

The macros fall into two categories:

* To register functions as canister entry points
* To export Candid definitions

### Register functions as canister entry points

These macros are directly related to the [Internet Computer Specification](https://internetcomputer.org/docs/current/references/ic-interface-spec#entry-points).

* [`init`](https://docs.rs/ic-cdk/latest/ic_cdk/attr.init.html)
* [`pre_upgrade`](https://docs.rs/ic-cdk/latest/ic_cdk/attr.pre_upgrade.html)
* [`post_upgrade`](https://docs.rs/ic-cdk/latest/ic_cdk/attr.post_upgrade.html)
* [`inspect_message`](https://docs.rs/ic-cdk/latest/ic_cdk/attr.inspect_message.html)
* [`heartbeat`](https://docs.rs/ic-cdk/latest/ic_cdk/attr.heartbeat.html)
* [`on_low_wasm_memory`](https://docs.rs/ic-cdk/latest/ic_cdk/attr.on_low_wasm_memory.html)
* [`update`](https://docs.rs/ic-cdk/latest/ic_cdk/attr.update.html)
* [`query`](https://docs.rs/ic-cdk/latest/ic_cdk/attr.query.html)

### Export Candid definitions

* [`export_candid`](https://docs.rs/ic-cdk/latest/ic_cdk/macro.export_candid.html)

Check [Generating Candid files for Rust canisters](https://internetcomputer.org/docs/current/developer-docs/backend/candid/generating-candid/) for more details.

## More examples

* [Basic examples](https://github.com/dfinity/cdk-rs/tree/main/examples): Demonstrate usage of `ic-cdk` API.
* [Comprehensive examples](https://github.com/dfinity/examples/tree/master/rust): Illustrate how to build useful Rust canisters.

## Manage Data Structure in Stable Memory

Using the `ic_cdk::storage::{stable_save, stable_restore}` API is easy but it doesn't scale well.

[`ic-stable-structures`](https://crates.io/crates/ic-stable-structures) is recommended when you are dealing with multiple data structures with larger datasets.
