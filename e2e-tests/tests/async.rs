use ic_cdk_e2e_tests::cargo_build_canister;
use pocket_ic::common::rest::RawEffectivePrincipal;
use pocket_ic::{call_candid, query_candid, CallError, ErrorCode, PocketIc};

#[test]
fn panic_after_async_frees_resources() {
    let pic: PocketIc = PocketIc::new();
    let wasm = cargo_build_canister("async");
    let canister_id = pic.create_canister();
    pic.add_cycles(canister_id, 2_000_000_000_000);
    pic.install_canister(canister_id, wasm, vec![], None);

    for i in 1..3 {
        match call_candid(
            &pic,
            canister_id,
            RawEffectivePrincipal::None,
            "panic_after_async",
            (),
        ) {
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

        let (n,): (u64,) = call_candid(
            &pic,
            canister_id,
            RawEffectivePrincipal::None,
            "invocation_count",
            (),
        )
        .expect("failed to call invocation_count");

        assert_eq!(i, n, "expected the invocation count to be {}, got {}", i, n);
    }

    let (message,): (String,) = call_candid(
        &pic,
        canister_id,
        RawEffectivePrincipal::None,
        "invalid_reply_payload_does_not_trap",
        (),
    )
    .expect("call failed");
    assert_eq!(&message, "handled decoding error gracefully with code 5");

    let err = call_candid::<_, ()>(
        &pic,
        canister_id,
        RawEffectivePrincipal::None,
        "panic_twice",
        (),
    )
    .expect_err("failed to panic");
    assert!(
        matches!(err, CallError::UserError(u) if u.description.contains("Call already trapped"))
    );
    let _: (u64,) = call_candid(
        &pic,
        canister_id,
        RawEffectivePrincipal::None,
        "notifications_received",
        (),
    )
    .expect("failed to call unrelated function afterwards");
    let _: (u64,) = call_candid(
        &pic,
        canister_id,
        RawEffectivePrincipal::None,
        "invocation_count",
        (),
    )
    .expect("failed to recover lock");
}

#[test]
fn notify_calls() {
    let pic = PocketIc::new();
    let wasm = cargo_build_canister("async");
    let sender_id = pic.create_canister();
    pic.add_cycles(sender_id, 2_000_000_000_000);
    pic.install_canister(sender_id, wasm.clone(), vec![], None);
    let receiver_id = pic.create_canister();
    pic.add_cycles(receiver_id, 2_000_000_000_000);
    pic.install_canister(receiver_id, wasm, vec![], None);

    let (n,): (u64,) = query_candid(&pic, receiver_id, "notifications_received", ())
        .expect("failed to query 'notifications_received'");
    assert_eq!(n, 0);

    let () = call_candid(
        &pic,
        sender_id,
        RawEffectivePrincipal::None,
        "notify",
        (receiver_id, "on_notify"),
    )
    .expect("failed to call 'notify'");

    let (n,): (u64,) = query_candid(&pic, receiver_id, "notifications_received", ())
        .expect("failed to query 'notifications_received'");
    assert_eq!(n, 1);
}

// Composite queries are not enabled yet.
#[test]
fn test_composite_query() {
    let pic = PocketIc::new();
    let wasm = cargo_build_canister("async");
    let sender_id = pic.create_canister();
    pic.add_cycles(sender_id, 2_000_000_000_000);
    pic.install_canister(sender_id, wasm.clone(), vec![], None);
    let receiver_id = pic.create_canister();
    pic.add_cycles(receiver_id, 2_000_000_000_000);
    pic.install_canister(receiver_id, wasm, vec![], None);

    let (greeting,): (String,) = query_candid(&pic, sender_id, "greet_self", (receiver_id,))
        .expect("failed to query 'greet_self'");
    assert_eq!(greeting, "Hello, myself");
}
