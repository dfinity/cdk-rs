use ic_cdk::{api::call::ManualReply, query, update};

#[query]
fn instruction_counter() -> u64 {
    ic_cdk::api::instruction_counter()
}

#[query(manual_reply = true)]
fn manual_reject() -> ManualReply<u64> {
    ManualReply::reject("manual reject")
}

#[update]
fn cycles_burn(amount: u128) -> u128 {
    ic_cdk::api::cycles_burn(amount)
}

fn main() {}
