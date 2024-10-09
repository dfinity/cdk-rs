//! Threshold ECDSA signing API.

use candid::Principal;
use ic_cdk::prelude::*;

mod types;
pub use types::*;

const SIGN_WITH_ECDSA_FEE: u128 = 26_153_846_153;

/// Return a SEC1 encoded ECDSA public key for the given canister using the given derivation path.
///
/// See [IC method `ecdsa_public_key`](https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-ecdsa_public_key).
pub async fn ecdsa_public_key(
    arg: EcdsaPublicKeyArgument,
) -> CallResult<(EcdsaPublicKeyResponse,)> {
    Call::new(Principal::management_canister(), "ecdsa_public_key")
        .with_guaranteed_response()
        .with_args((arg,))
        .call()
        .await
}

/// Return a new ECDSA signature of the given message_hash that can be separately verified against a derived ECDSA public key.
///
/// See [IC method `sign_with_ecdsa`](https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-sign_with_ecdsa).
///
/// This call requires cycles payment.
/// This method handles the cycles cost under the hood.
/// Check [Threshold signatures](https://internetcomputer.org/docs/current/references/t-sigs-how-it-works) for more details.
pub async fn sign_with_ecdsa(arg: SignWithEcdsaArgument) -> CallResult<(SignWithEcdsaResponse,)> {
    Call::new(Principal::management_canister(), "sign_with_ecdsa")
        .with_guaranteed_response()
        .with_args((arg,))
        .with_cycles(SIGN_WITH_ECDSA_FEE)
        .call()
        .await
}
