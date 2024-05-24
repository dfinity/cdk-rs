use candid::Principal;
use ic_cdk::api::call::{Call, Sendable};
use ic_cdk::api::management_canister::main::{CanisterIdRecord, CreateCanisterArgument};
use ic_cdk::update;

#[update]
async fn create_canister_via_struct() -> Principal{
    let res: (CanisterIdRecord,) = Call::new(Principal::management_canister(), "create_canister")
        .with_args((CreateCanisterArgument::default(),))
        .call()
        .await
        .unwrap();
    res.0.canister_id
}

fn main() {}
