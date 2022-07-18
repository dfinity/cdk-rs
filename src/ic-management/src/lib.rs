use ic_cdk::api::call::{call, CallResult};
use ic_cdk::export::Principal;

// raw_rand : () -> (blob);
pub async fn raw_rand() -> CallResult<(Vec<u8>,)> {
    call(Principal::management_canister(), "raw_rand", ()).await
}
