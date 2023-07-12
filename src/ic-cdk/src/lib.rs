#![warn(
    elided_lifetimes_in_paths,
    missing_debug_implementations,
    missing_docs,
    unsafe_op_in_unsafe_fn,
    clippy::undocumented_unsafe_blocks,
    clippy::missing_safety_doc
)]
#![cfg_attr(docsrs, feature(doc_cfg))]

//! This crate provides building blocks for developing Internet Computer canisters.
//!
//! You can check the [Internet Computer Specification](
//! https://smartcontracts.org/docs/interface-spec/index.html#system-api-imports)
//! for a full list of the system API functions.

#[cfg(target_feature = "atomics")]
compile_error!("This version of the CDK does not support multithreading.");

#[doc(inline)]
pub use ic_cdk_macros::*;

pub mod api;
mod futures;
mod printer;
pub mod storage;

use std::sync::atomic::{AtomicBool, Ordering};

pub use api::call::call;
pub use api::call::notify;
pub use api::{caller, id, print, trap};

static DONE: AtomicBool = AtomicBool::new(false);

/// Re-exports crates those are necessary for using ic-cdk
// pub mod export {
//     pub use candid;
//     pub use candid::Principal;
//     pub use serde;
// }

/// Setup the stdlib hooks.
pub fn setup() {
    if !DONE.swap(true, Ordering::SeqCst) {
        printer::hook()
    }
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
    ($fmt:expr) => ($crate::print(format!($fmt)));
    ($fmt:expr, $($arg:tt)*) => ($crate::print(format!($fmt, $($arg)*)));
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
    ($fmt:expr) => ($crate::print(format!($fmt)));
    ($fmt:expr, $($arg:tt)*) => ($crate::print(format!($fmt, $($arg)*)));
}

/// Format and then print the formatted message
#[cfg(not(target_arch = "wasm32"))]
#[macro_export]
macro_rules! eprintln {
    ($fmt:expr) => (std::eprintln!($fmt));
    ($fmt:expr, $($arg:tt)*) => (std::eprintln!($fmt, $($arg)*));
}
