use ic_cdk_macros::*;

#[import(canister = "inter_mo")]
struct CounterCanister;

#[update]
async fn read() -> u64 {
    CounterCanister::read().await
}

#[update]
async fn inc() -> () {
    CounterCanister::inc().await
}

#[update]
async fn write(input: u64) -> () {
    CounterCanister::write(input).await
}
