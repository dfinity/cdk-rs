use ic_cdk_macros::*;

static mut COUNTER: Option<candid::Nat> = None;
static mut OWNER: Option<candid::Principal> = None;

#[init]
fn init() {
    unsafe {
        OWNER = Some(candid::Principal(ic_cdk::reflection::caller()));
        COUNTER = Some(candid::Nat::from(0));
    }
}

#[update]
fn inc() -> () {
    unsafe {
        ic_cdk::println!("{:?}", OWNER);
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
