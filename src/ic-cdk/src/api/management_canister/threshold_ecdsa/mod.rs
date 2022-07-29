use candid::Principal;
use ic_cdk::api::call::{call, CallResult};

mod types;
pub use types::*;

/**
  ecdsa_public_key : (record {
    canister_id : opt canister_id;
    derivation_path : vec blob;
    key_id : record { curve: ecdsa_curve; name: text };
  }) -> (record { public_key : blob; chain_code : blob; });
*/
pub async fn ecdsa_public_key(arg: EcdsaPublicKeyArgument) -> CallResult<(EcdsaPublicKeyReply,)> {
    call(Principal::management_canister(), "ecdsa_public_key", (arg,)).await
}

/**
  sign_with_ecdsa : (record {
    message_hash : blob;
    derivation_path : vec blob;
    key_id : record { curve: ecdsa_curve; name: text };
  }) -> (record { signature : blob });
*/
pub async fn sign_with_ecdsa(arg: SignWithEcdsaArgument) -> CallResult<(SignWithEcdsaReply,)> {
    call(Principal::management_canister(), "sign_with_ecdsa", (arg,)).await
}
