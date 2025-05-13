//! An async executor for [`ic-cdk`](https://docs.rs/ic-cdk). Most users should not use this crate directly.

use std::cell::{Cell, RefCell};
use std::collections::VecDeque;
use std::future::Future;
use std::mem;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll, Wake, Waker};

use slotmap::{new_key_type, SlotMap};

/// Spawn an asynchronous task to run in the background.
pub fn spawn<F: 'static + Future<Output = ()>>(future: F) {
    let in_query = match CONTEXT.get() {
        AsyncContext::None => panic!("`spawn` can only be called from an executor context"),
        AsyncContext::Query => true,
        AsyncContext::Update | AsyncContext::Cancel => false,
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
    let _guard = ContextGuard::new(AsyncContext::Update);
    let res = f();
    poll_all();
    res
}

/// Execute a composite query function in a context that allows calling [`spawn`] and notifying wakers.
pub fn in_query_executor_context<R>(f: impl FnOnce() -> R) -> R {
    let _guard = ContextGuard::new(AsyncContext::Query);
    let res = f();
    poll_all();
    res
}

/// Execute an inter-canister-call callback in a context that allows calling [`spawn`] and notifying wakers.
pub fn in_callback_executor_context(f: impl FnOnce()) {
    let _guard = ContextGuard::new(AsyncContext::FromTask);
    f();
    poll_all();
}

/// Execute an inter-canister-call callback in a context that allows calling [`spawn`] and notifying wakers,
/// but will cancel every awoken future.
pub fn in_callback_cancellation_context(f: impl FnOnce()) {
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
        AsyncContext::Update | AsyncContext::Cancel => false,
        AsyncContext::None => panic!("tasks can only be polled in an executor context"),
        AsyncContext::FromTask => unreachable!("FromTask"),
    };
    let mut ineligible = vec![];
    while let Some(task_id) = WAKEUP.with_borrow_mut(|queue| queue.pop_front()) {
        // Temporarily remove the task from the table. We need to execute it while `TASKS` is not borrowed, because it may schedule more tasks.
        let Some(mut task) = TASKS.with_borrow_mut(|tasks| tasks.get_mut(task_id).map(mem::take))
        else {
            // The task is dropped on the first callback that panics, but the last callback is the one that sets the flag.
            // So if multiple calls are sent concurrently, the waker will be asked to wake a future that no longer exists.
            // This should be the only possible case in which this happens.
            panic!("Call already trapped");
            // This also should not happen because the CallFuture handles this itself. But FuturesUnordered introduces some chaos.
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

#[derive(Default, Copy, Clone)]
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
/// *Almost* a do-nothing executor, i.e. wake directly calls poll with no scheduler, except it attempts to clean up tasks
/// whose execution has trapped - see `call::is_recovering_from_trap`.
#[derive(Clone)]
struct TaskWaker {
    task_id: TaskId,
    query: bool,
}

impl Wake for TaskWaker {
    fn wake(self: Arc<Self>) {
        if matches!(CONTEXT.get(), AsyncContext::Cancel) {
            // This task is recovering from a trap. We cancel it to run destructors.
            let _task = TASKS.with_borrow_mut(|tasks| tasks.remove(self.task_id));
            // _task must be dropped *outside* with_borrow_mut - its destructor may (inadvisably) schedule tasks
        } else {
            WAKEUP.with_borrow_mut(|wakeup| wakeup.push_back(self.task_id));
            CONTEXT.with(|context| {
                if matches!(context.get(), AsyncContext::FromTask) {
                    if self.query {
                        context.set(AsyncContext::Query)
                    } else {
                        context.set(AsyncContext::Update)
                    }
                }
            })
        }
    }
}
