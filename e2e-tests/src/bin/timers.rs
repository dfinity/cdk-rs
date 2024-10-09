use ic_cdk::{query, update};
use ic_cdk_timers::{clear_timer, set_timer, set_timer_interval, TimerId};
use std::{
    cell::{Cell, RefCell},
    sync::atomic::{AtomicU32, Ordering},
    time::Duration,
};

thread_local! {
    static EVENTS: RefCell<Vec<&'static str>> = RefCell::default();
    static LONG: Cell<TimerId> = Cell::default();
    static REPEATING: Cell<TimerId> = Cell::default();
}

static EXECUTED_TIMERS: AtomicU32 = AtomicU32::new(0);

#[query]
fn get_events() -> Vec<&'static str> {
    EVENTS.with(|events| events.borrow().clone())
}

#[update]
fn clear_events() {
    EVENTS.with(|events| events.borrow_mut().clear());
}

#[update]
fn schedule() {
    set_timer(Duration::from_secs(2), || add_event("2"));
    set_timer(Duration::from_secs(1), || {
        add_event("1");
        set_timer(Duration::from_secs(2), || add_event("3"));
    });
    set_timer(Duration::from_secs(4), || add_event("4"));
}

#[update]
fn schedule_n_timers(n: u32) {
    for i in 0..n {
        ic_cdk_timers::set_timer(Duration::from_nanos(i.into()), move || {
            EXECUTED_TIMERS.fetch_add(1, Ordering::Relaxed);
        });
    }
}

#[query]
fn executed_timers() -> u32 {
    EXECUTED_TIMERS.load(Ordering::Relaxed)
}

#[update]
fn schedule_long() {
    let id = set_timer(Duration::from_secs(9), || add_event("long"));
    LONG.with(|long| long.set(id));
}

#[update]
fn set_self_cancelling_timer() {
    let id = set_timer(Duration::from_secs(0), || {
        cancel_long();
        add_event("timer cancelled self");
    });
    LONG.with(|long| long.set(id));
}

#[update]
fn cancel_long() {
    LONG.with(|long| clear_timer(long.get()));
}

#[update]
fn start_repeating() {
    let id = set_timer_interval(Duration::from_secs(1), || add_event("repeat"));
    REPEATING.with(|repeating| repeating.set(id));
}

#[update]
fn set_self_cancelling_periodic_timer() {
    let id = set_timer_interval(Duration::from_secs(0), || {
        stop_repeating();
        add_event("periodic timer cancelled self")
    });
    REPEATING.with(|repeating| repeating.set(id));
}

#[update]
fn stop_repeating() {
    REPEATING.with(|repeating| clear_timer(repeating.get()));
}

fn add_event(event: &'static str) {
    EVENTS.with(|events| events.borrow_mut().push(event));
}

#[update]
fn set_global_timer(timestamp: u64) -> u64 {
    ic_cdk::api::set_global_timer(timestamp)
}

fn main() {}
