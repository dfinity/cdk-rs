use std::cell::RefCell;
use std::future::Future;
use std::mem;
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, Ordering};
use std::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};

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
    let task_id = TASKS.with(|tasks| tasks.borrow_mut().insert(task));
    let waker = TaskWaker { task_id };
    waker.wake();
}

pub(crate) static CLEANUP: AtomicBool = AtomicBool::new(false);

new_key_type! {
    struct TaskId;
}

thread_local! {
    static TASKS: RefCell<SlotMap<TaskId, Task>> = <_>::default();
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
/// whose execution has trapped - see `call::is_recovering_from_trap``.
#[derive(Clone)]
struct TaskWaker {
    task_id: TaskId,
}

impl TaskWaker {
    fn wake(&self) {
        if CLEANUP.load(Ordering::Relaxed) {
            // This task is recovering from a trap. We cancel it to run destructors.
            TASKS.with(|tasks| {
                tasks.borrow_mut().remove(self.task_id);
            })
        } else {
            // Temporarily remove the task from the table. We need to execute it while `TASKS` is not borrowed, because it may schedule more tasks.
            let Some(mut task) = TASKS.with(|tasks| {
                let Ok(mut tasks) = tasks.try_borrow_mut() else {
                    // If this is already borrowed, then wake was called from inside poll. There's not a lot we can do about this - we are not
                    // a true scheduler and so cannot immediately schedule another poll, nor can we reentrantly lock the future. So we ignore it.
                    // This will be disappointing to types like FuturesUnordered that expected this to work, but since the only source of asynchrony
                    // and thus a guaranteed source of wakeup notifications is the ic0.call_new callback, this shouldn't cause any actual problems.
                    return None;
                };
                tasks.get_mut(self.task_id).map(mem::take)
            }) else { return };
            let waker = self.clone().into_waker();
            let poll = task.future.as_mut().poll(&mut Context::from_waker(&waker));
            match poll {
                Poll::Pending => {
                    // more to do, put the task back in the table
                    TASKS.with(|tasks| {
                        if let Some(t) = tasks.borrow_mut().get_mut(self.task_id) {
                            *t = task;
                        }
                    });
                }
                Poll::Ready(()) => {
                    // task complete, remove its entry from the table fully
                    TASKS.with(|tasks| tasks.borrow_mut().remove(self.task_id));
                }
            }
        }
    }

    fn into_waker(self) -> Waker {
        let raw_waker = raw_waker(self);
        // SAFETY: raw_waker correctly implements RawWakerVTable's contract.
        unsafe { Waker::from_raw(raw_waker) }
    }
}

// Simple waker vtable implementation entirely unrelated to TaskWaker. Would be generic over a SimpleWaker trait
// for unsafe separation's sake if statics could be generic, but they can't and you need a static for the vtable.

static WAKER_VTABLE: RawWakerVTable = RawWakerVTable::new(clone, wake, wake_by_ref, drop);

/// Produces a RawWaker from a pointer representing a Box<TaskWaker>.
fn raw_waker(waker: TaskWaker) -> RawWaker {
    RawWaker::new(
        Box::into_raw(Box::new(waker)).cast::<()>().cast_const(),
        &WAKER_VTABLE,
    )
}

/// # Safety
/// Must only be called from the RawWaker produced by raw_waker.
unsafe fn clone(ptr: *const ()) -> RawWaker {
    // SAFETY: RawWaker's contract says `ptr` is the data-pointer from raw_waker, i.e. Box<TaskWaker>, semantically borrowed immutably.
    let waker = unsafe { ptr.cast::<TaskWaker>().as_ref().unwrap() };
    let cloned = waker.clone();
    raw_waker(cloned)
}

/// # Safety
/// Must only be called from the RawWaker produced by raw_waker.
unsafe fn wake(ptr: *const ()) {
    // SAFETY: RawWaker's contract says `ptr` is the data-pointer from raw_waker, i.e. Box<TaskWaker>, semantically owned.
    let waker = unsafe { Box::from_raw(ptr.cast::<TaskWaker>().cast_mut()) };
    waker.wake();
}

/// # Safety
/// Must only be called from the RawWaker produced by raw_waker.
unsafe fn wake_by_ref(ptr: *const ()) {
    // SAFETY: RawWaker's contract says `ptr` is the data-pointer from raw_waker, i.e. Box<TaskWaker>, semantically borrowed immutably.
    let waker = unsafe { ptr.cast::<TaskWaker>().as_ref().unwrap() };
    waker.wake();
}

/// # Safety
/// Must only be called from the RawWaker produced by raw_waker.
unsafe fn drop(ptr: *const ()) {
    // SAFETY: RawWaker's contract says `ptr` is the data-pointer from raw_waker, i.e. Box<TaskWaker>, semantically owned.
    let _waker = unsafe { Box::from_raw(ptr.cast::<TaskWaker>().cast_mut()) };
}
