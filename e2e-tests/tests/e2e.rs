use std::time::Duration;
use std::time::SystemTime;

use candid::utils::ArgumentDecoder;
use candid::utils::ArgumentEncoder;
use candid::{Encode, Principal};
use ic_cdk::api::management_canister::main::{
    CanisterChange, CanisterChangeDetails, CanisterChangeOrigin, CanisterIdRecord,
    CanisterInfoResponse, CanisterInstallMode,
    CodeDeploymentMode::{Install, Reinstall, Upgrade},
    CodeDeploymentRecord, ControllersChangeRecord, CreationRecord, FromCanisterRecord,
    FromUserRecord, InstallCodeArgument,
};
use ic_cdk_e2e_tests::cargo_build_canister;
use pocket_ic::common::rest::RawEffectivePrincipal;
use pocket_ic::{call_candid_as, query_candid, CallError, ErrorCode, PocketIc, WasmResult};

use serde_bytes::ByteBuf;
use sha2::Digest;

// 2T cycles
const INIT_CYCLES: u128 = 2_000_000_000_000;

/// wrapper around `pocket_ic::call_candid` that uses None as the effective principal.
fn call_candid<Input, Output>(
    env: &PocketIc,
    canister_id: Principal,
    method: &str,
    input: Input,
) -> Result<Output, pocket_ic::CallError>
where
    Input: ArgumentEncoder,
    Output: for<'a> ArgumentDecoder<'a>,
{
    pocket_ic::call_candid(env, canister_id, RawEffectivePrincipal::None, method, input)
}

/// Checks that a canister that uses [`ic_cdk::storage::stable_store`]
/// and [`ic_cdk::storage::stable_restore`] functions can keep its data
/// across upgrades.
#[test]
fn test_storage_roundtrip() {
    let pic = PocketIc::new();
    let wasm = cargo_build_canister("simple-kv-store");
    let canister_id = pic.create_canister();
    pic.add_cycles(canister_id, INIT_CYCLES);
    pic.install_canister(canister_id, wasm.clone(), vec![], None);

    let () = call_candid(&pic, canister_id, "insert", (&"candid", &b"did"))
        .expect("failed to insert 'candid'");

    pic.upgrade_canister(canister_id, wasm, vec![], None)
        .expect("failed to upgrade the simple-kv-store canister");

    let (result,): (Option<ByteBuf>,) =
        query_candid(&pic, canister_id, "lookup", (&"candid",)).expect("failed to lookup 'candid'");
    assert_eq!(result, Some(ByteBuf::from(b"did".to_vec())));
}

