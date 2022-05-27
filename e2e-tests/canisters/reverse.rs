use ic_cdk::api::call::{arg_data_raw, arg_data_raw_size, reply_raw};

#[export_name = "canister_query reverse"]
fn reverse() {
    let arg_bytes: Vec<u8> = arg_data_raw();
    assert_eq!(arg_bytes.len(), arg_data_raw_size());
    reply_raw(arg_bytes.into_iter().rev().collect::<Vec<_>>().as_ref());
}
#[export_name = "canister_update empty_call"]
fn empty_call() {
    reply_raw(&[]);
}

fn main() {}
