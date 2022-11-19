use ic_cdk::{import, update, export::candid};

#[import(canister = "multiply_deps")]
struct CounterCanister;

#[update]
async fn read() -> candid::Nat {
    CounterCanister::read().await.0
}
