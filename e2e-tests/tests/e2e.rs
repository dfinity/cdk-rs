use std::time::Duration;

use candid::{Encode, Principal};
use ic_cdk_e2e_tests::cargo_build_canister;
use ic_test_state_machine_client::{
    call_candid, query_candid, CallError, ErrorCode, StateMachine, WasmResult,
};
use serde_bytes::ByteBuf;

pub static STATE_MACHINE_BINARY: &str = "../ic-test-state-machine";

/// Checks that a canister that uses [`ic_cdk::storage::stable_store`]
/// and [`ic_cdk::storage::stable_restore`] functions can keep its data
/// across upgrades.
#[test]
fn test_storage_roundtrip() {
    let env = StateMachine::new(STATE_MACHINE_BINARY, false);
    let wasm = cargo_build_canister("simple-kv-store");
    let canister_id = env.create_canister();
    env.install_canister(canister_id, wasm.clone(), vec![]);

    let () = call_candid(&env, canister_id, "insert", (&"candid", &b"did"))
        .expect("failed to insert 'candid'");

    env.upgrade_canister(canister_id, wasm, vec![])
        .expect("failed to upgrade the simple-kv-store canister");

    let (result,): (Option<ByteBuf>,) =
        query_candid(&env, canister_id, "lookup", (&"candid",)).expect("failed to lookup 'candid'");
    assert_eq!(result, Some(ByteBuf::from(b"did".to_vec())));
}

#[test]
fn test_panic_after_async_frees_resources() {
    let env = StateMachine::new(STATE_MACHINE_BINARY, false);
    let wasm = cargo_build_canister("async");
    let canister_id = env.create_canister();
    env.install_canister(canister_id, wasm, vec![]);

    for i in 1..3 {
        match call_candid(&env, canister_id, "panic_after_async", ()) {
            Ok(()) => (),
            Err(CallError::Reject(msg)) => panic!("unexpected reject: {}", msg),
            Err(CallError::UserError(e)) => {
                println!("Got a user error as expected: {}", e);

                assert_eq!(e.code, ErrorCode::CanisterCalledTrap);
                let expected_message = "Goodbye, cruel world.";
                assert!(
                    e.description.contains(expected_message),
                    "Expected the user error to contain '{}', got: {}",
                    expected_message,
                    e.description
                );
            }
        }

        let (n,): (u64,) = call_candid(&env, canister_id, "invocation_count", ())
            .expect("failed to call invocation_count");

        assert_eq!(i, n, "expected the invocation count to be {}, got {}", i, n);
    }

    let (message,): (String,) =
        call_candid(&env, canister_id, "invalid_reply_payload_does_not_trap", ())
            .expect("call failed");
    assert_eq!(&message, "handled decoding error gracefully with code 5");
}

#[test]
fn test_raw_api() {
    let env = StateMachine::new(STATE_MACHINE_BINARY, false);
    let wasm = cargo_build_canister("reverse");
    let canister_id = env.create_canister();
    env.install_canister(canister_id, wasm, vec![]);

    let result = env
        .query_call(
            canister_id,
            Principal::anonymous(),
            "reverse",
            vec![1, 2, 3, 4],
        )
        .unwrap();
    assert_eq!(result, WasmResult::Reply(vec![4, 3, 2, 1]));

    let result = env
        .update_call(
            canister_id,
            Principal::anonymous(),
            "empty_call",
            Default::default(),
        )
        .unwrap();
    assert_eq!(result, WasmResult::Reply(Default::default()));
}

#[test]
fn test_notify_calls() {
    let env = StateMachine::new(STATE_MACHINE_BINARY, false);
    let wasm = cargo_build_canister("async");
    let sender_id = env.create_canister();
    env.install_canister(sender_id, wasm.clone(), vec![]);
    let receiver_id = env.create_canister();
    env.install_canister(receiver_id, wasm, vec![]);

    let (n,): (u64,) = query_candid(&env, receiver_id, "notifications_received", ())
        .expect("failed to query 'notifications_received'");
    assert_eq!(n, 0);

    let () = call_candid(&env, sender_id, "notify", (receiver_id, "on_notify"))
        .expect("failed to call 'notify'");

    let (n,): (u64,) = query_candid(&env, receiver_id, "notifications_received", ())
        .expect("failed to query 'notifications_received'");
    assert_eq!(n, 1);
}

