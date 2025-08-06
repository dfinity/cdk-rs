//! An async executor for [`ic-cdk`](https://docs.rs/ic-cdk). Most users should not use this crate directly.

use std::cell::{Cell, RefCell};
use std::collections::VecDeque;
use std::future::Future;
use std::mem;
use std::pin::Pin;
use std::sync::{Arc, Once};
use std::task::{Context, Poll, Wake, Waker};

use slotmap::{SlotMap, new_key_type};

/// Spawn an asynchronous task to run in the background.
pub fn spawn<F: 'static + Future<Output = ()>>(future: F) {
    set_panic_hook();
    let in_query = match CONTEXT.get() {
        AsyncContext::None => panic!("`spawn` can only be called from an executor context"),
        AsyncContext::Query => true,
        AsyncContext::Update => false,
        AsyncContext::Cancel => panic!("`spawn` cannot be called during panic recovery"),
        AsyncContext::FromTask => unreachable!("FromTask"),
    };
    let pinned_future = Box::pin(future);
    let task = Task {
        future: pinned_future,
        query: in_query,
    };
    let task_id = TASKS.with_borrow_mut(|tasks| tasks.insert(task));
    WAKEUP.with_borrow_mut(|wakeup| wakeup.push_back(task_id));
}

/// Execute an update function in a context that allows calling [`spawn`] and notifying wakers.
pub fn in_executor_context<R>(f: impl FnOnce() -> R) -> R {
    set_panic_hook();
    let _guard = ContextGuard::new(AsyncContext::Update);
    let res = f();
    poll_all();
    res
}

/// Execute a composite query function in a context that allows calling [`spawn`] and notifying wakers.
pub fn in_query_executor_context<R>(f: impl FnOnce() -> R) -> R {
    set_panic_hook();
    let _guard = ContextGuard::new(AsyncContext::Query);
    let res = f();
    poll_all();
    res
}

/// Execute an inter-canister-call callback in a context that allows calling [`spawn`] and notifying wakers.
pub fn in_callback_executor_context(f: impl FnOnce()) {
    set_panic_hook();
    let _guard = ContextGuard::new(AsyncContext::FromTask);
    f();
    poll_all();
}

/// Execute an inter-canister-call callback in a context that allows calling [`spawn`] and notifying wakers,
/// but will cancel every awoken future.
pub fn in_callback_cancellation_context(f: impl FnOnce()) {
    set_panic_hook();
    let _guard = ContextGuard::new(AsyncContext::Cancel);
    f();
}

/// Tells you whether the current async fn is being canceled due to a trap/panic.
pub fn is_recovering_from_trap() -> bool {
    matches!(CONTEXT.get(), AsyncContext::Cancel)
}

fn poll_all() {
    let in_query = match CONTEXT.get() {
        AsyncContext::Query => true,
        AsyncContext::Update => false,
        AsyncContext::None => panic!("tasks can only be polled in an executor context"),
        AsyncContext::FromTask => unreachable!("FromTask"),
        AsyncContext::Cancel => unreachable!("poll_all should not be called during panic recovery"),
    };
    let mut ineligible = vec![];
    while let Some(task_id) = WAKEUP.with_borrow_mut(|queue| queue.pop_front()) {
        // Temporarily remove the task from the table. We need to execute it while `TASKS` is not borrowed, because it may schedule more tasks.
        let Some(mut task) = TASKS.with_borrow_mut(|tasks| tasks.get_mut(task_id).map(mem::take))
        else {
            // This waker handle appears to be dead. The most likely cause is that the method returned before
            // a canceled call came back.
            continue;
            // In the case that a task panicked and that's why it's missing, but it was in an earlier callback so a later
            // one tries to re-wake, the responsibility for re-trapping lies with CallFuture.
        };
        if in_query && !task.query {
            TASKS.with_borrow_mut(|tasks| tasks[task_id] = task);
            ineligible.push(task_id);
            continue;
        }
        let waker = Waker::from(Arc::new(TaskWaker {
            task_id,
            query: task.query,
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
    if !ineligible.is_empty() {
        WAKEUP.with_borrow_mut(|wakeup| wakeup.extend(ineligible));
    }
}

new_key_type! {
    struct TaskId;
}

thread_local! {
    static TASKS: RefCell<SlotMap<TaskId, Task>> = <_>::default();
    static WAKEUP: RefCell<VecDeque<TaskId>> = <_>::default();
    static CONTEXT: Cell<AsyncContext> = <_>::default();
}

#[derive(Default, Copy, Clone, PartialEq, Eq)]
enum AsyncContext {
    #[default]
    None,
    Update,
    Query,
    FromTask,
    Cancel,
}

struct Task {
    future: Pin<Box<dyn Future<Output = ()>>>,
    query: bool,
}

impl Default for Task {
    fn default() -> Self {
        Self {
            future: Box::pin(std::future::pending()),
            query: false,
        }
    }
}

struct ContextGuard(());

impl ContextGuard {
    fn new(context: AsyncContext) -> Self {
        CONTEXT.with(|context_var| {
            assert!(
                matches!(context_var.get(), AsyncContext::None),
                "in_*_context called within an existing async context"
            );
            context_var.set(context);
            Self(())
        })
    }
}

impl Drop for ContextGuard {
    fn drop(&mut self) {
        CONTEXT.set(AsyncContext::None);
    }
}

/// Waker implementation for executing futures produced by `call`/`call_raw`/etc.
///
/// *Almost* a straightforward executor, i.e. wakeups are addressed immediately for everything,
/// except it attempts to clean up tasks whose execution has trapped - see `call::is_recovering_from_trap`.
#[derive(Clone)]
struct TaskWaker {
    task_id: TaskId,
    query: bool,
}

impl Wake for TaskWaker {
    fn wake(self: Arc<Self>) {
        set_panic_hook();
        let context = CONTEXT.get();
        assert!(
            context != AsyncContext::None,
            "wakers cannot be called outside an executor context"
        );
        if context == AsyncContext::Cancel {
            // This task is recovering from a trap. We cancel it to run destructors.
            let _task = TASKS.with_borrow_mut(|tasks| tasks.remove(self.task_id));
            // _task must be dropped *outside* with_borrow_mut - its destructor may (inadvisably) schedule tasks
        } else {
            WAKEUP.with_borrow_mut(|wakeup| wakeup.push_back(self.task_id));
            if context == AsyncContext::FromTask {
                if self.query {
                    CONTEXT.set(AsyncContext::Query);
                } else {
                    CONTEXT.set(AsyncContext::Update);
                }
            }
        }
    }
}

pub fn set_panic_hook() {
    static PANIC_HOOK_SET: Once = Once::new();
    PANIC_HOOK_SET.call_once(|| {
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
