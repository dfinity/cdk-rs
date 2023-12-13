use candid::Principal;
use ic_cdk::{api::call::ManualReply, init, query, update};
use std::cell::{Cell, RefCell};

thread_local! {
    static COUNTER: RefCell<candid::Nat> = RefCell::new(candid::Nat::from(0u8));
    static OWNER: Cell<Principal> = Cell::new(Principal::from_slice(&[]));
}

#[init]
fn init() {
    OWNER.with(|owner| owner.set(ic_cdk::api::caller()));
}

#[update]
fn inc() {
    ic_cdk::println!("{:?}", OWNER.with(|owner| owner.get()));
    COUNTER.with(|counter| *counter.borrow_mut() += 1u64);
}

#[query(manual_reply = true)]
fn read() -> ManualReply<candid::Nat> {
    COUNTER.with(|counter| ManualReply::one(counter))
}

#[update]
fn write(input: candid::Nat) {
    COUNTER.with(|counter| *counter.borrow_mut() = input);
}

#[update(no_export = true)]
fn update_no_export() {}

#[query(no_export = true)]
fn query_no_export() {}

ic_cdk::export_candid!();
