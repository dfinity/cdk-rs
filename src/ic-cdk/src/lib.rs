#![warn(missing_docs)]

//! This crate provides building blocks for developing Internet Computer Canister.
//!
//! You can check the [Internet Computer Specification](
//! https://smartcontracts.org/docs/interface-spec/index.html#system-api-imports)
//! for a full list of the system API functions.

pub mod api;
mod futures;
mod printer;
pub mod storage;

pub use api::call::call;
pub use api::{caller, id, print, trap};

static mut DONE: bool = false;

/// Re-exports crates those are necessary for using ic-cdk
pub mod export {
    pub use candid;
    pub use candid::types::ic_types::Principal;
    pub use serde;
}

/// Setup the stdlib hooks.
pub fn setup() {
    unsafe {
        if DONE {
            return;
        }
        DONE = true;
    }
    printer::hook()
}

/// See documentation for [spawn].
#[deprecated(
    since = "0.3.4",
    note = "Use the spawn() function instead, it does the same thing but is more appropriately named."
)]
pub fn block_on<F: 'static + std::future::Future<Output = ()>>(future: F) {
    futures::spawn(future);
}

/// Spawn an asynchronous task that drives the provided future to
/// completion.
pub fn spawn<F: 'static + std::future::Future<Output = ()>>(future: F) {
    futures::spawn(future);
}

/// Format and then print the formatted message
#[cfg(target_arch = "wasm32")]
#[macro_export]
macro_rules! println {
    ($fmt:expr) => (ic_cdk::print(format!($fmt)));
    ($fmt:expr, $($arg:tt)*) => (ic_cdk::print(format!($fmt, $($arg)*)));
}

/// Format and then print the formatted message
#[cfg(not(target_arch = "wasm32"))]
#[macro_export]
macro_rules! println {
    ($fmt:expr) => (std::println!($fmt));
    ($fmt:expr, $($arg:tt)*) => (std::println!($fmt, $($arg)*));
}

/// Format and then print the formatted message
#[cfg(target_arch = "wasm32")]
#[macro_export]
macro_rules! eprintln {
    ($fmt:expr) => (ic_cdk::print(format!($fmt)));
    ($fmt:expr, $($arg:tt)*) => (ic_cdk::print(format!($fmt, $($arg)*)));
}

/// Format and then print the formatted message
#[cfg(not(target_arch = "wasm32"))]
#[macro_export]
macro_rules! eprintln {
    ($fmt:expr) => (std::eprintln!($fmt));
    ($fmt:expr, $($arg:tt)*) => (std::eprintln!($fmt, $($arg)*));
}
