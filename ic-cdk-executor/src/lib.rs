//! An async executor for [`ic-cdk`](https://docs.rs/ic-cdk). Most users should not use this crate directly.
//!
//! ## Contexts
//!
//! The expected boilerplate for a canister method or other entrypoint (*not* including callbacks) looks like this:
//!
//! ```
//! pub extern "C" fn function() {
//!     in_executor_context(|| {
//!         spawn_entering_protected_scope(async {
//!             // ...
//!         })
//!     })
//! }
//! ```
//!
//! The `in_executor_context` function permits you to call `spawn_*` functions. The `spawn_entering_protected_scope` function
//! permits you to call `spawn_protected` in particular. As little code as possible should exist outside the innermost block,
//! because `in_executor_context` additionally sets up the panic handler, and because protected scopes only match the semantics
//! of canister methods if the entire method is inside the protected scope.
//!
//! The above applies to update contexts. Query contexts, including `inspect_message`, should use `in_query_executor_context`.
//!
//! The expected boilerplate for an inter-canister call callback looks like this:
//!
//! ```
//! unsafe extern "C" fn callback(env: usize) {
//!     in_callback_executor_context(|| {
//!        // wake the call future
//!     });
//! }
//! unsafe extern "C" fn cleanup(env: usize) {
//!    in_callback_cancellation_context(|| {
//!        // wake the call future
//!    });
//! }
//! ```
//!
//! In the cancellation context, 'waking' the task will actually cancel it, and no futures
//! will be executed. In the other contexts, all scheduled tasks are run *after* the closure passed to the context function
//! returns, but *before* the context function itself returns.
//!
//! Only one task should be awoken in `in_callback_executor_context`, it should only be the task for the single canister method
//! call that it is a callback for, and no additional tasks should be spawned.
//!
//! ## Protection
//!
//! The `spawn` function creates opportunities for footguns. A task can suspend in one context and wake up in another.
//! Method-specific global state such as `msg_caller` will change when this happens.
//!
//! Protected scopes allow affixing a task  to a particular method permanently; if a task is spawned within `spawn_protected`,
//! waking it outside a callback will not schedule it immediately, but rather it will be scheduled when the protected scope
//! has also been scheduled.
//!
//! If the scope ends and the protected task is still pending, it will be canceled.

use std::cell::{Cell, RefCell};
use std::collections::VecDeque;
use std::future::Future;
use std::mem;
use std::pin::Pin;
use std::rc::Rc;
use std::sync::Arc;
use std::task::{Context, Poll, Wake, Waker};

use pin_project_lite::pin_project;
use slotmap::{new_key_type, Key, SlotMap, SparseSecondaryMap};

/// Spawn an asynchronous task to run in the background.
pub fn spawn<F: 'static + Future<Output = ()>>(future: F) {
    spawn_inner(future, None);
}

/// Spawn a context-protected asynchronous task to run in the background.
///
/// This function can only be called from a protected scope. If the future is still pending when that scope ends,
/// the task will be canceled, and its destructors will be run. The future will only be polled when the protected
/// scope is polled.
pub fn spawn_protected<F: 'static + Future<Output = ()>>(future: F) {
    // A protected task propagates its protection scope to any tasks it spawns. Extract the current scope first.
    let Some(scope_id) = PROTECTION_SCOPE.get() else {
        panic!("`spawn_protected` called from outside a protected scope (are you on an older CDK version than your libraries?)");
    };
    pin_project! {
        struct PropagateScope<F> {
            #[pin]
            future: F,
            scope_id: TaskId,
        }
    }
    impl<F: Future<Output = ()>> Future for PropagateScope<F> {
        type Output = ();
        fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
            let this = self.project();
            // Whenever this future is executing, PROTECTION_SCOPE is set to the scope extracted above.
            // Other calls to spawn_protected will then read this value.
            let tmp = PROTECTION_SCOPE.replace(Some(*this.scope_id));
            struct Guard(Option<TaskId>);
            impl Drop for Guard {
                fn drop(&mut self) {
                    PROTECTION_SCOPE.set(self.0);
                }
            }
            let _guard = Guard(tmp);
            this.future.poll(cx)
        }
    }
    // Spawn attaching the future to the current scope.
    spawn_inner(PropagateScope { future, scope_id }, Some(scope_id));
}

