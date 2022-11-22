use ic_cdk::{heartbeat, init, inspect_message, post_upgrade, pre_upgrade, query, update};

#[init]
fn init() {}

#[pre_upgrade]
fn pre_upgrade() {}

#[post_upgrade]
fn post_upgrade() {}

#[update]
fn update() {}

#[query]
fn query() {}

#[query(composite = true)]
fn composite_query() {}

#[heartbeat]
fn heartbeat() {}

#[inspect_message]
fn inspect_message() {}

fn main() {}
