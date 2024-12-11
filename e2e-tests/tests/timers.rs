use ic_cdk_e2e_tests::cargo_build_canister;
use pocket_ic::common::rest::RawEffectivePrincipal;
use pocket_ic::{call_candid, query_candid, PocketIc};
use std::time::Duration;
use std::time::SystemTime;

#[test]
fn test_timers() {
    let pic = PocketIc::new();
    let wasm = cargo_build_canister("timers");
    let canister_id = pic.create_canister();
    pic.add_cycles(canister_id, 2_000_000_000_000);
    pic.install_canister(canister_id, wasm, vec![], None);

    call_candid::<(), ()>(
        &pic,
        canister_id,
        RawEffectivePrincipal::None,
        "schedule",
        (),
    )
    .expect("Failed to call schedule");
    advance_seconds(&pic, 5);

    call_candid::<_, ()>(
        &pic,
        canister_id,
        RawEffectivePrincipal::None,
        "schedule_long",
        (),
    )
    .expect("Failed to call schedule_long");
    advance_seconds(&pic, 5);
    call_candid::<_, ()>(
        &pic,
        canister_id,
        RawEffectivePrincipal::None,
        "cancel_long",
        (),
    )
    .expect("Failed to call cancel_long");
    advance_seconds(&pic, 5);

    call_candid::<_, ()>(
        &pic,
        canister_id,
        RawEffectivePrincipal::None,
        "start_repeating",
        (),
    )
    .expect("Failed to call start_repeating");
    advance_seconds(&pic, 3);
    call_candid::<_, ()>(
        &pic,
        canister_id,
        RawEffectivePrincipal::None,
        "stop_repeating",
        (),
    )
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
    pic.add_cycles(canister_id, 2_000_000_000_000);
    pic.install_canister(canister_id, wasm, vec![], None);

    call_candid::<_, ()>(
        &pic,
        canister_id,
        RawEffectivePrincipal::None,
        "set_self_cancelling_timer",
        (),
    )
    .expect("Failed to call set_self_cancelling_timer");
    call_candid::<_, ()>(
        &pic,
        canister_id,
        RawEffectivePrincipal::None,
        "set_self_cancelling_periodic_timer",
        (),
    )
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
    pic.add_cycles(canister_id, 100_000_000_000_000u128);
    pic.install_canister(canister_id, wasm, vec![], None);

    let () = call_candid(
        &pic,
        canister_id,
        RawEffectivePrincipal::None,
        "schedule_n_timers",
        (timers_to_schedule,),
    )
    .expect("Error calling schedule_n_timers");

    // Up to 20 timers will be executed per round
    // Be conservative that advance 2 times the minimum number of rounds
    const TIMERS_PER_ROUND: u32 = 20;
    advance_seconds(&pic, 2 * timers_to_schedule / TIMERS_PER_ROUND);

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
    let pic = PocketIc::new();

    let wasm = cargo_build_canister("timers");
    let canister_id = pic.create_canister();
    pic.add_cycles(canister_id, 2_000_000_000_000);
    pic.install_canister(canister_id, wasm, vec![], None);

    // Set a 9s timer at t0, it expires at t1 = t0 + 9s
    let t0 = pic
        .get_time()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_nanos() as u64;
    let t1 = t0 + 9_000_000_000;
    call_candid::<_, ()>(
        &pic,
        canister_id,
        RawEffectivePrincipal::None,
        "schedule_long",
        (),
    )
    .expect("Failed to call schedule_long");

    // 5 seconds later, the 9s timer is still active
    advance_seconds(&pic, 5);

    // Set the expiration time of the timer to t2 = t1 + 5s
    let t2 = t1 + 5_000_000_000;
    let (previous,) = call_candid::<(u64,), (u64,)>(
        &pic,
        canister_id,
        RawEffectivePrincipal::None,
        "set_global_timer",
        (t2,),
    )
    .expect("Failed to call set_global_timer");
    assert!(previous.abs_diff(t1) < 2); // time error no more than 1 nanosecond

    // Deactivate the timer
    let (previous,) = call_candid::<(u64,), (u64,)>(
        &pic,
        canister_id,
        RawEffectivePrincipal::None,
        "set_global_timer",
        (0,),
    )
    .expect("Failed to call set_global_timer");
    assert!(previous.abs_diff(t2) < 2); // time error no more than 1 nanosecond
}
