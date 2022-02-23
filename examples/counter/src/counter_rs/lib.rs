use ic_cdk::{
    api::call::ManualReply,
    export::{candid, Principal},
};
use ic_cdk_macros::*;
use std::cell::{Cell, RefCell};

thread_local! {
    static COUNTER: RefCell<candid::Nat> = RefCell::new(candid::Nat::from(0));
    static OWNER: Cell<Principal> = Cell::new(Principal::from_slice(&[]));
}

#[init]
fn init() {
    OWNER.with(|owner| owner.set(ic_cdk::api::caller()));
}

#[update]
fn inc() -> () {
    ic_cdk::println!("{:?}", OWNER.with(|owner| owner.get()));
    COUNTER.with(|counter| *counter.borrow_mut() += 1u64);
}

#[query(manual_reply = true)]
fn read() -> ManualReply<candid::Nat> {
    COUNTER.with(|counter| ManualReply::one(counter))
}

#[update]
fn write(input: candid::Nat) -> () {
    COUNTER.with(|counter| *counter.borrow_mut() = input);
}
