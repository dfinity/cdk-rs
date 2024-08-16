use ic_cdk::{heartbeat, init, inspect_message, post_upgrade, pre_upgrade};

fn guard_function() -> Result<(), String> {
    unimplemented!()
}

#[init(guard = "guard_function")]
fn init() {}

#[pre_upgrade(guard = "guard_function")]
fn pre_upgrade() {}

#[post_upgrade(guard = "guard_function")]
fn post_upgrade() {}

#[heartbeat(guard = "guard_function")]
fn heartbeat() {}

#[inspect_message(guard = "guard_function")]
fn inspect_message() {}

fn main() {}
