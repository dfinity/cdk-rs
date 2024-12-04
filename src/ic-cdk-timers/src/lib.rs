//! The library implements multiple and periodic timers on the Internet Computer.
//!
//! # Example
//!
//! ```rust,no_run
//! # use std::time::Duration;
//! # fn main() {
//! ic_cdk_timers::set_timer(Duration::from_secs(1), || ic_cdk::println!("Hello from the future!"));
//! # }
//! ```

#![warn(
    elided_lifetimes_in_paths,
    missing_debug_implementations,
    missing_docs,
    unsafe_op_in_unsafe_fn,
    clippy::undocumented_unsafe_blocks,
    clippy::missing_safety_doc
)]

use std::{
    cell::{Cell, RefCell},
    cmp::Ordering,
    collections::BinaryHeap,
    mem,
    time::Duration,
};

use futures::{stream::FuturesUnordered, StreamExt};
use slotmap::{new_key_type, KeyData, SlotMap};

use ic_cdk::api::call::RejectionCode;

// To ensure that tasks are removable seamlessly, there are two separate concepts here: tasks, for the actual function being called,
// and timers, the scheduled execution of tasks. As this is an implementation detail, this does not affect the exported name TimerId,
// which is more accurately a task ID. (The obvious solution to this, `pub use`, invokes a very silly compiler error.)

thread_local! {
    static TASKS: RefCell<SlotMap<TimerId, Task>> = RefCell::default();
    static TIMERS: RefCell<BinaryHeap<Timer>> = RefCell::default();
    static MOST_RECENT: Cell<Option<u64>> = const { Cell::new(None) };
}

enum Task {
    Repeated {
        func: Box<dyn FnMut()>,
        interval: Duration,
    },
    Once(Box<dyn FnOnce()>),
}

impl Default for Task {
    fn default() -> Self {
        Self::Once(Box::new(|| ()))
    }
}

new_key_type! {
    /// Type returned by the [`set_timer`] and [`set_timer_interval`] functions. Pass to [`clear_timer`] to remove the timer.
    pub struct TimerId;
}

struct Timer {
    task: TimerId,
    time: u64,
}

// Timers are sorted such that x > y if x should be executed _before_ y.

impl Ord for Timer {
    fn cmp(&self, other: &Self) -> Ordering {
        self.time.cmp(&other.time).reverse()
    }
}

