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
pub mod call;
pub mod futures;
mod macros;
pub mod management_canister;
mod printer;
pub mod stable;
pub mod storage;

use std::sync::Once;

#[doc(inline)]
pub use api::trap;

#[doc(inline)]
#[allow(deprecated)]
pub use api::{
    call::{call, notify},
    caller, id, print,
};

#[doc(inline)]
pub use macros::*;

static SETUP: Once = Once::new();

/// Setup the stdlib hooks.
fn setup() {
    SETUP.call_once(printer::hook);
}

/// Format and then print the formatted message
#[cfg(target_family = "wasm")]
#[macro_export]
macro_rules! println {
    ($fmt:expr) => ($crate::api::debug_print(format!($fmt)));
    ($fmt:expr, $($arg:tt)*) => ($crate::api::debug_print(format!($fmt, $($arg)*)));
}

/// Format and then print the formatted message
#[cfg(not(target_family = "wasm"))]
#[macro_export]
macro_rules! println {
    ($fmt:expr) => (std::println!($fmt));
    ($fmt:expr, $($arg:tt)*) => (std::println!($fmt, $($arg)*));
}

/// Format and then print the formatted message
#[cfg(target_family = "wasm")]
#[macro_export]
macro_rules! eprintln {
    ($fmt:expr) => ($crate::api::debug_print(format!($fmt)));
    ($fmt:expr, $($arg:tt)*) => ($crate::api::debug_print(format!($fmt, $($arg)*)));
}

/// Format and then print the formatted message
#[cfg(not(target_family = "wasm"))]
#[macro_export]
macro_rules! eprintln {
    ($fmt:expr) => (std::eprintln!($fmt));
    ($fmt:expr, $($arg:tt)*) => (std::eprintln!($fmt, $($arg)*));
}
