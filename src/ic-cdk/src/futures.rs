use std::cell::RefCell;
use std::future::Future;
use std::mem;
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::task::{Context, Poll, Wake, Waker};

use slotmap::{new_key_type, SlotMap};

/// Must be called on every top-level future corresponding to a method call of a
/// canister by the IC, other than async functions marked `#[update]` or similar.
#[cfg_attr(not(target_arch = "wasm32"), allow(unused_variables, unreachable_code))]
pub fn spawn<F: 'static + Future<Output = ()>>(future: F) {
    #[cfg(not(target_arch = "wasm32"))]
    panic!("Cannot be run outside of wasm!"); // really, just cannot be run in a multi-threaded environment
    let pinned_future = Box::pin(future);
    let task = Task {
        future: pinned_future,
    };
    let task_id = TASKS.with_borrow_mut(|tasks| tasks.insert(task));
    WAKEUP.with_borrow_mut(|wakeup| wakeup.push(task_id));
}

#[doc(hidden)]
pub fn poll_all() {
    let mut tasks = WAKEUP.with_borrow_mut(mem::take); // clear the wakeup list
    for task_id in tasks.drain(..) {
        // Temporarily remove the task from the table. We need to execute it while `TASKS` is not borrowed, because it may schedule more tasks.
        let Some(mut task) = TASKS.with_borrow_mut(|tasks| tasks.get_mut(task_id).map(mem::take))
        else {
            // The task is dropped on the first callback that panics, but the last callback is the one that sets the flag.
            // So if multiple calls are sent concurrently, the waker will be asked to wake a future that no longer exists.
            // This should be the only possible case in which this happens.
            crate::trap("Call already trapped");
            // This also should not happen because the CallFuture handles this itself. But FuturesUnordered introduces some chaos.
        };
        let waker = Waker::from(Arc::new(TaskWaker { task_id }));
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
    WAKEUP.with_borrow_mut(|wakeup| *wakeup = tasks); // give the allocation back
}

pub(crate) static CLEANUP: AtomicBool = AtomicBool::new(false);

new_key_type! {
    struct TaskId;
}

thread_local! {
    static TASKS: RefCell<SlotMap<TaskId, Task>> = <_>::default();
    static WAKEUP: RefCell<Vec<TaskId>> = <_>::default();
}

struct Task {
    future: Pin<Box<dyn Future<Output = ()>>>,
}

impl Default for Task {
    fn default() -> Self {
        Self {
            future: Box::pin(async {}),
        }
    }
}

/// Waker implementation for executing futures produced by `call`/`call_raw`/etc.
///
/// *Almost* a do-nothing executor, i.e. wake directly calls poll with no scheduler, except it attempts to clean up tasks
/// whose execution has trapped - see `call::is_recovering_from_trap`.
#[derive(Clone)]
struct TaskWaker {
    task_id: TaskId,
}

impl Wake for TaskWaker {
    fn wake(self: Arc<Self>) {
        if CLEANUP.load(Ordering::Relaxed) {
            // This task is recovering from a trap. We cancel it to run destructors.
            TASKS.with_borrow_mut(|tasks| {
                tasks.remove(self.task_id);
            })
        } else {
            WAKEUP.with_borrow_mut(|wakeup| wakeup.push(self.task_id));
        }
    }
}
