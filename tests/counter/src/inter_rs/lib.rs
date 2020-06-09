use ic_cdk_macros::*;

#[import(canister = "counter_mo")]
struct CounterCanister;

#[update]
async fn read() -> candid::Nat {
    CounterCanister::read().await
}

#[update]
async fn inc() -> () {
    CounterCanister::inc().await
}

#[update]
async fn write(input: candid::Nat) -> () {
    CounterCanister::write(input).await
}
