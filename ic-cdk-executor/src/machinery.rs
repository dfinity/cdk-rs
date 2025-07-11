#![cfg_attr(not(feature = "v1.1"), allow(dead_code))]

#[cfg(feature = "v1.0")]
use std::ops::ControlFlow;
use std::{
    cell::{Cell, RefCell},
    collections::VecDeque,
    future::Future,
    mem::take,
    pin::Pin,
    sync::{Arc, Once},
    task::{Context, Poll, Wake, Waker},
};

use slotmap::{new_key_type, HopSlotMap, Key, SecondaryMap, SlotMap};

#[derive(Copy, Clone, Debug)]
pub(crate) struct MethodContext {
    pub(crate) kind: ContextKind,
    pub(crate) handles: usize,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub(crate) enum ContextKind {
    Update,
    Query,
}

new_key_type! {
    pub(crate) struct MethodId;
    pub(crate) struct TaskId;
}

thread_local! {
    pub(crate) static METHODS: RefCell<SlotMap<MethodId, MethodContext>> = RefCell::default();
    pub(crate) static TASKS: RefCell<HopSlotMap<TaskId, Task>> = RefCell::default();
    pub(crate) static PROTECTED_WAKEUPS: RefCell<SecondaryMap<MethodId, VecDeque<TaskId>>> = RefCell::default();
    pub(crate) static MIGRATORY_WAKEUPS: RefCell<VecDeque<TaskId>> = const { RefCell::new(VecDeque::new()) };
    pub(crate) static CURRENT_METHOD: Cell<Option<MethodId>> = const { Cell::new(None) };
    pub(crate) static RECOVERING: Cell<bool> = const { Cell::new(false) };
}

pub(crate) struct Task {
    future: Pin<Box<dyn Future<Output = ()>>>,
    method_binding: Option<MethodId>,
}

impl Default for Task {
    fn default() -> Self {
        Self {
            future: Box::pin(std::future::pending()),
            method_binding: None,
        }
    }
}

/// Execute an update function in a context that allows calling [`spawn_protected`] and [`spawn_migratory`]
pub fn in_tracking_executor_context<R>(f: impl FnOnce() -> R) -> R {
    setup_panic_hook();
    let method = METHODS.with_borrow_mut(|methods| {
        methods.insert(MethodContext {
            kind: ContextKind::Update,
            handles: 0,
        })
    });
    let guard = MethodHandle::for_method(method);
    enter_current_method(guard, |_| {
        let res = f();
        poll_all();
        res
    })
}

/// Execute a function in a context that is not tracked across callbacks, able to call [`spawn_migratory`]
/// but not [`spawn_protected`].
pub fn in_null_context<R>(f: impl FnOnce() -> R) -> R {
    setup_panic_hook();
    let guard = MethodHandle::for_method(MethodId::null());
    enter_current_method(guard, |_| {
        let res = f();
        poll_all();
        res
    })
}

/// Execute a query function in a context that allows calling [`spawn_protected`] but not [`spawn_migratory`].
pub fn in_tracking_query_executor_context<R>(f: impl FnOnce() -> R) -> R {
    setup_panic_hook();
    let method = METHODS.with_borrow_mut(|methods| {
        methods.insert(MethodContext {
            kind: ContextKind::Query,
            handles: 0,
        })
    });
    let guard = MethodHandle::for_method(method);
    enter_current_method(guard, |_| {
        let res = f();
        poll_all();
        res
    })
}

/// Execute an inter-canister call callback in the context of the method that made it.
pub fn in_callback_executor_context_for<R>(
    method_handle: MethodHandle,
    f: impl FnOnce() -> R,
) -> R {
    setup_panic_hook();
    enter_current_method(method_handle, |_| {
        let res = f();
        poll_all();
        res
    })
}

/// Enters a panic recovery context for calling [`cancel_all_tasks_attached_to_current_method`] in.
pub fn in_trap_recovery_context_for<R>(method: MethodHandle, f: impl FnOnce() -> R) -> R {
    setup_panic_hook();
    enter_current_method(method, |_| {
        RECOVERING.set(true);
        let res = f();
        RECOVERING.set(false);
        res
    })
}

/// Cancels all tasks made with [`spawn_protected`] attached to the current method.
pub fn cancel_all_tasks_attached_to_current_method() {
    let Some(method_id) = CURRENT_METHOD.get() else {
        panic!("`cancel_all_tasks_attached_to_current_method` can only be called within a method context");
    };
    let _tasks = TASKS.with(|tasks| {
        let Ok(mut tasks) = tasks.try_borrow_mut() else {
            panic!(
                "`cancel_all_tasks_attached_to_current_method` cannot be called from an async task"
            );
        };
        let mut to_cancel = vec![];
        for (task_id, task) in tasks.iter_mut() {
            if task.method_binding == Some(method_id) {
                to_cancel.push(task_id);
            }
        }
        let mut canceled = Vec::with_capacity(to_cancel.len());
        for task_id in to_cancel {
            canceled.push(tasks.remove(task_id));
        }
        canceled
    });
    drop(_tasks); // always run task destructors outside of a refcell borrow
}

pub(crate) fn cancel_task(task_id: TaskId) {
    let _task = TASKS.with_borrow_mut(|tasks| tasks.remove(task_id));
    drop(_task); // always run task destructors outside of a refcell borrow
}

/// Returns true if tasks are being canceled due to a trap or panic.
pub fn is_recovering_from_trap() -> bool {
    RECOVERING.get()
}

/// Produces a handle to the current method context.
///
/// The method is active as long as the handle is alive.
pub fn extend_current_method_context() -> MethodHandle {
    setup_panic_hook();
    let Some(method_id) = CURRENT_METHOD.get() else {
        panic!("`extend_method_context` can only be called within a tracking executor context");
    };
    MethodHandle::for_method(method_id)
}

pub(crate) fn poll_all() {
    let Some(method_id) = CURRENT_METHOD.get() else {
        panic!("tasks can only be polled within an executor context");
    };
    let kind =
        if let Some(kind) = METHODS.with_borrow(|methods| methods.get(method_id).map(|m| m.kind)) {
            kind
        } else {
            ContextKind::Update
        };
    fn pop_wakeup(method_id: MethodId, update: bool) -> Option<TaskId> {
        if let Some(task_id) = PROTECTED_WAKEUPS.with_borrow_mut(|wakeups| {
            wakeups
                .get_mut(method_id)
                .and_then(|queue| queue.pop_front())
        }) {
            Some(task_id)
        } else if update {
            MIGRATORY_WAKEUPS.with_borrow_mut(|unattached| unattached.pop_front())
        } else {
            None
        }
    }
    while let Some(task_id) = pop_wakeup(method_id, kind == ContextKind::Update) {
        // Temporarily remove the task from the table. We need to execute it while `TASKS` is not borrowed, because it may schedule more tasks.
        let Some(mut task) = TASKS.with_borrow_mut(|tasks| tasks.get_mut(task_id).map(take)) else {
            // This waker handle appears to be dead. The most likely cause is that the method returned before
            // a canceled call came back.
            continue;
            // In the case that a task panicked and that's why it's missing, but it was in an earlier callback so a later
            // one tries to re-wake, the responsibility for re-trapping lies with CallFuture.
        };
        let waker = Waker::from(Arc::new(TaskWaker {
            task_id,
            #[cfg(feature = "v1.0")]
            source_method: method_id,
        }));
        let poll = task.future.as_mut().poll(&mut Context::from_waker(&waker));
        match poll {
            Poll::Pending => {
                // more to do, put the task back in the table
                TASKS.with_borrow_mut(|tasks| {
                    if let Some(t) = tasks.get_mut(task_id) {
                        *t = task;
                    }
                });
            }
            Poll::Ready(()) => {
                // task complete, remove its entry from the table fully
                TASKS.with_borrow_mut(|tasks| tasks.remove(task_id));
            }
        }
    }
}

pub(crate) fn enter_current_method<R>(
    method_guard: MethodHandle,
    f: impl FnOnce(MethodId) -> R,
) -> R {
    CURRENT_METHOD.with(|context_var| {
        assert!(
            context_var.get().is_none(),
            "in_*_context called within an existing async context"
        );
        context_var.set(Some(method_guard.method_id));
    });
    let r = f(method_guard.method_id);
    drop(method_guard); // drop the guard *before* the method freeing logic, but *after* the in-context code
    let method_id = CURRENT_METHOD.replace(None);
    if let Some(method_id) = method_id {
        METHODS.with_borrow_mut(|methods| {
            if let Some(method) = methods.get(method_id) {
                if method.handles == 0 {
                    methods.remove(method_id);
                }
            }
        });
    }
    r
}

/// A handle to a method context. If the function returns and all handles have been dropped, the method is considered returned.
///
/// This should be created before performing an inter-canister call via [`extend_current_method_context`],
/// threaded through the `env` parameter, and then used when calling [`in_callback_executor_context_for`] or
/// [`in_trap_recovery_context_for`]. Failure to track this properly may result in unexpected cancellation of tasks.
#[derive(Debug)]
pub struct MethodHandle {
    method_id: MethodId,
}

impl MethodHandle {
    pub(crate) fn for_method(method_id: MethodId) -> Self {
        if method_id.is_null() {
            return Self { method_id };
        }
        METHODS.with_borrow_mut(|methods| {
            let Some(method) = methods.get_mut(method_id) else {
                panic!("internal error: method context deleted while in use (for_method)");
            };
            method.handles += 1;
        });
        Self { method_id }
    }
}

impl Drop for MethodHandle {
    fn drop(&mut self) {
        METHODS.with_borrow_mut(|methods| {
            if let Some(method) = methods.get_mut(self.method_id) {
                method.handles -= 1;
            }
        })
    }
}

/// A handle to a spawned task.
pub struct TaskHandle {
    _task_id: TaskId,
}

pub(crate) struct TaskWaker {
    pub(crate) task_id: TaskId,
    #[cfg(feature = "v1.0")]
    pub(crate) source_method: MethodId,
}

impl Wake for TaskWaker {
    fn wake(self: Arc<Self>) {
        #[cfg(feature = "v1.0")]
        if crate::legacy::v0_wake_hook(&self) == ControlFlow::Break(()) {
            return;
        }
        TASKS.with_borrow_mut(|tasks| {
            if let Some(task) = tasks.get(self.task_id) {
                if let Some(method_id) = task.method_binding {
                    PROTECTED_WAKEUPS.with_borrow_mut(|wakeups| {
                        if let Some(entry) = wakeups.entry(method_id) {
                            entry.or_default().push_back(self.task_id);
                        }
                    });
                } else {
                    MIGRATORY_WAKEUPS.with_borrow_mut(|unattached| {
                        unattached.push_back(self.task_id);
                    });
                }
            }
        })
    }
}

/// Spawns a task that can migrate between methods.
///
/// When the task is awoken, it will run in the context of the method that woke it.
pub fn spawn_migratory(f: impl Future<Output = ()> + 'static) -> TaskHandle {
    setup_panic_hook();
    let Some(method_id) = CURRENT_METHOD.get() else {
        panic!("`spawn_*` can only be called within an executor context");
    };
    if is_recovering_from_trap() {
        panic!("tasks cannot be spawned while recovering from a trap");
    }
    let kind = METHODS.with_borrow(|methods| {
        if let Some(method) = methods.get(method_id) {
            method.kind
        } else {
            ContextKind::Update
        }
    });
    if kind == ContextKind::Query {
        panic!("unprotected spawns cannot be made within a query context");
    }
    let task = Task {
        future: Box::pin(f),
        method_binding: None,
    };
    let task_id = TASKS.with_borrow_mut(|tasks| tasks.insert(task));
    MIGRATORY_WAKEUPS.with_borrow_mut(|unattached| {
        unattached.push_back(task_id);
    });
    TaskHandle { _task_id: task_id }
}

/// Spawns a task attached to the current method.
///
/// When the task is awoken, if a different method is currently running, it will not run until the method
/// it is attached to continues. If the attached method returns before the task completes, it will be canceled.
pub fn spawn_protected(f: impl Future<Output = ()> + 'static) -> TaskHandle {
    setup_panic_hook();
    if is_recovering_from_trap() {
        panic!("tasks cannot be spawned while recovering from a trap");
    }
    let Some(method_id) = CURRENT_METHOD.get() else {
        panic!("`spawn_*` can only be called within an executor context");
    };
    if method_id.is_null() {
        panic!("`spawn_protected` cannot be called outside of a tracking context");
    }
    let task = Task {
        future: Box::pin(f),
        method_binding: Some(method_id),
    };
    let task_id = TASKS.with_borrow_mut(|tasks| tasks.insert(task));
    PROTECTED_WAKEUPS.with_borrow_mut(|wakeups| {
        let Some(entry) = wakeups.entry(method_id) else {
            panic!("internal error: method context deleted while in use (spawn_protected)");
        };
        entry.or_default().push_back(task_id);
    });
    TaskHandle { _task_id: task_id }
}

fn setup_panic_hook() {
    static SETUP: Once = Once::new();
    SETUP.call_once(|| {
        std::panic::set_hook(Box::new(|info| {
            let file = info.location().unwrap().file();
            let line = info.location().unwrap().line();
            let col = info.location().unwrap().column();

            let msg = match info.payload().downcast_ref::<&'static str>() {
                Some(s) => *s,
                None => match info.payload().downcast_ref::<String>() {
                    Some(s) => &s[..],
                    None => "Box<Any>",
                },
            };

            let err_info = format!("Panicked at '{msg}', {file}:{line}:{col}");
            ic0::debug_print(err_info.as_bytes());
            ic0::trap(err_info.as_bytes());
        }));
    });
}
