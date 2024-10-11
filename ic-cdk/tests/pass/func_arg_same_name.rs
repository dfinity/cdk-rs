use ic_cdk::{query, update};

#[update]
fn foo(foo: i32) {}

#[query]
fn bar(bar: i32) {}

fn main() {}
