use ic_cdk_macros::*;
use ic_cdk::export::candid;

#[import(canister = "multiply_deps")]
mod counter_canister {}

#[query]
async fn read() -> candid::Nat {
    counter_canister::read().await
}

