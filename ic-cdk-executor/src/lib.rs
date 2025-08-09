//! An async executor for [`ic-cdk`](https://docs.rs/ic-cdk). Most users should not use this crate directly. It is useful
//! primarily for those who are writing their own CDK or a runtime host for non-Rust languages.
//!
//! When you depend on this crate, it is recommended that you enable only the feature with the same name as the
//! current minor version:
//!
//! ```toml
//! [dependencies]
//! ic-cdk-executor = { version = "1.1.0", default-features = false, features = ["v1.1"] }
//! ```
//!
//! This is due to the API having multiple 'generations', but desiring to release as few major versions as possible.
//! Idiomatic usage of previous generations of the API is made compatible with idiomatic usage of later generations
//! on a best-effort basis, but mixing them in the same abstraction is not recommended.
//!
//! ## Contexts
//!
//! The expected boilerplate for a canister method or other entrypoint (*not* including callbacks) looks like this:
//!
//! ```
//! # use ic_cdk_executor::*;
//! pub extern "C" fn function() {
//!     in_tracking_executor_context(|| {
//!         // method goes here
//!     });
//! }
//! ```
//!
//! The `in_tracking_executor_context` function permits you to call `spawn_*` functions. As little code as possible
//! should exist outside the block, because `in_tracking_executor_context` additionally sets up the panic handler.
//!
//! The above applies to update contexts. Query contexts, including `inspect_message`, should use
//! `in_tracking_query_executor_context`.
//!
//! The expected boilerplate for an inter-canister call callback looks like this:
//!
//! ```
//! # use ic_cdk_executor::*;
//! # fn unpack_env(env: usize) -> MethodHandle { unimplemented!() }
//! unsafe extern "C" fn callback(env: usize) {
//!     let method = unpack_env(env);
//!     in_callback_executor_context_for(method, || {
//!        // wake the call future
//!     });
//! }
//! unsafe extern "C" fn cleanup(env: usize) {
//!     let method = unpack_env(env);
//!     in_trap_recovery_context_for(method, || {
//!         cancel_all_tasks_attached_to_current_method();
//!     });
//! }
//! ```
//!
//! In async contexts, all scheduled tasks are run *after* the closure passed to the context function
//! returns, but *before* the context function itself returns.
//!
//! The `method` parameter must be retrieved *before* making inter-canister calls via the [`extend_current_method_context`]
//! function. Calling this function from the callback instead will trap.
//!
//! ## Protection
//!
//! Tasks can be either *protected* or *migratory*. Protected tasks are attached to the method that spawned them,
//! when awoken will not resume until that method continues, and will be canceled if the method returns before they complete.
//! Migratory tasks are not attached to any method, and will resume in whatever method wakes them.

#[cfg(feature = "v1.0")]
mod legacy;
mod machinery;

#[cfg(feature = "v1.0")]
#[doc(inline)]
#[allow(deprecated)]
pub use legacy::{
    in_callback_cancellation_context, in_callback_executor_context, in_executor_context,
    in_query_executor_context, spawn,
};
#[cfg(any(feature = "v1.0", feature = "v1.1"))]
#[doc(inline)]
pub use machinery::is_recovering_from_trap;
#[cfg(feature = "v1.1")]
#[doc(inline)]
pub use machinery::{
    cancel_all_tasks_attached_to_current_method, cancel_task, extend_current_method_context,
    in_callback_executor_context_for, in_tracking_executor_context,
    in_tracking_query_executor_context, in_trap_recovery_context_for, spawn_migratory,
    spawn_protected, MethodHandle, TaskHandle,
};
