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
pub mod bitcoin_canister;
pub mod call;
pub mod futures;
mod macros;
pub mod management_canister;
mod printer;
pub mod stable;
pub mod storage;

use std::{
    future::Future,
    sync::{Arc, Once},
    task::{Context, Poll, Wake, Waker},
};

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

#[doc(hidden)]
#[deprecated(
    since = "0.18.0",
    note = "Use ic_cdk::futures::spawn. Compatibility notice: Code execution order will change, \
        see https://github.com/dfinity/cdk-rs/blob/0.18.3/ic-cdk/V18_GUIDE.md#futures-ordering-changes"
)]
pub fn spawn<F: 'static + Future<Output = ()>>(fut: F) {
    struct DummyWaker;
    impl Wake for DummyWaker {
        fn wake(self: Arc<Self>) {
            panic!("Your code is incompatible with the ic_cdk::spawn compatibility adapter. Migrate to ic_cdk::futures::spawn. \
                Notice: Code execution order will change, see https://github.com/dfinity/cdk-rs/blob/0.18.3/ic-cdk/V18_GUIDE.md#futures-ordering-changes")
        }
    }
    // Emulated behavior: A spawned future is polled once immediately, then backgrounded and run at a normal pace.
    // We poll it once with an unimplemented waker, then spawn it, which will poll it again with the real waker.
    // In a correctly implemented future, this second poll should overwrite the fake waker with the real one.
    // The only way to hit the fake waker's wake function is if the first poll calls wake.
    // A more complex compat adapter will be needed to handle this case.
    let mut pin = Box::pin(fut);
    let poll = pin
        .as_mut()
        .poll(&mut Context::from_waker(&Waker::from(Arc::new(DummyWaker))));
    match poll {
        Poll::Ready(()) => {}
        Poll::Pending => crate::futures::spawn(pin),
    }
}
