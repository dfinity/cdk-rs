use ic_cdk::{
    query,
    timer::{clear_timer, set_timer, set_timer_interval, TimerId},
    update,
};
use std::{
    cell::{Cell, RefCell},
    time::Duration,
};

thread_local! {
    static EVENTS: RefCell<Vec<&'static str>> = RefCell::default();
    static LONG: Cell<TimerId> = Cell::default();
    static REPEATING: Cell<TimerId> = Cell::default();
}

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
fn schedule_long() {
    let id = set_timer(Duration::from_secs(9), || add_event("long"));
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
fn stop_repeating() {
    REPEATING.with(|repeating| clear_timer(repeating.get()));
}

fn add_event(event: &'static str) {
    EVENTS.with(|events| events.borrow_mut().push(event));
}

fn main() {}