impl PartialOrd for Timer {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Timer {
    fn eq(&self, other: &Self) -> bool {
        self.time == other.time
    }
}

impl Eq for Timer {}

// This function is called by the IC at or after the timestamp provided to `ic0.global_timer_set`.
#[export_name = "canister_global_timer"]
extern "C" fn global_timer() {
    ic_cdk::setup();
    ic_cdk::spawn(async {
        // All the calls are made first, according only to the timestamp we *started* with, and then all the results are awaited.
        // This allows us to use the minimum number of execution rounds, as well as avoid any race conditions.
        // The only thing that can happen interleavedly is canceling a task, which is seamless by design.
        let mut call_futures = FuturesUnordered::new();
        let now = ic_cdk::api::time();
        TIMERS.with(|timers| {
            // pop every timer that should have been completed by `now`, and get ready to run its task if it exists
            loop {
                let mut timers = timers.borrow_mut();
                if let Some(timer) = timers.peek() {
                    if timer.time <= now {
                        let timer = timers.pop().unwrap();
                        if TASKS.with(|tasks| tasks.borrow().contains_key(timer.task)) {
                            // This is the biggest hack in this code. If a callback was called explicitly, and trapped, the rescheduling step wouldn't happen.
                            // The closest thing to a catch_unwind that's available here is performing an inter-canister call to ourselves;
                            // traps will be caught at the call boundary. This invokes a meaningful cycles cost, and should an alternative for catching traps
                            // become available, this code should be rewritten.
                            let task_id = timer.task;
                            call_futures.push(async move {
                                (
                                    timer,
                                    ic_cdk::call(
                                        ic_cdk::api::id(),
                                        "<ic-cdk internal> timer_executor",
                                        (task_id.0.as_ffi(),),
                                    )
                                    .await,
                                )
                            });
                        }
                        continue;
                    }
                }
                break;
            }
        });
        // run all the collected tasks, and clean up after them if necessary
        while let Some((timer, res)) = call_futures.next().await {
            let task_id = timer.task;
            match res {
                Ok(()) => {}
                Err((code, msg)) => {
                    ic_cdk::println!("in canister_global_timer: {code:?}: {msg}");
                    match code {
                        RejectionCode::SysTransient => {
                            // Try to execute the timer again later.
                            TIMERS.with(|timers| {
                                timers.borrow_mut().push(timer);
                            });
                            continue;
                        }
                        RejectionCode::NoError
                        | RejectionCode::SysFatal
                        | RejectionCode::DestinationInvalid
                        | RejectionCode::CanisterReject
                        | RejectionCode::CanisterError
                        | RejectionCode::Unknown => {}
                    }
                }
            }
            TASKS.with(|tasks| {
                let mut tasks = tasks.borrow_mut();
                if let Some(task) = tasks.get(task_id) {
                    match task {
                        // duplicated on purpose - it must be removed in the function call, to access self by value;
                        // and it must be removed here, because it may have trapped and not actually been removed.
                        // Luckily slotmap ops are equivalent to simple vector indexing.
                        Task::Once(_) => {
                            tasks.remove(task_id);
                        }
                        // reschedule any repeating tasks
                        Task::Repeated { interval, .. } => {
                            match now.checked_add(interval.as_nanos() as u64) {
                                Some(time) => TIMERS.with(|timers| {
                                    timers.borrow_mut().push(Timer {
                                        task: task_id,
                                        time,
                                    })
                                }),
                                None => ic_cdk::println!(
                                    "Failed to reschedule task (needed {interval}, currently {now}, and this would exceed u64::MAX)",
                                    interval = interval.as_nanos(),
                                ),
                            }
                        }
                    }
                }
            });
        }
        MOST_RECENT.with(|recent| recent.set(None));
        update_ic0_timer();
    });
}

/// Sets `func` to be executed later, after `delay`. Panics if `delay` + [`time()`][ic_cdk::api::time] is more than [`u64::MAX`] nanoseconds.
///
/// To cancel the timer before it executes, pass the returned `TimerId` to [`clear_timer`].
///
/// Note that timers are not persisted across canister upgrades.
pub fn set_timer(delay: Duration, func: impl FnOnce() + 'static) -> TimerId {
    let delay_ns = u64::try_from(delay.as_nanos()).expect(
        "delay out of bounds (must be within `u64::MAX - ic_cdk::api::time()` nanoseconds)",
    );
    let scheduled_time = ic_cdk::api::time().checked_add(delay_ns).expect(
        "delay out of bounds (must be within `u64::MAX - ic_cdk::api::time()` nanoseconds)",
    );
    let key = TASKS.with(|tasks| tasks.borrow_mut().insert(Task::Once(Box::new(func))));
    TIMERS.with(|timers| {
        timers.borrow_mut().push(Timer {
            task: key,
            time: scheduled_time,
        });
    });
    update_ic0_timer();
    key
}

/// Sets `func` to be executed every `interval`. Panics if `interval` + [`time()`][ic_cdk::api::time] is more than [`u64::MAX`] nanoseconds.
///
/// To cancel the interval timer, pass the returned `TimerId` to [`clear_timer`].
///
/// Note that timers are not persisted across canister upgrades.
pub fn set_timer_interval(interval: Duration, func: impl FnMut() + 'static) -> TimerId {
    let interval_ns = u64::try_from(interval.as_nanos()).expect(
        "delay out of bounds (must be within `u64::MAX - ic_cdk::api::time()` nanoseconds)",
    );
    let scheduled_time = ic_cdk::api::time().checked_add(interval_ns).expect(
        "delay out of bounds (must be within `u64::MAX - ic_cdk::api::time()` nanoseconds)",
    );
    let key = TASKS.with(|tasks| {
        tasks.borrow_mut().insert(Task::Repeated {
            func: Box::new(func),
            interval,
        })
    });
    TIMERS.with(|timers| {
        timers.borrow_mut().push(Timer {
            task: key,
            time: scheduled_time,
        })
    });
    update_ic0_timer();
    key
}

/// Cancels an existing timer. Does nothing if the timer has already been canceled.
pub fn clear_timer(id: TimerId) {
    TASKS.with(|tasks| tasks.borrow_mut().remove(id));
}

/// Calls `ic0.global_timer_set` with the soonest timer in [`TIMERS`]. This is needed after inserting a timer, and after executing one.
fn update_ic0_timer() {
    TIMERS.with(|timers| {
        let timers = timers.borrow();
        let soonest_timer = timers.peek().map(|timer| timer.time);
        let should_change = match (soonest_timer, MOST_RECENT.with(|recent| recent.get())) {
            (Some(timer), Some(recent)) => timer < recent,
            (Some(_), None) => true,
            _ => false,
        };
        if should_change {
            // SAFETY: ic0::global_timer_set is always a safe call
            unsafe { ic0::global_timer_set(soonest_timer.unwrap() as i64) };
            MOST_RECENT.with(|recent| recent.set(soonest_timer));
        }
    });
}

#[cfg_attr(
    target_family = "wasm",
    export_name = "canister_update <ic-cdk internal> timer_executor"
)]
#[cfg_attr(
    not(target_family = "wasm"),
    export_name = "canister_update_ic_cdk_internal.timer_executor"
)]
extern "C" fn timer_executor() {
    if ic_cdk::api::caller() != ic_cdk::api::id() {
        ic_cdk::trap("This function is internal to ic-cdk and should not be called externally.");
    }
    let config = ic_cdk::api::call::ArgDecoderConfig {
        decoding_quota: Some(10_000),
        skipping_quota: Some(100),
        debug: false,
    };
    let (task_id,) = ic_cdk::api::call::arg_data(config);
    let task_id = TimerId(KeyData::from_ffi(task_id));
    // We can't be holding `TASKS` when we call the function, because it may want to schedule more tasks.
    // Instead, we swap the task out in order to call it, and then either swap it back in, or remove it.
    let task = TASKS.with(|tasks| {
        let mut tasks = tasks.borrow_mut();
        tasks.get_mut(task_id).map(mem::take)
    });
    if let Some(mut task) = task {
        match task {
            Task::Once(func) => {
                func();
                TASKS.with(|tasks| tasks.borrow_mut().remove(task_id));
            }
            Task::Repeated { ref mut func, .. } => {
                func();
                TASKS.with(|tasks| tasks.borrow_mut().get_mut(task_id).map(|slot| *slot = task));
            }
        }
    }
    ic_cdk::api::call::reply(());
}
