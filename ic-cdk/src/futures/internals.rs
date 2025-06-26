//! Functions used internally by the entrypoint macros.
//!
//! You do not need to use anything in this module unless you are eschewing the macros and writing your own exported symbols.

use std::future::Future;

/// Execute an update function in a context that allows calling [`spawn_entering_protection_scope`] and
/// [`spawn_migratory`](super::spawn_migratory).
///
/// Background tasks will be polled in the process (and will not be run otherwise).
/// Panics if called inside an existing executor context.
pub fn in_executor_context<R>(f: impl FnOnce() -> R) -> R {
    crate::setup();
    ic_cdk_executor::in_executor_context(f)
}

/// Execute a composite query function in a context that allows calling [`spawn_entering_protection_scope`].
///
/// Background composite query tasks will be polled in the process (and will not be run otherwise).
/// Panics if called inside an existing executor context.
pub fn in_query_executor_context<R>(f: impl FnOnce() -> R) -> R {
    crate::setup();
    ic_cdk_executor::in_query_executor_context(f)
}

/// Execute the body of an async method in the context that allows calling [`spawn_local`](super::spawn_local)
/// and [`spawn_weak`](super::spawn_weak).
pub fn spawn_entering_protection_scope<F: 'static + Future<Output = ()>>(fut: F) {
    ic_cdk_executor::spawn_entering_protection_scope(fut);
}
