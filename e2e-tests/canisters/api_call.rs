use ic_cdk::{api::call::ManualReply, query};

#[query]
fn instruction_counter() -> u64 {
    ic_cdk::api::instruction_counter()
}

#[query(manual_reply = true)]
fn manual_reject() -> ManualReply<u64> {
    ManualReply::reject("manual reject")
}

fn main() {}
