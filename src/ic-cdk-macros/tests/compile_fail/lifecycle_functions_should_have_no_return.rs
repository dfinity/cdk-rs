use ic_cdk_macros::*;

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
