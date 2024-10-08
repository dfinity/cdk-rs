use candid::Principal;
use ic_cdk::prelude::*;
use ic_cdk_management_canister::core::{CanisterIdRecord, CreateCanisterArgument};

#[update]
async fn create_canister_via_struct() -> Principal {
    let res: (CanisterIdRecord,) = Call::new(Principal::management_canister(), "create_canister")
        .with_args((CreateCanisterArgument::default(),))
        .with_cycles(200_000_000_000)
        .with_guaranteed_response()
        .call()
        .await
        .unwrap();
    res.0.canister_id
}

fn main() {}
