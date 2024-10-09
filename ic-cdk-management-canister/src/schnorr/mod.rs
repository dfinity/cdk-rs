//! Threshold Schnorr signing API.

use candid::Principal;
use ic_cdk::prelude::*;

mod types;
pub use types::*;

// Source: https://internetcomputer.org/docs/current/references/t-sigs-how-it-works/#fees-for-the-t-schnorr-production-key
const SIGN_WITH_SCHNORR_FEE: u128 = 26_153_846_153;

/// Return a SEC1 encoded Schnorr public key for the given canister using the given derivation path.
///
/// See [IC method `schnorr_public_key`](https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-schnorr_public_key).
pub async fn schnorr_public_key(
    arg: SchnorrPublicKeyArgument,
) -> CallResult<(SchnorrPublicKeyResponse,)> {
    Call::new(Principal::management_canister(), "schnorr_public_key")
        .with_guaranteed_response()
        .with_args((arg,))
        .call()
        .await
}

/// Return a new Schnorr signature of the given message that can be separately verified against a derived Schnorr public key.
///
/// See [IC method `sign_with_schnorr`](https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-sign_with_schnorr).
///
/// This call requires cycles payment.
/// This method handles the cycles cost under the hood.
/// Check [Threshold signatures](https://internetcomputer.org/docs/current/references/t-sigs-how-it-works) for more details.
pub async fn sign_with_schnorr(
    arg: SignWithSchnorrArgument,
) -> CallResult<(SignWithSchnorrResponse,)> {
    Call::new(Principal::management_canister(), "sign_with_schnorr")
        .with_guaranteed_response()
        .with_args((arg,))
        .with_cycles(SIGN_WITH_SCHNORR_FEE)
        .call()
        .await
}
