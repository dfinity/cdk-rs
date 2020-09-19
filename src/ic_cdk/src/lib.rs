// Use `wee_alloc` as the global allocator.
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

mod api;
mod futures;
mod ic0;
mod printer;
pub mod storage;

#[cfg(feature = "experimental")]
mod ic1;

pub use api::*;

static mut DONE: bool = false;

pub mod export {
    pub use ic_types::Principal;
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

/// Block on a promise in a WASM-friendly way (no multithreading!).
pub fn block_on<F: 'static + std::future::Future<Output = ()>>(future: F) {
    futures::block_on(future);
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
