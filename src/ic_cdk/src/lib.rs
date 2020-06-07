// Use `wee_alloc` as the global allocator.
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

mod api;
mod futures;
mod ic0;
mod printer;
pub mod storage;

pub use api::*;

use std::sync::Once;
static START: Once = Once::new();

pub fn setup() -> () {
    START.call_once(|| printer::hook());
}

pub fn block_on<F: 'static + std::future::Future<Output = ()>>(future: F) -> () {
    futures::block_on(future);
}
