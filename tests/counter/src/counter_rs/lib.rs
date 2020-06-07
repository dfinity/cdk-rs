use ic_cdk_macros::*;

static mut COUNTER: u32 = 0;

#[update]
fn inc() -> () {
    unsafe {
        COUNTER = COUNTER + 1;
    }
}

#[query]
fn read() -> u32 {
    unsafe { COUNTER }
}

#[update]
fn write(input: u32) -> () {
    unsafe {
        COUNTER = input;
    }
}
