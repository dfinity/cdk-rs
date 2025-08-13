use std::{cell::Cell, future::Future, ops::ControlFlow};

use slotmap::Key;
use smallvec::SmallVec;

use crate::machinery::{
    CURRENT_METHOD, ContextKind, METHODS, MethodContext, MethodHandle, MethodId, RECOVERING,
    TaskWaker, delete_task, enter_current_method, in_null_context, poll_all, spawn_migratory,
    spawn_protected,
};

// This module adapts the 1.0 interface onto the 1.1 executor. It has the following limitations:
// 1. Method contexts cannot be traced. Code using the 1.0 callback contexts cannot support protected tasks.
// 2. Cancellation is based on the waker, not on the pending wakeups.
// 3. Relevant method information is stored in the waker, rather than in a method handle.
// So this module adapts the interface in the following ways:
// - The `spawn` function will create a migratory task in update methods.
// - The callback context functions will trap if they are asked to participate in method lifetime tracking, ie
//   if they are called from nonnull update contexts.
// - Query methods get a special `QUERY_METHOD` method, deliberately memleaked.
// - The `v0_wake_hook` function is called from the waker to infer context in callback contexts. Those context closures
//   set `INFER_CONTEXT` to indicate whether the context is a continuation or cancelation.

thread_local! {
    /// global: 1.0 queries are implemented as a single known query method context that is never freed
    pub(crate) static QUERY_METHOD: MethodId = METHODS.with_borrow_mut(|methods| methods.insert(MethodContext {
        // 1 handle will always be outstanding
        handles: 1,
        kind: ContextKind::Query,
        tasks: SmallVec::new(),
    }));
    /// dynamically scoped: set in 1.0 callback contexts to indicate that the context should be inferred from the waker.
    pub(crate) static INFER_CONTEXT: Cell<Option<InferContext>> = const { Cell::new(None) };
}

#[deprecated(
    since = "1.1.0",
    note = "Use `spawn_migratory` or `spawn_protected` instead"
)]
pub fn spawn<F: 'static + Future<Output = ()>>(future: F) {
    // 1.0 update spawns are represented as migratory tasks.
    // 1.0 query spawns are represented as protected tasks within QUERY_METHOD.
    let Some(current) = CURRENT_METHOD.get() else {
        panic!("`spawn` can only be called within an executor context");
    };
    let kind = METHODS.with_borrow(|methods| {
        if let Some(method) = methods.get(current) {
            method.kind
        } else {
            ContextKind::Update
        }
    });
    match kind {
        ContextKind::Query => spawn_protected(future),
        ContextKind::Update => spawn_migratory(future),
    };
}

#[deprecated(since = "1.1.0", note = "Use `in_tracking_executor_context` instead")]
pub fn in_executor_context<R>(f: impl FnOnce() -> R) -> R {
    // 1.0 update contexts are represented as null contexts.
    in_null_context(f)
}

#[deprecated(
    since = "1.1.0",
    note = "Use `in_tracking_query_executor_context` instead"
)]
pub fn in_query_executor_context<R>(f: impl FnOnce() -> R) -> R {
    // 1.0 query contexts are represented as tracking contexts for QUERY_METHOD.
    let guard = MethodHandle::for_method(QUERY_METHOD.with(|m| *m));
    enter_current_method(guard, |_| {
        let res = f();
        poll_all();
        res
    })
}

#[deprecated(
    since = "1.1.0",
    note = "Use `in_callback_executor_context_for` instead"
)]
pub fn in_callback_executor_context(f: impl FnOnce()) {
    // 1.0 callback contexts are inferred using v0_wake_hook.
    INFER_CONTEXT.set(Some(InferContext::Continue));
    in_null_context(f);
    INFER_CONTEXT.set(None);
}

#[deprecated(since = "1.1.0", note = "Use `in_trap_recovery_context_for` instead")]
pub fn in_callback_cancellation_context(f: impl FnOnce()) {
    // 1.0 cancellation contexts are inferred using v0_wake_hook.
    INFER_CONTEXT.set(Some(InferContext::Cancel));
    in_null_context(f);
    INFER_CONTEXT.set(None);
}

pub(crate) fn v0_wake_hook(waker: &TaskWaker) -> ControlFlow<()> {
    if CURRENT_METHOD.get().is_some_and(|m| m.is_null()) {
        match INFER_CONTEXT.get() {
            Some(InferContext::Continue) => {
                let v0_query_method = QUERY_METHOD.with(|m| *m);
                if waker.source_method == v0_query_method {
                    CURRENT_METHOD.set(Some(v0_query_method));
                } else if !waker.source_method.is_null() {
                    // It cannot be permitted to have 1.0 waker usage in 1.1 methods. The wake callsite is responsible
                    // for propagating method lifetime; without tracking it properly, protected tasks could leak
                    // or cancel early. This cannot be caught before the call, but we know whether it has occurred if a
                    // task spawned in a tracking context is woken from an inferred context.
                    panic!(
                        "waker API usage mismatch between canister method and inter-canister call; try running `cargo update -p ic-cdk`"
                    );
                }
                ControlFlow::Continue(())
            }
            Some(InferContext::Cancel) => {
                RECOVERING.set(true);
                delete_task(waker.task_id);
                RECOVERING.set(false);
                ControlFlow::Break(())
            }
            None => ControlFlow::Continue(()),
        }
    } else {
        ControlFlow::Continue(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum InferContext {
    Continue,
    Cancel,
}
