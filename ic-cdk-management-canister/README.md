[![Documentation](https://docs.rs/ic-cdk-management-canister/badge.svg)](https://docs.rs/ic-cdk-management-canister/)
[![Crates.io](https://img.shields.io/crates/v/ic-cdk-management-canister.svg)](https://crates.io/crates/ic-cdk-management-canister)
[![License](https://img.shields.io/crates/l/ic-cdk-management-canister.svg)](https://github.com/dfinity/cdk-rs/blob/main/LICENSE)
[![Downloads](https://img.shields.io/crates/d/ic-cdk-management-canister.svg)](https://crates.io/crates/ic-cdk-management-canister)
[![CI](https://github.com/dfinity/cdk-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/dfinity/cdk-rs/actions/workflows/ci.yml)

# ic-cdk-management-canister

Functions and types for interacting with the [IC management canister](https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-management-canister).

## Type Definitions

This crate defines the types of arguments and results for the management canister entry points.
Most of these types are re-exported from the `ic-management-canister-types` crate.

The only exception is that for the argument types that has a `sender_canister_version` field, this module provides reduced versions instead.
The reduced versions don't need the `sender_canister_version` field as it is set automatically in the corresponding functions.

## Call

The majority of the functions in this module are for making calls to the management canister.

### Bounded-wait vs. Unbounded-wait

Interacting with the IC management canister involves making inter-canister calls,
which can be either [bounded-wait][bounded-wait] or [unbounded-wait][unbounded-wait].
This crate selects the appropriate type of wait call for each method based on the characteristics of the entry point.

The strategy for choosing the type of wait call is as follows:
- Unbounded-wait call by default because the management canister is universally trusted.
- Bounded-wait call (with the default 300s timeout) for methods that only read state.

Please check the documentation of each function for the type of wait call it uses.

If the default behavior is not suitable for a particular use case, the [`Call`] struct can be used directly to make the call.

For example, [`sign_with_ecdsa`] makes an unbounded-wait call. If a bounded-wait call is preferred, the call can be made as follows:
```rust,no_run
use ic_cdk_management_canister::{cost_sign_with_ecdsa, SignCallError, SignWithEcdsaArgs, SignWithEcdsaResult};
use ic_cdk::call::Call;
use candid::Principal;

async fn example() -> Result<SignWithEcdsaResult, SignCallError> {
    let callee = Principal::management_canister();
    let arg = SignWithEcdsaArgs::default();
    let cycles = cost_sign_with_ecdsa(&arg)?;
    let res: SignWithEcdsaResult = Call::bounded_wait(callee, "sign_with_ecdsa")
        .with_arg(&arg)
        .with_cycles(cycles)
        .await?
        .candid()?;
    Ok(res)
}
```

### Cycle Cost

Some management canister entry points require cycles to be attached to the call.
The functions for calling management canister automatically calculate the required cycles and attach them to the call.

For completeness, this module also provides functions to calculate the cycle cost:
- [`cost_http_request`]
- [`cost_sign_with_ecdsa`]
- [`cost_sign_with_schnorr`]
- [`cost_vetkd_derive_key`]

[bounded-wait]: https://docs.rs/ic-cdk/latest/ic_cdk/call/struct.Call.html#method.bounded_wait
[unbounded-wait]: https://docs.rs/ic-cdk/latest/ic_cdk/call/struct.Call.html#method.unbounded_wait
[`Call`]: https://docs.rs/ic-cdk/latest/ic_cdk/call/struct.Call.html
[`sign_with_ecdsa`]: https://docs.rs/ic-cdk-management-canister/latest/ic_cdk_management_canister/fn.sign_with_ecdsa.html
[`cost_http_request`]: https://docs.rs/ic-cdk-management-canister/latest/ic_cdk_management_canister/fn.cost_http_request.html
[`cost_sign_with_ecdsa`]: https://docs.rs/ic-cdk-management-canister/latest/ic_cdk_management_canister/fn.cost_sign_with_ecdsa.html
[`cost_sign_with_schnorr`]: https://docs.rs/ic-cdk-management-canister/latest/ic_cdk_management_canister/fn.cost_sign_with_schnorr.html
[`cost_vetkd_derive_key`]: https://docs.rs/ic-cdk-management-canister/latest/ic_cdk_management_canister/fn.cost_vetkd_derive_key.html
