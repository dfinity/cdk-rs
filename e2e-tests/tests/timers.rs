use pocket_ic::{query_candid, PocketIc};
use std::time::Duration;

mod test_utilities;
use test_utilities::{cargo_build_canister, pic_base, update};

#[test]
fn test_timers() {
    let wasm = cargo_build_canister("timers");
    let pic = pic_base().build();
    let canister_id = pic.create_canister();
    pic.add_cycles(canister_id, 2_000_000_000_000);
    pic.install_canister(canister_id, wasm, vec![], None);

    update::<(), ()>(&pic, canister_id, "schedule", ()).expect("Failed to call schedule");
    advance_seconds(&pic, 5);

    update::<_, ()>(&pic, canister_id, "schedule_long", ()).expect("Failed to call schedule_long");
    advance_seconds(&pic, 5);
    update::<_, ()>(&pic, canister_id, "cancel_long", ()).expect("Failed to call cancel_long");
    advance_seconds(&pic, 5);

    update::<_, ()>(&pic, canister_id, "start_repeating", ())
        .expect("Failed to call start_repeating");
    advance_seconds(&pic, 3);
    update::<_, ()>(&pic, canister_id, "stop_repeating", ())
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
    let pic = pic_base().build();
    let wasm = cargo_build_canister("timers");
    let canister_id = pic.create_canister();
    pic.add_cycles(canister_id, 2_000_000_000_000);
    pic.install_canister(canister_id, wasm, vec![], None);

    update::<_, ()>(&pic, canister_id, "set_self_cancelling_timer", ())
        .expect("Failed to call set_self_cancelling_timer");
    update::<_, ()>(&pic, canister_id, "set_self_cancelling_periodic_timer", ())
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
    let wasm = cargo_build_canister("timers");
    // Must be more than the queue limit (500)
    let timers_to_schedule = 1_000;
    let pic = pic_base().build();
    let canister_id = pic.create_canister();
    pic.add_cycles(canister_id, 100_000_000_000_000u128);
    pic.install_canister(canister_id, wasm, vec![], None);

    let () = update(
        &pic,
        canister_id,
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
    let wasm = cargo_build_canister("timers");
    let pic = pic_base().build();

    let canister_id = pic.create_canister();
    pic.add_cycles(canister_id, 2_000_000_000_000);
    pic.install_canister(canister_id, wasm, vec![], None);

    // Set a 9s timer at t0, it expires at t1 = t0 + 9s
    let t0 = pic.get_time().as_nanos_since_unix_epoch();
    let t1 = t0 + 9_000_000_000;
    update::<_, ()>(&pic, canister_id, "schedule_long", ()).expect("Failed to call schedule_long");

    // 5 seconds later, the 9s timer is still active
    advance_seconds(&pic, 5);

    // Set the expiration time of the timer to t2 = t1 + 5s
    let t2 = t1 + 5_000_000_000;
    let (previous,) =
        update::<(u64,), (u64,)>(&pic, canister_id, "global_timer_set", (t2,)).unwrap();
    assert!(previous.abs_diff(t1) < 2); // time error no more than 1 nanosecond

    // Deactivate the timer
    let (previous,) =
        update::<(u64,), (u64,)>(&pic, canister_id, "global_timer_set", (0,)).unwrap();
    assert!(previous.abs_diff(t2) < 2); // time error no more than 1 nanosecond
}
