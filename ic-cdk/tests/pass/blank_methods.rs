use ic_cdk::{
    heartbeat, init, inspect_message, on_low_wasm_memory, post_upgrade, pre_upgrade, query, update,
};

#[init]
fn init() {}

#[pre_upgrade]
fn pre_upgrade() {}

#[post_upgrade]
fn post_upgrade() {}

#[update]
fn update() {}

#[update(hidden = true)]
fn update_hidden() {}

#[query]
fn query() {}

#[query(hidden = true)]
fn query_hidden() {}

#[query(composite = true)]
fn composite_query() {}

#[heartbeat]
fn heartbeat() {}

#[inspect_message]
fn inspect_message() {}

#[on_low_wasm_memory]
fn on_low_wasm_memory() {}

fn main() {}
