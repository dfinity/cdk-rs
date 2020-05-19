use ic_cdk_macros::*;

static mut COUNTER: u32 = 0;

#[query]
fn read() -> u32 {
    unsafe {
        ic_cdk::print(format!("read {}", COUNTER));
        COUNTER
    }
}

#[update]
fn inc() -> () {
    unsafe {
        ic_cdk::print(format!("inc {} + 1", COUNTER));
        COUNTER = COUNTER + 1;
    }
}

#[update]
fn write(input: u32) -> () {
    unsafe {
        ic_cdk::print(format!("write {} := {}", COUNTER, input));
        COUNTER = input;
    }
}
