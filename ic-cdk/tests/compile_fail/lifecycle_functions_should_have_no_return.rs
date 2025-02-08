use ic_cdk::{heartbeat, init, inspect_message, post_upgrade, pre_upgrade};

#[init]
fn init() -> u32 {}

#[pre_upgrade]
fn pre_upgrade() -> u32 {}

#[post_upgrade]
fn post_upgrade() -> u32 {}

#[heartbeat]
fn heartbeat() -> u32 {}

#[inspect_message]
fn inspect_message() -> u32 {}

fn main() {}
