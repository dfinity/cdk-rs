//! Functions relating to the async executor.
//!
//! ## Running async tasks
//!
//! Most async tasks can be run just by changing your canister entry point to `async`:
//!
//! ```
//! # use ic_cdk::update;
//! # async fn some_other_async_fn() {}
//! #[update]
//! async fn foo() {
//!     some_other_async_fn().await;
//! }
//! ```
//!
//! To run async tasks in the *background*, however, use [`spawn`]:
//!
//! ```
//! # use ic_cdk::{update, futures::spawn};
//! # async fn some_other_async_fn() {}
//! #[update]
//! async fn foo() {
//!     spawn(async { some_other_async_fn().await; });
//!     // do other stuff
//! }
//! ```
//!
//! The spawned future will not be run at the same time as the remaining code, nor will it run immediately. It will start
//! running while `foo` awaits (or after it ends if it does not await). Unlike some other libraries, `spawn` does not
//! return a join-handle; if you want to await multiple results concurrently, use `futures`' [`join_all`] function.
//!
//! "Background" is a tricky subject on the IC. Background tasks can only run in the context of a canister message.
//! If you await a future whose completion you manually trigger in code, such as sending to an async channel,
//! then the code after the await will be in the call context of whatever you completed it in. This means that global state
//! like [`caller`], [`in_replicated_execution`], and even [`canister_self`] may have changed. (The canister method
//! itself cannot await anything triggered by another canister method, or you will get an error that it 'failed to reply'.)
//! It will also take from that call's instruction limit, which can introduce hidden sources of instruction limit based traps.
//!
//! Most importantly, a background task that runs in other call contexts must never trap. When it traps, it will cancel
//! (see below) the execution of the call whose context it's in, even though that call didn't do anything wrong, and it
//! may not undo whatever caused it to trap, meaning the canister could end up bricked. Tasks that you expect to complete
//! before the canister method ends are safe, but traps/panics in tasks that are expected to continue running into other
//! calls/timers may produce surprising results and behavioral errors.
//!
//! ## Automatic cancellation
//!
//! Asynchronous tasks can be *canceled*, meaning that a partially completed function will halt at an
//! `await` point, never complete, and drop its local variables as though it had returned. Cancellation
//! is caused by panics and traps: if an async function panics, time will be rewound to the
//! previous await as though the code since then never ran, and then the task will be canceled.
//!
//! Use panics sparingly in async functions after the first await, and beware system functions that trap
//! (which is most of them in the right context). Make atomic transactions between awaits wherever
//! possible, and use [`scopeguard`] or a [`Drop`] impl for any cleanup functions that must run no matter what.
//! If an await cannot be removed from the middle of a transaction, and it must be rolled back if it fails,
//! [`is_recovering_from_trap`] can be used to detect when the task is being automatically canceled.
//!
//! [`scopeguard`]: https://docs.rs/scopeguard
//! [`join_all`]: https://docs.rs/futures/latest/futures/future/fn.join_all.html
//! [`caller`]: crate::api::caller
//! [`in_replicated_execution`]: crate::api::in_replicated_execution
//! [`canister_self`]: crate::api::canister_self

use std::{
    future::Future,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    task::{Context, Poll, Wake, Waker},
};

/// Spawn an asynchronous task to run in the background. For information about semantics, see
/// [the module docs](self).
pub fn spawn<F: 'static + Future<Output = ()>>(future: F) {
    ic_cdk_executor::spawn(future);
}

/// Execute an update function in a context that allows calling [`spawn`].
///
/// You do not need to worry about this function unless you are avoiding the attribute macros.
///
/// Background tasks will be polled in the process (and will not be run otherwise).
/// Panics if called inside an existing executor context.
pub fn in_executor_context<R>(f: impl FnOnce() -> R) -> R {
    ic_cdk_executor::in_executor_context(f)
}

/// Execute a composite query function in a context that allows calling [`spawn`].
///
/// You do not need to worry about this function unless you are avoiding the attribute macros.
///
/// Background composite query tasks will be polled in the process (and will not be run otherwise).
/// Panics if called inside an existing executor context.
pub fn in_query_executor_context<R>(f: impl FnOnce() -> R) -> R {
    ic_cdk_executor::in_query_executor_context(f)
}

/// Tells you whether the current async fn is being canceled due to a trap/panic.
///
/// In a destructor, `is_recovering_from_trap` serves the same purpose as
/// [`std::thread::panicking`] - it tells you whether the destructor is executing *because* of a trap,
/// as opposed to just because the scope was exited, so you could e.g. implement mutex poisoning.
///
/// For information about when and how this occurs, see [the module docs](self).
pub fn is_recovering_from_trap() -> bool {
    ic_cdk_executor::is_recovering_from_trap()
}

/// Like `spawn`, but preserves the code ordering behavior of `ic-cdk` 0.17 and before.
///
/// Namely, the spawned future will start executing immediately, with control returning to the surrounding code
/// after the first `await`.
pub fn spawn_017_compat<F: 'static + Future<Output = ()>>(fut: F) {
    struct DummyWaker(AtomicBool);
    impl Wake for DummyWaker {
        fn wake(self: Arc<Self>) {
            self.0.store(true, Ordering::SeqCst);
        }
    }
    // Emulated behavior: A spawned future is polled once immediately, then backgrounded and run at a normal pace.
    // We poll it once with an unimplemented waker, then spawn it, which will poll it again with the real waker.
    // In a correctly implemented future, this second poll should overwrite the fake waker with the real one.
    // If the `poll` function calls `wake`, call it again until it is 'really' pending.
    let mut pin = Box::pin(fut);
    loop {
        let dummy = Arc::new(DummyWaker(AtomicBool::new(false)));
        let poll = pin
            .as_mut()
            .poll(&mut Context::from_waker(&Waker::from(dummy.clone())));
        match poll {
            Poll::Ready(()) => break,
            Poll::Pending => {
                if dummy.0.load(Ordering::SeqCst) {
                    continue;
                } else {
                    crate::futures::spawn(pin);
                    break;
                }
            }
        }
    }
}
