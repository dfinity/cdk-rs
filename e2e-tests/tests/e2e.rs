use candid::{Decode, Encode};
use ic_cdk_e2e_tests::cargo_build_canister;
use ic_ic00_types::CanisterInstallMode;
use ic_state_machine_tests::StateMachine;
use serde_bytes::ByteBuf;

#[test]
fn test_storage_roundtrip() {
    let env = StateMachine::new();
    let kv_store_wasm = cargo_build_canister("simple-kv-store");
    let canister_id = env
        .install_canister(kv_store_wasm.clone(), vec![], None)
        .unwrap();

    env.execute_ingress(canister_id, "insert", Encode!(&"candid", &b"did").unwrap())
        .expect("failed to insert 'candid'");

    env.install_wasm_in_mode(
        canister_id,
        CanisterInstallMode::Upgrade,
        kv_store_wasm,
        vec![],
    )
    .expect("failed to upgrade the simple-kv-store canister");

    let result = Decode!(
        &env.query(canister_id, "lookup", Encode!(&"candid").unwrap(),)
            .unwrap()
            .bytes(),
        Option<ByteBuf>
    )
    .unwrap();

    assert_eq!(result, Some(ByteBuf::from(b"did".to_vec())));
}