pub fn spawn_entering_protection_scope<F: 'static + Future<Output = ()>>(future: F) {
    // Protection scopes cannot be nested. Otherwise the propagation behavior would get surprising, and also its behavior
    // is only useful when encasing the entire canister method.
    assert!(
        PROTECTION_SCOPE.get().is_none(),
        "`spawn_entering_protected_scope` called from within another protected scope"
    );
    // To be able to set the scope, the task must know its own task ID. It will be initialized to null, spawned,
    // then replaced with the real ID after it is spawned and before it is executed.
    pin_project! {
        struct ProtectedScope<F> {
            #[pin]
            future: F,
            self_id: Rc<Cell<TaskId>>,
        }
    }
    impl<F: Future<Output = ()>> Future for ProtectedScope<F> {
        type Output = ();
        fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
            let this = self.project();
            let task_id = this.self_id.get();
            assert!(!task_id.is_null(), "internal error: null scope ID");
            // Whenever this future is executing, PROTECTION_SCOPE is set to the scope extracted above.
            // Calls to spawn_protected will then read this value.
            let tmp = PROTECTION_SCOPE.replace(Some(task_id));
            struct Guard(Option<TaskId>);
            impl Drop for Guard {
                fn drop(&mut self) {
                    PROTECTION_SCOPE.set(self.0);
                }
            }
            let _guard = Guard(tmp);
            this.future.poll(cx)
        }
    }
    let transmitter = Rc::new(Cell::new(TaskId::null()));
    let task_id = spawn_inner(
        ProtectedScope {
            future,
            self_id: transmitter.clone(),
        },
        None,
    );
    // Inform the scoping task about its own ID
    transmitter.set(task_id);
}

fn spawn_inner<F: 'static + Future<Output = ()>>(
    future: F,
    protection_scope: Option<TaskId>,
) -> TaskId {
    let Some(context) = CONTEXT.get() else {
        panic!("`spawn` can only be called from an executor context")
    };
    let in_query = match context.mode {
        ContextMode::Query => true,
        ContextMode::Update => false,
        ContextMode::Cancel => panic!("`spawn` cannot be called during panic recovery"),
        ContextMode::FromTask => unreachable!("FromTask"),
    };
    let pinned_future = Box::pin(future);
    let task = Task {
        future: pinned_future,
        query: in_query,
        protection_scope,
    };
    let task_id = TASKS.with_borrow_mut(|tasks| tasks.insert(task));
    WAKEUP.with_borrow_mut(|wakeup| wakeup.push_back(task_id));
    if let Some(scope_id) = protection_scope {
        TASK_PROTECTION_PARENTS.with_borrow_mut(|parents| {
            if let Some(entry) = parents.entry(scope_id) {
                entry.or_default().push(task_id);
            }
        });
    }
    task_id
}

/// Execute an update function in a context that allows calling [`spawn`] and notifying wakers.
pub fn in_executor_context<R>(f: impl FnOnce() -> R) -> R {
    let _guard = ContextGuard::new(ContextMode::Update);
    let res = f();
    poll_all();
    res
}

/// Execute a composite query function in a context that allows calling [`spawn`] and notifying wakers.
pub fn in_query_executor_context<R>(f: impl FnOnce() -> R) -> R {
    let _guard = ContextGuard::new(ContextMode::Query);
    let res = f();
    poll_all();
    res
}

/// Execute an inter-canister-call callback in a context that allows calling [`spawn`] and notifying wakers.
pub fn in_callback_executor_context(f: impl FnOnce()) {
    let _guard = ContextGuard::new(ContextMode::FromTask);
    f();
    poll_all();
}

