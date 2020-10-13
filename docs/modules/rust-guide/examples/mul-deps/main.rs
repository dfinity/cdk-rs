use ic_cdk_macros::*;

#[import(canister = "multiply_deps")]
struct CounterCanister;

#[query]
async fn read() -> candid::Nat {
    CounterCanister::read().await.0
}

