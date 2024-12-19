use candid::Principal;
use ic_cdk_e2e_tests::cargo_build_canister;
use pocket_ic::{ErrorCode, PocketIcBuilder, UserError, WasmResult};

#[test]
fn call_api() {
    let pic = PocketIcBuilder::new()
        .with_application_subnet()
        .with_nonmainnet_features(true)
        .build();
    let wasm = cargo_build_canister("api");
    let canister_id = pic.create_canister();
    pic.add_cycles(canister_id, 100_000_000_000_000);
    pic.install_canister(canister_id, wasm, vec![], None);
    let sender = Principal::anonymous();
    let res = pic
        .update_call(canister_id, sender, "call_msg_arg_data", vec![42])
        .unwrap();
    assert_eq!(res, WasmResult::Reply(vec![]));
    let res = pic
        .update_call(canister_id, sender, "call_msg_caller", vec![])
        .unwrap();
    assert_eq!(res, WasmResult::Reply(vec![]));
    // `msg_reject_code` and `msg_reject_msg` can't be tested here.
    // They are invoked in the reply/reject callback of inter-canister calls.
    // So the `call.rs` test covers them.
    let res = pic
        .update_call(canister_id, sender, "call_msg_reply", vec![])
        .unwrap();
    assert_eq!(res, WasmResult::Reply(vec![42]));
    let res = pic
        .update_call(canister_id, sender, "call_msg_reject", vec![])
        .unwrap();
    assert_eq!(res, WasmResult::Reject("e2e test reject".to_string()));
    let res = pic
        .update_call(canister_id, sender, "call_msg_cycles_available", vec![])
        .unwrap();
    assert_eq!(res, WasmResult::Reply(vec![]));
    // `msg_cycles_refunded` can't be tested here.
    // It can only be called in the reply/reject callback of inter-canister calls.
    // TODO: Find a way to test it.
    let res = pic
        .update_call(canister_id, sender, "call_msg_cycles_accept", vec![])
        .unwrap();
    assert_eq!(res, WasmResult::Reply(vec![]));
    let res = pic
        .update_call(canister_id, sender, "call_cycles_burn", vec![])
        .unwrap();
    assert_eq!(res, WasmResult::Reply(vec![]));
    let res = pic
        .update_call(
            canister_id,
            sender,
            "call_canister_self",
            canister_id.as_slice().to_vec(),
        )
        .unwrap();
    assert_eq!(res, WasmResult::Reply(vec![]));
    let res = pic
        .update_call(canister_id, sender, "call_canister_cycle_balance", vec![])
        .unwrap();
    assert_eq!(res, WasmResult::Reply(vec![]));
    let res = pic
        .update_call(canister_id, sender, "call_canister_status", vec![])
        .unwrap();
    assert_eq!(res, WasmResult::Reply(vec![]));
    let res = pic
        .update_call(canister_id, sender, "call_canister_version", vec![])
        .unwrap();
    assert_eq!(res, WasmResult::Reply(vec![]));
    // `msg_method_name` and `accept_message` are invoked in the inspect_message entry point.
    // Every calls above/below execute the inspect_message entry point.
    // So these two API bindings are tested implicitly.
    let res = pic
        .update_call(canister_id, sender, "call_certified_data_set", vec![])
        .unwrap();
    assert_eq!(res, WasmResult::Reply(vec![]));
    let res = pic
        .query_call(canister_id, sender, "call_data_certificate", vec![])
        .unwrap();
    assert_eq!(res, WasmResult::Reply(vec![]));
    let res = pic
        .update_call(canister_id, sender, "call_time", vec![])
        .unwrap();
    assert_eq!(res, WasmResult::Reply(vec![]));
    // `global_timer_set` is tested in `timers.rs`.
    let res = pic
        .update_call(canister_id, sender, "call_performance_counter", vec![])
        .unwrap();
    assert_eq!(res, WasmResult::Reply(vec![]));
    let res = pic
        .update_call(canister_id, sender, "call_is_controller", vec![])
        .unwrap();
    assert_eq!(res, WasmResult::Reply(vec![]));
    let res = pic
        .update_call(canister_id, sender, "call_in_replicated_execution", vec![])
        .unwrap();
    assert_eq!(res, WasmResult::Reply(vec![1]));
    let res = pic
        .query_call(canister_id, sender, "call_in_replicated_execution", vec![])
        .unwrap();
    assert_eq!(res, WasmResult::Reply(vec![0]));
    let res = pic
        .update_call(canister_id, sender, "call_debug_print", vec![])
        .unwrap();
    assert_eq!(res, WasmResult::Reply(vec![]));
    let UserError { code, description } = pic
        .update_call(canister_id, sender, "call_trap", vec![])
        .unwrap_err();
    assert_eq!(code, ErrorCode::CanisterCalledTrap);
    assert!(description.contains("It's a trap!"));
}
