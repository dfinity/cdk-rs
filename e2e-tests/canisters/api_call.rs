use ic_cdk::api::call::{call_raw, msg_cycles_refunded, CallFutureRaw, ManualReply};
use ic_cdk::export::Principal;
use ic_cdk_macros::{query, update};

#[query]
fn instruction_counter() -> u64 {
    ic_cdk::api::instruction_counter()
}

#[query(manual_reply = true)]
fn manual_reject() -> ManualReply<u64> {
    ManualReply::reject("manual reject")
}

#[update]
async fn call_future_raw() -> u32 {
    let call_future: CallFutureRaw = call_raw(
        Principal::management_canister(),
        "raw_rand",
        &[68, 73, 68, 76, 0, 0],
        0,
    );
    let call_perform_status_code: u32 = call_future.call_perform_status_code;
    if call_future.will_perform() {
        let _call_result = call_future.await;
        // this code is within a callback.
        msg_cycles_refunded();
    }
    call_perform_status_code
}

fn main() {}
