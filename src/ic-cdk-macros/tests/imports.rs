// https://github.com/dtolnay/trybuild/issues/95
use ic_cdk_macros::import;

#[import(canister_id = "aaaaa-aa", candid_path = "tests/ic.did")]
mod mgmt {
    pub type Nat = u128;
    pub struct CanisterIdContainer {
        canister_id: Principal,
    }
    #[rustfmt::skip] // {} vs ;
    pub async fn create_canister(can: CanisterIdContainer) -> CanisterIdContainer;
}

#[test]
fn it_works() {}
