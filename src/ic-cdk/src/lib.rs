#![doc = include_str!("../README.md")]
#![warn(
    elided_lifetimes_in_paths,
    missing_debug_implementations,
    missing_docs,
    unsafe_op_in_unsafe_fn,
    clippy::undocumented_unsafe_blocks,
    clippy::missing_safety_doc
)]
#![cfg_attr(docsrs, feature(doc_cfg))]

#[cfg(target_feature = "atomics")]
compile_error!("This version of the CDK does not support multithreading.");

pub mod api;
mod macros;
mod printer;
pub mod storage;

use std::sync::atomic::{AtomicBool, Ordering};

#[doc(inline)]
pub use api::call::call;
#[doc(inline)]
pub use api::call::notify;
#[doc(inline)]
pub use api::{caller, id, print, trap};

#[doc(inline)]
pub use macros::*;

static DONE: AtomicBool = AtomicBool::new(false);

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
    ic_cdk_executor::spawn(future);
}

/// Spawn an asynchronous task that drives the provided future to
/// completion.
pub fn spawn<F: 'static + std::future::Future<Output = ()>>(future: F) {
    ic_cdk_executor::spawn(future);
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
