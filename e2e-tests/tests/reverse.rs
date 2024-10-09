use candid::Principal;
use ic_cdk_e2e_tests::cargo_build_canister;
use pocket_ic::{PocketIc, WasmResult};

#[test]
fn test_raw_api() {
    let pic = PocketIc::new();
    let wasm = cargo_build_canister("reverse");
    let canister_id = pic.create_canister();
    pic.add_cycles(canister_id, 2_000_000_000_000);
    pic.install_canister(canister_id, wasm, vec![], None);

    let result = pic
        .query_call(
            canister_id,
            Principal::anonymous(),
            "reverse",
            vec![1, 2, 3, 4],
        )
        .unwrap();
    assert_eq!(result, WasmResult::Reply(vec![4, 3, 2, 1]));

    let result = pic
        .update_call(
            canister_id,
            Principal::anonymous(),
            "empty_call",
            Default::default(),
        )
        .unwrap();
    assert_eq!(result, WasmResult::Reply(Default::default()));
}
