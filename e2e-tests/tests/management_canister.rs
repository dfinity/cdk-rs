use ic_cdk_e2e_tests::cargo_build_canister;

mod common;
use common::*;

#[test]
fn manage_canister() {
    let env = StateMachine::new();
    let rev = cargo_build_canister("call-management");
    let canister_id = env.install_canister(rev, vec![], None).unwrap();

    let (_result,): (CanisterId,) = call_candid(&env, canister_id, "call_create_canister", ())
        .expect("failed to call call_create_canister");
}

#[test]
fn raw_rand() {
    let env = StateMachine::new();
    let rev = cargo_build_canister("call-management");
    let canister_id = env.install_canister(rev, vec![], None).unwrap();

    let (result,): (Vec<u8>,) =
        call_candid(&env, canister_id, "call_raw_rand", ()).expect("failed to call call_raw_rand");
    assert_eq!(result.len(), 32);
}