#[test]
fn test_panic_after_async_frees_resources() {
    let pic: PocketIc = PocketIc::new();
    let wasm = cargo_build_canister("async");
    let canister_id = pic.create_canister();
    pic.add_cycles(canister_id, INIT_CYCLES);
    pic.install_canister(canister_id, wasm, vec![], None);

    for i in 1..3 {
        match call_candid(&pic, canister_id, "panic_after_async", ()) {
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

        let (n,): (u64,) = call_candid(&pic, canister_id, "invocation_count", ())
            .expect("failed to call invocation_count");

        assert_eq!(i, n, "expected the invocation count to be {}, got {}", i, n);
    }

    let (message,): (String,) =
        call_candid(&pic, canister_id, "invalid_reply_payload_does_not_trap", ())
            .expect("call failed");
    assert_eq!(&message, "handled decoding error gracefully with code 5");

    let err =
        call_candid::<_, ()>(&pic, canister_id, "panic_twice", ()).expect_err("failed to panic");
    assert!(
        matches!(err, CallError::UserError(u) if u.description.contains("Call already trapped"))
    );
    let _: (u64,) = call_candid(&pic, canister_id, "notifications_received", ())
        .expect("failed to call unrelated function afterwards");
    let _: (u64,) =
        call_candid(&pic, canister_id, "invocation_count", ()).expect("failed to recover lock");
}

#[test]
fn test_raw_api() {
    let pic = PocketIc::new();
    let wasm = cargo_build_canister("reverse");
    let canister_id = pic.create_canister();
    pic.add_cycles(canister_id, INIT_CYCLES);
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

#[test]
fn test_notify_calls() {
    let pic = PocketIc::new();
    let wasm = cargo_build_canister("async");
    let sender_id = pic.create_canister();
    pic.add_cycles(sender_id, INIT_CYCLES);
    pic.install_canister(sender_id, wasm.clone(), vec![], None);
    let receiver_id = pic.create_canister();
    pic.add_cycles(receiver_id, INIT_CYCLES);
    pic.install_canister(receiver_id, wasm, vec![], None);

    let (n,): (u64,) = query_candid(&pic, receiver_id, "notifications_received", ())
        .expect("failed to query 'notifications_received'");
    assert_eq!(n, 0);

    let () = call_candid(&pic, sender_id, "notify", (receiver_id, "on_notify"))
        .expect("failed to call 'notify'");

    let (n,): (u64,) = query_candid(&pic, receiver_id, "notifications_received", ())
        .expect("failed to query 'notifications_received'");
    assert_eq!(n, 1);
}

// Composite queries are not enabled yet.
#[test]
#[ignore]
fn test_composite_query() {
    let pic = PocketIc::new();
    let wasm = cargo_build_canister("async");
    let sender_id = pic.create_canister();
    pic.add_cycles(sender_id, INIT_CYCLES);
    pic.install_canister(sender_id, wasm.clone(), vec![], None);
    let receiver_id = pic.create_canister();
    pic.add_cycles(receiver_id, INIT_CYCLES);
    pic.install_canister(receiver_id, wasm, vec![], None);

    let (greeting,): (String,) = query_candid(&pic, sender_id, "greet_self", (receiver_id,))
        .expect("failed to query 'greet_self'");
    assert_eq!(greeting, "Hello, myself");
}

#[test]
fn test_api_call() {
    let pic = PocketIc::new();
    let wasm = cargo_build_canister("api-call");
    let canister_id = pic.create_canister();
    pic.add_cycles(canister_id, INIT_CYCLES);
    pic.install_canister(canister_id, wasm, vec![], None);
    let (result,): (u64,) = query_candid(&pic, canister_id, "instruction_counter", ())
        .expect("failed to query instruction_counter");
    assert!(result > 0);

    let result = pic
        .query_call(
            canister_id,
            Principal::anonymous(),
            "manual_reject",
            Encode!().unwrap(),
        )
        .unwrap();
    assert_eq!(result, WasmResult::Reject("manual reject".to_string()));

    let (result,): (bool,) = call_candid(&pic, canister_id, "update_is_replicated", ())
        .expect("Failed to call update_is_replicated");
    assert!(result);

    let (result,): (bool,) = query_candid(&pic, canister_id, "query_is_not_replicated", ())
        .expect("Failed to call query_is_not_replicated");
    assert!(!result);
}

#[test]
fn test_timers() {
    let pic = PocketIc::new();
    let wasm = cargo_build_canister("timers");
    let canister_id = pic.create_canister();
    pic.add_cycles(canister_id, INIT_CYCLES);
    pic.install_canister(canister_id, wasm, vec![], None);

    call_candid::<(), ()>(&pic, canister_id, "schedule", ()).expect("Failed to call schedule");
    advance_seconds(&pic, 5);

    call_candid::<_, ()>(&pic, canister_id, "schedule_long", ())
        .expect("Failed to call schedule_long");
    advance_seconds(&pic, 5);
    call_candid::<_, ()>(&pic, canister_id, "cancel_long", ()).expect("Failed to call cancel_long");
    advance_seconds(&pic, 5);

    call_candid::<_, ()>(&pic, canister_id, "start_repeating", ())
        .expect("Failed to call start_repeating");
    advance_seconds(&pic, 3);
    call_candid::<_, ()>(&pic, canister_id, "stop_repeating", ())
        .expect("Failed to call stop_repeating");
    advance_seconds(&pic, 2);

    let (events,): (Vec<String>,) =
        query_candid(&pic, canister_id, "get_events", ()).expect("Failed to call get_events");
    assert_eq!(
        events[..],
        ["1", "2", "3", "4", "repeat", "repeat", "repeat"]
    );
}

#[test]
fn test_timers_can_cancel_themselves() {
    let pic = PocketIc::new();
    let wasm = cargo_build_canister("timers");
    let canister_id = pic.create_canister();
    pic.add_cycles(canister_id, INIT_CYCLES);
    pic.install_canister(canister_id, wasm, vec![], None);

    call_candid::<_, ()>(&pic, canister_id, "set_self_cancelling_timer", ())
        .expect("Failed to call set_self_cancelling_timer");
    call_candid::<_, ()>(&pic, canister_id, "set_self_cancelling_periodic_timer", ())
        .expect("Failed to call set_self_cancelling_periodic_timer");

    advance_seconds(&pic, 1);

    let (events,): (Vec<String>,) =
        query_candid(&pic, canister_id, "get_events", ()).expect("Failed to call get_events");
    assert_eq!(
        events,
        ["timer cancelled self", "periodic timer cancelled self"]
    );
}

#[test]
fn test_scheduling_many_timers() {
    // Must be more than the queue limit (500)
    let timers_to_schedule = 1_000;
    let pic = PocketIc::new();
    let wasm = cargo_build_canister("timers");
    let canister_id = pic.create_canister();
    pic.add_cycles(canister_id, INIT_CYCLES);
    pic.install_canister(canister_id, wasm, vec![], None);

    let () = call_candid(
        &pic,
        canister_id,
        "schedule_n_timers",
        (timers_to_schedule,),
    )
    .expect("Error calling schedule_n_timers");

    // Up to 500 timers will be executed per round
    advance_seconds(&pic, timers_to_schedule / 500);

    let (executed_timers,): (u32,) = query_candid(&pic, canister_id, "executed_timers", ())
        .expect("Error querying executed_timers");

    assert_eq!(timers_to_schedule, executed_timers);
}

fn advance_seconds(pic: &PocketIc, seconds: u32) {
    for _ in 0..seconds {
        pic.advance_time(Duration::from_secs(1));
        pic.tick();
    }
}

#[test]
fn test_set_global_timers() {
    // Must be more than the queue limit (500)
    let pic = PocketIc::new();
    let system_time = std::time::SystemTime::now();

    pic.set_time(system_time);

    let wasm = cargo_build_canister("timers");
    let canister_id = pic.create_canister();
    pic.add_cycles(canister_id, INIT_CYCLES);
    pic.install_canister(canister_id, wasm, vec![], None);

    call_candid::<_, ()>(&pic, canister_id, "schedule_long", ())
        .expect("Failed to call schedule_long");
    let ts0 = system_time
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos() as u64
        + 9_000_000_000; // the long event is scheduled 9 seconds from ts0
    advance_seconds(&pic, 5);

    // set the timer to 5 seconds from ts0
    let ts1 = ts0 + 5_000_000_000;
    let (previous,) = call_candid::<(u64,), (u64,)>(&pic, canister_id, "set_global_timer", (ts1,))
        .expect("Failed to call set_global_timer");
    assert_eq!(previous, ts0);

    // deactivate the timer
    let (previous,) = call_candid::<(u64,), (u64,)>(&pic, canister_id, "set_global_timer", (0,))
        .expect("Failed to call set_global_timer");
    assert_eq!(previous, ts1);
}

#[test]
fn test_canister_info() {
    let pic = PocketIc::new();
    let wasm = cargo_build_canister("canister_info");
    let canister_id = pic.create_canister();
    pic.add_cycles(canister_id, INIT_CYCLES);
    pic.add_cycles(canister_id, 1_000_000_000_000);
    pic.install_canister(canister_id, wasm, vec![], None);

    let new_canister: (Principal,) = call_candid(&pic, canister_id, "canister_lifecycle", ())
        .expect("Error calling canister_lifecycle");

    let () = call_candid_as(
        &pic,
        Principal::management_canister(),
        RawEffectivePrincipal::None,
        Principal::anonymous(),
        "uninstall_code",
        (CanisterIdRecord {
            canister_id: new_canister.0,
        },),
    )
    .expect("Error calling uninstall_code");
    let () = call_candid_as(
        &pic,
        Principal::management_canister(),
        RawEffectivePrincipal::None,
        Principal::anonymous(),
        "install_code",
        (InstallCodeArgument {
            mode: CanisterInstallMode::Install,
            arg: vec![],
            wasm_module: vec![0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00],
            canister_id: new_canister.0,
        },),
    )
    .expect("Error calling install_code");

    let info: (CanisterInfoResponse,) = call_candid(&pic, canister_id, "info", (new_canister.0,))
        .expect("Error calling canister_info");

    let timestamp_nanos = pic
        .get_time()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_nanos() as u64;
    assert_eq!(
        info.0,
        CanisterInfoResponse {
            total_num_changes: 9,
            recent_changes: vec![
                CanisterChange {
                    timestamp_nanos,
                    canister_version: 0,
                    origin: CanisterChangeOrigin::FromCanister(FromCanisterRecord {
                        canister_id,
                        canister_version: Some(1)
                    }),
                    details: CanisterChangeDetails::Creation(CreationRecord {
                        controllers: vec![canister_id]
                    }),
                },
                CanisterChange {
                    timestamp_nanos,
                    canister_version: 1,
                    origin: CanisterChangeOrigin::FromCanister(FromCanisterRecord {
                        canister_id,
                        canister_version: Some(2)
                    }),
                    details: CanisterChangeDetails::CodeDeployment(CodeDeploymentRecord {
                        mode: Install,
                        module_hash: hex::decode(
                            "93a44bbb96c751218e4c00d479e4c14358122a389acca16205b1e4d0dc5f9476"
                        )
                        .unwrap(),
                    }),
                },
                CanisterChange {
                    timestamp_nanos,
                    canister_version: 2,
                    origin: CanisterChangeOrigin::FromCanister(FromCanisterRecord {
                        canister_id,
                        canister_version: Some(3)
                    }),
                    details: CanisterChangeDetails::CodeUninstall,
                },
                CanisterChange {
                    timestamp_nanos,
                    canister_version: 3,
                    origin: CanisterChangeOrigin::FromCanister(FromCanisterRecord {
                        canister_id,
                        canister_version: Some(4)
                    }),
                    details: CanisterChangeDetails::CodeDeployment(CodeDeploymentRecord {
                        mode: Install,
                        module_hash: hex::decode(
                            "93a44bbb96c751218e4c00d479e4c14358122a389acca16205b1e4d0dc5f9476"
                        )
                        .unwrap(),
                    }),
                },
                CanisterChange {
                    timestamp_nanos,
                    canister_version: 4,
                    origin: CanisterChangeOrigin::FromCanister(FromCanisterRecord {
                        canister_id,
                        canister_version: Some(5)
                    }),
                    details: CanisterChangeDetails::CodeDeployment(CodeDeploymentRecord {
                        mode: Reinstall,
                        module_hash: hex::decode(
                            "93a44bbb96c751218e4c00d479e4c14358122a389acca16205b1e4d0dc5f9476"
                        )
                        .unwrap(),
                    }),
                },
                CanisterChange {
                    timestamp_nanos,
                    canister_version: 5,
                    origin: CanisterChangeOrigin::FromCanister(FromCanisterRecord {
                        canister_id,
                        canister_version: Some(6)
                    }),
                    details: CanisterChangeDetails::CodeDeployment(CodeDeploymentRecord {
                        mode: Upgrade,
                        module_hash: hex::decode(
                            "93a44bbb96c751218e4c00d479e4c14358122a389acca16205b1e4d0dc5f9476"
                        )
                        .unwrap(),
                    }),
                },
                CanisterChange {
                    timestamp_nanos,
                    canister_version: 6,
                    origin: CanisterChangeOrigin::FromCanister(FromCanisterRecord {
                        canister_id,
                        canister_version: Some(7)
                    }),
                    details: CanisterChangeDetails::ControllersChange(ControllersChangeRecord {
                        controllers: vec![Principal::anonymous(), canister_id, new_canister.0]
                    }),
                },
                CanisterChange {
                    timestamp_nanos,
                    canister_version: 7,
                    origin: CanisterChangeOrigin::FromUser(FromUserRecord {
                        user_id: Principal::anonymous(),
                    }),
                    details: CanisterChangeDetails::CodeUninstall,
                },
                CanisterChange {
                    timestamp_nanos,
                    canister_version: 8,
                    origin: CanisterChangeOrigin::FromUser(FromUserRecord {
                        user_id: Principal::anonymous(),
                    }),
                    details: CanisterChangeDetails::CodeDeployment(CodeDeploymentRecord {
                        mode: Install,
                        module_hash: hex::decode(
                            "93a44bbb96c751218e4c00d479e4c14358122a389acca16205b1e4d0dc5f9476"
                        )
                        .unwrap(),
                    }),
                },
            ],
            module_hash: Some(
                hex::decode("93a44bbb96c751218e4c00d479e4c14358122a389acca16205b1e4d0dc5f9476")
                    .unwrap()
            ),
            controllers: vec![Principal::anonymous(), canister_id, new_canister.0],
        }
    );
}

#[test]
fn test_cycles_burn() {
    let pic = PocketIc::new();
    let wasm = cargo_build_canister("api-call");
    let canister_id = pic.create_canister();
    pic.add_cycles(canister_id, INIT_CYCLES);
    pic.add_cycles(canister_id, 1500);

    pic.install_canister(canister_id, wasm, vec![], None);
    eprintln!("Canister installed.");
    let balance1 = pic.cycle_balance(canister_id);
    eprintln!("Balance 1: {balance1}");

    let attempted = 1000u128;

    // Scenario 1: burn less than balance
    let (burned,): (u128,) = call_candid(&pic, canister_id, "cycles_burn", (attempted,))
        .expect("Error calling cycles_burn");
    eprintln!("Attempted to burn {attempted}, actually burned {burned}");
    assert_eq!(burned, attempted);
    let balance2 = pic.cycle_balance(canister_id);
    eprintln!("Balance 2: {balance2}");

    // Scenario 2: burn more than balance
    let (burned,): (u128,) = call_candid(&pic, canister_id, "cycles_burn", (attempted,))
        .expect("Error calling cycles_burn");
    eprintln!("Attempted to burn {attempted}, actually burned {burned}");
    assert!(burned < attempted);
    assert_eq!(burned, balance2);
    let balance3 = pic.cycle_balance(canister_id);
    eprintln!("Balance 3: {balance3}");
    assert_eq!(balance3, 0);
}

#[test]
fn call_management() {
    let pic = PocketIc::new();
    let wasm = cargo_build_canister("management_caller");
    let canister_id = pic.create_canister();
    pic.add_cycles(canister_id, INIT_CYCLES);
    pic.add_cycles(canister_id, 100_000_000_000_000);
    pic.install_canister(canister_id, wasm, vec![], None);
    let () = call_candid(&pic, canister_id, "execute_main_methods", ())
        .expect("Error calling execute_main_methods");
    let () = call_candid(&pic, canister_id, "execute_provisional_methods", ())
        .expect("Error calling execute_provisional_methods");
}

#[test]
fn test_chunk() {
    let pic = PocketIc::new();
    let wasm = cargo_build_canister("chunk");
    let canister_id = pic.create_canister();
    pic.add_cycles(canister_id, INIT_CYCLES);
    pic.add_cycles(canister_id, 100_000_000_000_000);
    pic.install_canister(canister_id, wasm, vec![], None);
    let (target_canister_id,): (Principal,) =
        call_candid(&pic, canister_id, "call_create_canister", ())
            .expect("Error calling call_create_canister");

    let wasm_module = b"\x00asm\x01\x00\x00\x00".to_vec();
    let wasm_module_hash = sha2::Sha256::digest(&wasm_module).to_vec();
    let chunk1 = wasm_module[..4].to_vec();
    let chunk2 = wasm_module[4..].to_vec();
    let hash1_expected = sha2::Sha256::digest(&chunk1).to_vec();
    let hash2_expected = sha2::Sha256::digest(&chunk2).to_vec();

    let (hash1_return,): (Vec<u8>,) = call_candid(
        &pic,
        canister_id,
        "call_upload_chunk",
        (target_canister_id, chunk1.clone()),
    )
    .expect("Error calling call_upload_chunk");
    assert_eq!(&hash1_return, &hash1_expected);

    let () = call_candid(
        &pic,
        canister_id,
        "call_clear_chunk_store",
        (target_canister_id,),
    )
    .expect("Error calling call_clear_chunk_store");

    let (_hash1_return,): (Vec<u8>,) = call_candid(
        &pic,
        canister_id,
        "call_upload_chunk",
        (target_canister_id, chunk1),
    )
    .expect("Error calling call_upload_chunk");
    let (_hash2_return,): (Vec<u8>,) = call_candid(
        &pic,
        canister_id,
        "call_upload_chunk",
        (target_canister_id, chunk2),
    )
    .expect("Error calling call_upload_chunk");

    let (hashes,): (Vec<Vec<u8>>,) = call_candid(
        &pic,
        canister_id,
        "call_stored_chunks",
        (target_canister_id,),
    )
    .expect("Error calling call_stored_chunks");
    // the hashes returned are not guaranteed to be in order
    assert_eq!(hashes.len(), 2);
    assert!(hashes.contains(&hash1_expected));
    assert!(hashes.contains(&hash2_expected));

    let () = call_candid(
        &pic,
        canister_id,
        "call_install_chunked_code",
        (
            target_canister_id,
            // the order of the hashes matters
            vec![hash1_expected, hash2_expected],
            wasm_module_hash,
        ),
    )
    .expect("Error calling call_install_chunked_code");
}
