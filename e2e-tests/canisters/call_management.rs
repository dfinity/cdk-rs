use ic_cdk_macros::update;
use ic_management::*;

#[update]
async fn call_create_canister() -> CanisterId {
    create_canister(CreateCanisterArgument::default()).await.unwrap().0.canister_id
}

#[update]
async fn call_raw_rand() -> Vec<u8> {
    raw_rand().await.unwrap().0
}

fn main() {}