use ic_cdk_macros::*;

#[init]
fn init_1() {}

#[init]
fn init_2() {}

#[pre_upgrade]
fn pre_upgrade_1() {}

#[pre_upgrade]
fn pre_upgrade_2() {}

#[post_upgrade]
fn post_upgrade_1() {}

#[post_upgrade]
fn post_upgrade_2() {}

#[heartbeat]
fn heartbeat_1() {}

#[heartbeat]
fn heartbeat_2() {}

#[inspect_message]
fn inspect_message_1() {}

#[inspect_message]
fn inspect_message_2() {}

fn main() {}