/// Execute an inter-canister-call callback in a context that allows calling [`spawn`] and notifying wakers,
/// but will cancel every awoken future.
pub fn in_callback_cancellation_context(f: impl FnOnce()) {
    let _guard = ContextGuard::new(ContextMode::Cancel);
    f();
}

/// Tells you whether the current async fn is being canceled due to a trap/panic.
pub fn is_recovering_from_trap() -> bool {
    matches!(CONTEXT.get(), Some(context) if context.mode == ContextMode::Cancel)
}

fn poll_all() {
    let Some(context) = CONTEXT.get() else {
        panic!("tasks can only be polled in an executor context");
    };
    let in_query = match context.mode {
        ContextMode::Query => true,
        ContextMode::Update => false,
        ContextMode::FromTask => unreachable!("FromTask"),
        ContextMode::Cancel => unreachable!("poll_all should not be called during panic recovery"),
    };
    let mut ineligible = vec![];
    let mut removing = vec![];
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
        // Background tasks cannot be resumed during a query method, since they may call non-query-compatible functions.
        if in_query && !task.query {
            TASKS.with_borrow_mut(|tasks| tasks[task_id] = task);
            ineligible.push(task_id);
            continue;
        }
        // If an actively scheduled task is a protection scope, or if it is protected by a scope,
        // all other tasks in that scope are clear to be scheduled too.
        if let Some(protection_scope) = task.protection_scope {
            if let Some(children) = WAKEUP_IN_PROTECTED_CONTEXT
                .with_borrow_mut(|wakeup| wakeup.remove(protection_scope))
            {
                WAKEUP.with_borrow_mut(|queue| queue.extend(children));
            }
        } else if let Some(children) =
            WAKEUP_IN_PROTECTED_CONTEXT.with_borrow_mut(|wakeup| wakeup.remove(task_id))
        {
            WAKEUP.with_borrow_mut(|queue| queue.extend(children));
        }
        // Poll the task.
        let waker = Waker::from(Arc::new(TaskWaker {
            task_id,
            query: task.query,
            protection_scope: task.protection_scope,
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
                // task complete, we will be deleting it in a second but first giving scope children a chance to complete
                removing.push(task_id);
                TASKS.with_borrow_mut(|tasks| tasks[task_id] = task);
            }
        }
    }
    if !ineligible.is_empty() {
        WAKEUP.with_borrow_mut(|wakeup| wakeup.extend(ineligible));
    }
    // delete completed tasks now that the queue is empty, which triggers cascading deletion of protection scope children
    for task_id in removing {
        cancel_task(task_id);
    }
}

fn cancel_task(task_id: TaskId) {
    // If the task was a protection scope, we need to remove its children.
    if let Some(children) =
        TASK_PROTECTION_PARENTS.with_borrow_mut(|parents| parents.remove(task_id))
    {
        // Because protection scopes are not nested, we can safely remove all children.
        for child in children {
            let _task = TASKS.with_borrow_mut(|tasks| tasks.remove(child));
        }
    }
    let _task = TASKS.with_borrow_mut(|tasks| tasks.remove(task_id));
    // tasks must be dropped *outside* with_borrow_mut - its destructor may (inadvisably) schedule tasks
}

new_key_type! {
    struct TaskId;
}

thread_local! {
    /// Global; the table of all the tasks the executor knows about. Removals should be done with cancel_task.
    static TASKS: RefCell<SlotMap<TaskId, Task>> = <_>::default();
    /// Global; a mapping of protection scope parent -> protected task.
    static TASK_PROTECTION_PARENTS: RefCell<SparseSecondaryMap<TaskId, Vec<TaskId>>> = <_>::default();
    /// Global; the list of every task that has been scheduled for wakeup in the current context.
    static WAKEUP: RefCell<VecDeque<TaskId>> = <_>::default();
    /// Global; the list of every protected task that should be scheduled for wakeup once its parent wakes up (map of parent -> children).
    static WAKEUP_IN_PROTECTED_CONTEXT: RefCell<SparseSecondaryMap<TaskId, Vec<TaskId>>> = <_>::default();
    /// Dynamically-scoped; the information from the `in_*_context function`.
    static CONTEXT: Cell<Option<AsyncContext>> = <_>::default();
    /// Dynamically-scoped; the current protection scope to be inherited by `spawn_protected` calls.
    static PROTECTION_SCOPE: Cell<Option<TaskId>> = <_>::default();
}

