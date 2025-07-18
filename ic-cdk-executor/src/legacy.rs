use std::{cell::Cell, future::Future, ops::ControlFlow};

use slotmap::Key;

use crate::machinery::{
    cancel_task, enter_current_method, in_null_context, poll_all, spawn_migratory, spawn_protected,
    ContextKind, MethodContext, MethodHandle, MethodId, TaskWaker, CURRENT_METHOD, METHODS,
    RECOVERING,
};

thread_local! {
    pub(crate) static QUERY_METHOD: MethodId = METHODS.with_borrow_mut(|methods| methods.insert(MethodContext {
        handles: 1,
        kind: ContextKind::Query,
    }));
    pub(crate) static INFER_CONTEXT: Cell<Option<InferContext>> = const { Cell::new(None) };
}

#[deprecated(
    since = "1.1.0",
    note = "Use `spawn_migratory` or `spawn_protected` instead"
)]
pub fn spawn<F: 'static + Future<Output = ()>>(future: F) {
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
    in_null_context(f)
}

#[deprecated(
    since = "1.1.0",
    note = "Use `in_tracking_query_executor_context` instead"
)]
pub fn in_query_executor_context<R>(f: impl FnOnce() -> R) -> R {
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
    INFER_CONTEXT.set(Some(InferContext::Continue));
    in_null_context(f);
    INFER_CONTEXT.set(None);
}

#[deprecated(since = "1.1.0", note = "Use `in_trap_recovery_context_for` instead")]
pub fn in_callback_cancellation_context(f: impl FnOnce()) {
    INFER_CONTEXT.set(Some(InferContext::Cancel));
    in_null_context(f);
    INFER_CONTEXT.set(None);
}

pub(crate) fn v0_wake_hook(waker: &TaskWaker) -> ControlFlow<()> {
    if CURRENT_METHOD.get().is_some_and(|m| m.is_null()) {
        match INFER_CONTEXT.get() {
            Some(InferContext::Continue) => {
                if METHODS.with_borrow(|methods| {
                    methods
                        .get(waker.source_method)
                        .is_some_and(|method| method.kind == ContextKind::Query)
                }) {
                    CURRENT_METHOD.set(Some(QUERY_METHOD.with(|m| *m)));
                }
                ControlFlow::Continue(())
            }
            Some(InferContext::Cancel) => {
                RECOVERING.set(true);
                cancel_task(waker.task_id);
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
