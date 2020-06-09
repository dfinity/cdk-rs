use ic_cdk_macros::*;

static mut COUNTER: Option<candid::Nat> = None;

#[init]
fn init() {
    ic_cdk::print("HELLO");
    unsafe {
        COUNTER = Some(candid::Nat::from(0));
    }
}

#[update]
fn inc() -> () {
    unsafe {
        COUNTER.as_mut().unwrap().0 += 1u64;
    }
}

#[query]
fn read() -> candid::Nat {
    unsafe { COUNTER.as_mut().unwrap().clone() }
}

#[update]
fn write(input: candid::Nat) -> () {
    unsafe {
        COUNTER.as_mut().unwrap().0 = input.0;
    }
}