/// Corresponds to the `in_*_context` functions. `FromTask` gets replaced by the context the task was spawned in.
#[derive(Copy, Clone, PartialEq, Eq)]
enum ContextMode {
    Update,
    Query,
    FromTask,
    Cancel,
}

#[derive(Copy, Clone)]
struct AsyncContext {
    mode: ContextMode,
}

struct Task {
    future: Pin<Box<dyn Future<Output = ()>>>,
    query: bool,
    protection_scope: Option<TaskId>,
}

impl Default for Task {
    fn default() -> Self {
        Self {
            future: Box::pin(std::future::pending()),
            query: false,
            protection_scope: None,
        }
    }
}

/// Ensures the context is set properly, handling the reentrance check and clearing the context afterwards.
struct ContextGuard(());

impl ContextGuard {
    fn new(context: ContextMode) -> Self {
        CONTEXT.with(|context_var| {
            assert!(
                context_var.get().is_none(),
                "in_*_context called within an existing async context"
            );
            context_var.set(Some(AsyncContext { mode: context }));
            Self(())
        })
    }
}

impl Drop for ContextGuard {
    fn drop(&mut self) {
        CONTEXT.set(None);
    }
}

/// Waker implementation for executing futures produced by `call`/`call_raw`/etc.
///
/// *Almost* a straightforward executor, i.e. wakeups are addressed immediately for everything,
/// except it attempts to clean up tasks whose execution has trapped - see `call::is_recovering_from_trap`.
#[derive(Clone)]
struct TaskWaker {
    task_id: TaskId,
    // repeated from the task to preserve behavior even if the task has already been removed
    query: bool,
    protection_scope: Option<TaskId>,
}

impl Wake for TaskWaker {
    fn wake(self: Arc<Self>) {
        let Some(context) = CONTEXT.get() else {
            panic!("wakers cannot be called outside an executor context");
        };
        if context.mode == ContextMode::Cancel {
            cancel_task(self.task_id);
        } else {
            // If the task is protected to a scope, don't schedule it if this is in a different context
            if let Some(scope_id) = self.protection_scope {
                // FromTask context, i.e. method callback, is always the right scope. Otherwise check the current scope
                if context.mode == ContextMode::FromTask || PROTECTION_SCOPE.get() == Some(scope_id)
                {
                    WAKEUP.with_borrow_mut(|wakeup| wakeup.push_back(self.task_id));
                } else {
                    // Make sure the intended scope is still attachable.
                    if TASKS.with_borrow_mut(|tasks| !tasks.contains_key(scope_id)) {
                        // The protection scope has ended; this task should be canceled.
                        cancel_task(self.task_id);
                    } else {
                        // The waker was called in a different scope. Schedule it to be awoken when the right scope occurs.
                        WAKEUP_IN_PROTECTED_CONTEXT.with_borrow_mut(|wakeup| {
                            if let Some(entry) = wakeup.entry(scope_id) {
                                entry.or_default().push(self.task_id);
                            }
                        });
                    }
                }
            } else {
                WAKEUP.with_borrow_mut(|wakeup| wakeup.push_back(self.task_id));
            }
            if context.mode == ContextMode::FromTask {
                if self.query {
                    CONTEXT.set(Some(AsyncContext {
                        mode: ContextMode::Query,
                    }));
                } else {
                    CONTEXT.set(Some(AsyncContext {
                        mode: ContextMode::Update,
                    }));
                }
            }
        }
    }
}
