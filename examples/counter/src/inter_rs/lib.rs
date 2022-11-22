use ic_cdk::{export::candid, import, update};

#[import(canister = "counter_mo")]
struct CounterCanister;

#[update]
async fn read() -> candid::Nat {
    CounterCanister::read().await.0
}

#[update]
async fn inc() {
    CounterCanister::inc().await
}

#[update]
async fn write(input: candid::Nat) {
    CounterCanister::write(input).await
}