// Composite queries are not enabled yet.
#[test]
#[ignore]
fn test_composite_query() {
    let env = StateMachine::new(STATE_MACHINE_BINARY, false);
    let wasm = cargo_build_canister("async");
    let sender_id = env.create_canister();
    env.install_canister(sender_id, wasm.clone(), vec![]);
    let receiver_id = env.create_canister();
    env.install_canister(receiver_id, wasm, vec![]);

    let (greeting,): (String,) = query_candid(&env, sender_id, "greet_self", (receiver_id,))
        .expect("failed to query 'greet_self'");
    assert_eq!(greeting, "Hello, myself");
}

#[test]
fn test_api_call() {
    let env = StateMachine::new(STATE_MACHINE_BINARY, false);
    let wasm = cargo_build_canister("api-call");
    let canister_id = env.create_canister();
    env.install_canister(canister_id, wasm, vec![]);
    let (result,): (u64,) = query_candid(&env, canister_id, "instruction_counter", ())
        .expect("failed to query instruction_counter");
    assert!(result > 0);

    let result = env
        .query_call(
            canister_id,
            Principal::anonymous(),
            "manual_reject",
            Encode!().unwrap(),
        )
        .unwrap();
    assert_eq!(result, WasmResult::Reject("manual reject".to_string()));
}

#[test]
fn test_timers() {
    let env = StateMachine::new(STATE_MACHINE_BINARY, false);
    let wasm = cargo_build_canister("timers");
    let canister_id = env.create_canister();
    env.install_canister(canister_id, wasm, vec![]);

    call_candid::<(), ()>(&env, canister_id, "schedule", ()).expect("Failed to call schedule");
    advance_seconds(&env, 5);

    call_candid::<_, ()>(&env, canister_id, "schedule_long", ())
        .expect("Failed to call schedule_long");
    advance_seconds(&env, 5);
    call_candid::<_, ()>(&env, canister_id, "cancel_long", ()).expect("Failed to call cancel_long");
    advance_seconds(&env, 5);

    call_candid::<_, ()>(&env, canister_id, "start_repeating", ())
        .expect("Failed to call start_repeating");
    advance_seconds(&env, 3);
    call_candid::<_, ()>(&env, canister_id, "stop_repeating", ())
        .expect("Failed to call stop_repeating");
    advance_seconds(&env, 2);

    let (events,): (Vec<String>,) =
        query_candid(&env, canister_id, "get_events", ()).expect("Failed to call get_events");
    assert_eq!(
        events[..],
        ["1", "2", "3", "4", "repeat", "repeat", "repeat"]
    );
}

#[test]
fn test_timers_can_cancel_themselves() {
    let env = StateMachine::new(STATE_MACHINE_BINARY, false);
    let wasm = cargo_build_canister("timers");
    let canister_id = env.create_canister();
    env.install_canister(canister_id, wasm, vec![]);

    call_candid::<_, ()>(&env, canister_id, "set_self_cancelling_timer", ())
        .expect("Failed to call set_self_cancelling_timer");
    call_candid::<_, ()>(&env, canister_id, "set_self_cancelling_periodic_timer", ())
        .expect("Failed to call set_self_cancelling_periodic_timer");

    advance_seconds(&env, 1);

    let (events,): (Vec<String>,) =
        query_candid(&env, canister_id, "get_events", ()).expect("Failed to call get_events");
    assert_eq!(
        events,
        ["timer cancelled self", "periodic timer cancelled self"]
    );
}

#[test]
fn test_scheduling_many_timers() {
    // Must be more than the queue limit (500)
    let timers_to_schedule = 1_000;
    let env = StateMachine::new(STATE_MACHINE_BINARY, false);
    let wasm = cargo_build_canister("timers");
    let canister_id = env.create_canister();
    env.install_canister(canister_id, wasm, vec![]);

    let () = call_candid(
        &env,
        canister_id,
        "schedule_n_timers",
        (timers_to_schedule,),
    )
    .expect("Error calling schedule_n_timers");

    // Up to 500 timers will be executed per round
    advance_seconds(&env, timers_to_schedule / 500);

    let (executed_timers,): (u32,) = query_candid(&env, canister_id, "executed_timers", ())
        .expect("Error querying executed_timers");

    assert_eq!(timers_to_schedule, executed_timers);
}

fn advance_seconds(env: &StateMachine, seconds: u32) {
    for _ in 0..seconds {
        env.advance_time(Duration::from_secs(1));
        env.tick();
    }
}
