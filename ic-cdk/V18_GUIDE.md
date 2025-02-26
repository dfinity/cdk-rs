# Version 0.18 Guide

## Introduction

`ic-cdk` v0.18 introduces many new features and changes that improve the user experience.
This guide will go through the major features and changes and help you migrate your code from version 0.17 or before.

### How to Upgrade

Update your `Cargo.toml` to use the alpha version of the library:
```toml
[dependencies]
ic-cdk = "0.18.0-alpha.1"
```

> [!NOTE]
> The new version relies on the “Bounded-Wait Calls” feature that is not yet fully enabled on the mainnet.
> To allow users to start experimenting with the new features and provide feedback, we are releasing this version as an alpha (v0.18.0-alpha.1).
> A stable release will follow once the “Bounded-Wait Calls” feature is fully enabled on the mainnet.
>
> The Canister module built with the new Rust CDK is compatible with:
> - `dfx start`: defaults to enable the “Bounded-Wait Calls” feature.
> - `PocketIC`: enable the feature with `PocketIcBuilder::with_nonmainnet_features(true)`.

## Features

### New `Call` API

This version introduces a revamped API for inter-canister calls, utilizing a builder pattern for flexible call configuration and execution.

```rust
use ic_cdk::call::Call;
let id : Principal =...;
let method : &str =...;
let res: u32 = Call::bounded_wait(id, method) // Choose the "bounded-wait" constructor
    .with_arg(42)                             // Specify Candid argument
    .with_cycles(1000)                        // Attach cycles
    .await?                                   // Execute the call by awaiting it
    .candid()?;                               // Decode the response bytes as Candid value
```

Please check the [docs](https://docs.rs/ic-cdk/0.18.0-alpha.1/ic_cdk/call/struct.Call.html) for more details.

#### Migration

The functions for inter-canister calls in the `ic_cdk::api::call` module are deprecated in favor of the new `Call` API. These functions were created before the introduction of the "Bounded-Wait Calls" feature. To maintain the same behavior, use the `Call::unbounded_wait()` constructor. You can later evaluate if a specific call should switch to `Call::bounded_wait()`.

| Before                                             | After                                                                                    |
| -------------------------------------------------- | ---------------------------------------------------------------------------------------- |
| `call(id, method, arg)`                            | `Call::unbounded_wait(id, method).with_arg(arg).await?.candid()?`                        |
| `call_raw(id, method, args_raw, payment)`          | `Call::unbounded_wait(id, method).with_raw_args(args_raw).with_cycles(payment).await?`   |
| `call_raw128(id, method, args_raw, payment)`       | `Call::unbounded_wait(id, method).with_raw_args(args_raw).with_cycles(payment).await?`   |
| `call_with_payment(id, method, arg, payment)`      | `Call::unbounded_wait(id, method).with_arg(arg).with_cycles(payment).await?.candid()?`   |
| `call_with_payment128(id, method, arg, payment)`   | `Call::unbounded_wait(id, method).with_arg(arg).with_cycles(payment).await?.candid()?`   |
| `call_with_config(...)`                            | `DecoderConfig` is no longer supported.                                                  |
| `notify(id, method, arg)`                          | `Call::unbounded_wait(id, method).with_arg(arg).oneway()?`                               |
| `notify_raw(id, method, args_raw, payment)`        | `Call::unbounded_wait(id, method).with_raw_args(arg_raw).with_cycles(payment).oneway()?` |
| `notify_with_payment128(id, method, arg, payment)` | `Call::unbounded_wait(id, method).with_arg(arg).with_cycles(payment).oneway()?`          |

> [!NOTE]
> Some deprecated APIs expected a tuple of Candid values as input arguments. Often, there is a single Candid value that needs to be wrapped in parentheses. Therefore, it is recommended to use the `with_arg()` method, which accepts a single `CandidType` value. Use `with_args()` when specifying a Candid tuple.
>
> Similarly, for response decoding, it is recommended to use `candid()`, which decodes to a single `CandidType`. Use `candid_tuple()` when decoding the response as a Candid tuple.

### Wasm64 Compilation

No changes to the source code are required. However, setting up the Rust toolchain for Wasm64 support requires some additional steps.

1. Install nightly toolchain: 
```bash
rustup toolchain install nightly
```
2. Add rust-src component:
```bash
rustup component add rust-src --toolchain nightly
```
3. Build with necessary flags:
```bash
cargo +nightly build -Z build-std=std,panic_abort --target wasm64-unknown-unknown
```

### Custom Decoders in `update`/`query`/`init` Macros

The macros are enhanced to accept custom decoders.

```rust
// The update method specifies a custom decoder function by its name
#[update(decode_with = "decode_two_u32")]
fn expect_two_u32(a: u32, b: u32) {
    ...
}
// The decoder function should have empty arguments
// and return the same type(s) that the update method expects.
fn decode_two_u32() -> (u32, u32) {
    let arg_bytes = msg_arg_data();
    decode_args(&arg_bytes).unwrap() // decode with any data format not limited to Candid
}
```

### Simplified Module Structure

The module hierarchy has been flattened to improve usability and consistency:
- The `api` module provides consistent System API bindings.
- The `management_canister` module facilitates convenient Management Canister calls.
- The `bitcoin_canister` module will soon support direct Bitcoin Canister calls.

#### Migration

Submodules in `api` are now deprecated in favor of root-level modules.
- `api/call` -> `call`
- `api/management_canister` -> `management_canister` & `bitcoin_canister`
- `api/stable` -> `stable`
