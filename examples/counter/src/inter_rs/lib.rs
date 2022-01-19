use ic_cdk_macros::*;
use ic_cdk::export::candid;

#[import(canister = "counter_mo")]
mod counter_canister {}

#[update]
async fn read() -> candid::Nat {
    counter_canister::read().await
}

#[update]
async fn inc() -> () {
    counter_canister::inc().await
}

#[update]
async fn write(input: candid::Nat) -> () {
    counter_canister::write(input).await
}
