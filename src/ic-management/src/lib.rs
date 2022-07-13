use ic_cdk::export::Principal;
use ::ic_cdk::api::call::{call, CallResult};

// raw_rand : () -> (blob);
pub async fn raw_rand() -> CallResult<Vec<u8>> {
    call(Principal::management_canister(), "start_canister", ())
        .await
}
