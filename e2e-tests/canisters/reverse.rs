use ic_cdk::api::call::{arg_data_raw, reply_raw};

#[export_name = "canister_query reverse"]
fn reverse() {
    reply_raw(
        arg_data_raw()
            .into_iter()
            .rev()
            .collect::<Vec<_>>()
            .as_ref(),
    );
}
#[export_name = "canister_update empty_call"]
fn empty_call() {
    reply_raw(&[]);
}

fn main() {}
