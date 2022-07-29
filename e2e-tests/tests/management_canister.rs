use ic_cdk_e2e_tests::cargo_build_canister;

mod common;
use common::*;

#[test]
fn main_canister() {
    let env = StateMachine::new();
    let rev = cargo_build_canister("call-management");
    let canister_id = env.install_canister(rev, vec![], None).unwrap();

    let result: Result<(), _> = call_candid(&env, canister_id, "call_create_canister", ());
    assert!(result.is_ok());

    let result: Result<(), _> = call_candid(&env, canister_id, "call_update_settings", ());
    assert!(result.is_ok());

    let result: Result<(), _> = call_candid(&env, canister_id, "call_install_code", ());
    assert!(result.is_ok());

    let result: Result<(), _> = call_candid(&env, canister_id, "call_uninstall_code", ());
    assert!(result.is_ok());

    let result: Result<(), _> = call_candid(&env, canister_id, "call_start_canister", ());
    assert!(result.is_ok());

    let result: Result<(), _> = call_candid(&env, canister_id, "call_stop_canister", ());
    assert!(result.is_ok());

    let result: Result<(ic_management::CanisterStatusReply,), _> =
        call_candid(&env, canister_id, "call_canister_status", ());
    assert!(result.is_ok());

    let result: Result<(), _> = call_candid(&env, canister_id, "call_deposit_cycles", ());
    assert!(result.is_ok());

    let result: Result<(), _> = call_candid(&env, canister_id, "call_delete_canister", ());
    assert!(result.is_ok());
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

#[test]
fn provisional() {
    let env = StateMachine::new();
    let rev = cargo_build_canister("call-management");
    let canister_id = env.install_canister(rev, vec![], None).unwrap();

    let result: Result<(), _> = call_candid(
        &env,
        canister_id,
        "call_provisional_create_canister_with_cycles",
        (),
    );
    assert!(result.is_ok());

    let result: Result<(), _> =
        call_candid(&env, canister_id, "call_provisional_top_up_canister", ());
    assert!(result.is_ok());
}
