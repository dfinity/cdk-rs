//! This library implements multiple and periodic timers on the Internet Computer.
//!
//! # Example
//!
//! ```rust,no_run
//! # use std::time::Duration;
//! # fn main() {
//! ic_cdk_timers::set_timer(Duration::from_secs(1), async { ic_cdk::println!("Hello from the future!") });
//! # }
//! ```
//!
//! # Details
//!
//! Timers internally use a bounded-wait self-call for error handling purposes. This is not guaranteed to
//! remain the case in the future, but means that if the system is under heavy load, timers may begin to
//! slow down by a lot as the self-calls begin to time out and the timers are rescheduled for the next global
//! timer tick. This also means that each executed timer incurs the cycle cost of a canister call.
//!
//! <div class="warning">
//!
//! Timers are not persisted across canister upgrades.
//!
//! </div>

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
    future::Future,
    mem,
    pin::Pin,
    time::Duration,
};

use ic_cdk_executor::MethodHandle;
use slotmap::{KeyData, SlotMap, new_key_type};

// To ensure that tasks are removable seamlessly, there are two separate concepts here: tasks, for the actual function being called,
// and timers, the scheduled execution of tasks. As this is an implementation detail, this does not affect the exported name TimerId,
// which is more accurately a task ID. (The obvious solution to this, `pub use`, invokes a very silly compiler error.)

thread_local! {
    static TIMER_COUNTER: Cell<u128> = const { Cell::new(0) };
    static TASKS: RefCell<SlotMap<TimerId, Task>> = RefCell::default();
    static TIMERS: RefCell<BinaryHeap<Timer>> = RefCell::default();
    static MOST_RECENT: Cell<Option<u64>> = const { Cell::new(None) };
}

enum Task {
    Repeated {
        func: Box<dyn FnMut() -> Pin<Box<dyn Future<Output = ()>>>>,
        interval: Duration,
    },
    Once(Pin<Box<dyn Future<Output = ()>>>),
    Invalid,
}

new_key_type! {
    /// Type returned by the [`set_timer`] and [`set_timer_interval`] functions. Pass to [`clear_timer`] to remove the timer.
    pub struct TimerId;
}

#[derive(Debug)]
struct Timer {
    task: TimerId,
    time: u64,
    counter: u128,
}

// Timers are sorted first by time, then by insertion order to ensure deterministic ordering.
// The ordering is reversed (earlier timer > later) for use in BinaryHeap which is a max-heap.

impl Ord for Timer {
    fn cmp(&self, other: &Self) -> Ordering {
        self.time
            .cmp(&other.time)
            .then_with(|| self.counter.cmp(&other.counter))
            .reverse()
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

fn next_counter() -> u128 {
    TIMER_COUNTER.with(|c| {
        let v = c.get();
        c.set(v + 1);
        v
    })
}

// This function is called by the IC at or after the timestamp provided to `ic0.global_timer_set`.
#[unsafe(export_name = "canister_global_timer")]
extern "C" fn global_timer() {
    ic_cdk_executor::in_tracking_executor_context(|| {
        let mut canister_self = [0; 32];
        let canister_self = {
            let sz = ic0::canister_self_size();
            ic0::canister_self_copy(&mut canister_self[..sz], 0);
            &canister_self[..sz]
        };
        // All the calls are made concurrently, according only to the timestamp we *started* with.
        // This allows us to use the minimum number of execution rounds, as well as avoid any race conditions.
        // The only thing that can happen interleavedly is canceling a task, which is seamless by design.
        let now = ic0::time();
        TIMERS.with_borrow_mut(|timers| {
            let mut to_reschedule = Vec::new();
            let mut insn_count = 0;
            let mut first = true;
            // pop every timer that should have been completed by `now`, and get ready to run its task if it exists
            loop {
                if first {
                    first = false;
                } else if insn_count == 0 {
                    insn_count = ic0::performance_counter(0);
                } else if insn_count * 3 + ic0::performance_counter(0) > 40_000_000_000 {
                    ic0::debug_print(b"[ic-cdk-timers] canister_global_timer: approaching instruction limit, deferring remaining timers to next round");
                    break;
                }
                if let Some(timer) = timers.peek() {
                    if timer.time <= now {
                        let timer: Timer = timers.pop().unwrap();
                        let timer_scheduled_time = timer.time;
                        if TASKS.with_borrow(|tasks| tasks.contains_key(timer.task)) {
                            // This is the biggest hack in this code. If a callback was called explicitly, and trapped, the rescheduling step wouldn't happen.
                            // The closest thing to a catch_unwind that's available here is performing an inter-canister call to ourselves;
                            // traps will be caught at the call boundary. This invokes a meaningful cycles cost, and should an alternative for catching traps
                            // become available, this code should be rewritten.
                            let task_id = timer.task;
                            let env = Box::new(CallEnv {
                                timer,
                                method_handle: ic_cdk_executor::extend_current_method_context(),
                            });
                            const METHOD_NAME: &str = "<ic-cdk internal> timer_executor";
                            let liquid_cycles = ic0::canister_liquid_cycle_balance128();
                            let cost = ic0::cost_call(METHOD_NAME.len() as u64, 8);
                            // --- no allocations between the liquid cycles check and call_perform
                            if liquid_cycles < cost {
                                ic0::debug_print(
                                    b"[ic-cdk-timers] unable to schedule timer: not enough liquid cycles",
                                );
                                to_reschedule.push(env.timer);
                                break;
                            }
                            let env = Box::<CallEnv>::into_raw(env) as usize;
                            // SAFETY:
                            // - `timer_scope_callback` is intended as an entrypoint and therefore can be called as both
                            //      reply and reject fn for ic0.call_new.
                            // - `timer_scope_cleanup` is intended as an entrypoint and therefore can be called as
                            //      cleanup fn for ic0.call_on_cleanup.
                            // - `state_ptr` is a pointer created via Box::<CallEnv>::into_raw, and can therefore
                            //      be passed as the userdata for `callback` and `cleanup`.
                            // - if-and-only-if ic0.call_perform returns 0, exactly one of `timer_scope_callback` or
                            //      `timer_scope_cleanup` receive ownership of `state_ptr`
                            // - both functions deallocate `state_ptr`, and this enclosing function deallocates
                            //      `state_ptr` if ic0.call_perform returns !=0, and therefore `state_ptr`'s ownership
                            //      can be passed to FFI without leaking memory.
                            unsafe {
                                ic0::call_new(
                                    canister_self,
                                    METHOD_NAME,
                                    timer_scope_callback,
                                    env,
                                    timer_scope_callback,
                                    env,
                                );
                                ic0::call_on_cleanup(timer_scope_cleanup, env);
                            }
                            ic0::call_with_best_effort_response(300);
                            ic0::call_data_append(task_id.0.as_ffi().to_be_bytes().as_ref());
                            let errcode = ic0::call_perform();
                            // ---allocations resumed
                            if errcode != 0 {
                                // SAFETY:
                                // - We just created this from a Box<CallEnv>
                                // - A nonzero error code from call_perform releases ownership back to us
                                let env = unsafe { Box::from_raw(env as *mut CallEnv) };
                                ic0::debug_print(
                                    format!("[ic-cdk-timers] canister_global_timer: call_perform failed with error code {errcode}").as_bytes(),
                                );
                                // If the attempted call failed, we will try to execute the timer again later.
                                to_reschedule.push(env.timer);
                                // This error most likely will recur if any more timers are scheduled this round.
                                break;
                            } else {
                                // If a repeated timer is successfully dispatched (irrespective of the timer's own success), reschedule it.
                                TASKS.with_borrow(|tasks| {
                                    if let Task::Repeated { interval, .. } = &tasks[task_id] {
                                        match timer_scheduled_time.checked_add(interval.as_nanos() as u64) {
                                            Some(time) => {
                                                timers.push(Timer {
                                                    task: task_id,
                                                    time,
                                                    counter: next_counter(),
                                                });
                                            }
                                            None => ic0::debug_print(
                                                format!(
                                                    "Failed to reschedule task (needed {interval}, currently {now}, and this would exceed u64::MAX)",
                                                    interval = interval.as_nanos()
                                                ).as_bytes(),
                                            ),
                                        }
                                    }
                                })
                            }
                        }
                        continue;
                    }
                }
                break;
            }
            timers.extend(to_reschedule);
        });

        MOST_RECENT.set(None);
        update_ic0_timer();
    });
}

struct CallEnv {
    timer: Timer,
    method_handle: MethodHandle,
}

/// # Safety
///
/// This function must only be passed to the IC with a pointer from Box::<CallEnv>::into_raw as userdata.
unsafe extern "C" fn timer_scope_callback(env: usize) {
    // SAFETY: This function is only ever called by the IC, and we only ever pass a Box<CallEnv> as userdata.
    let CallEnv {
        timer,
        method_handle,
    } = *unsafe { Box::<CallEnv>::from_raw(env as *mut CallEnv) };
    ic_cdk_executor::in_callback_executor_context_for(method_handle, || {
        let task_id = timer.task;
        let reject_code = ic0::msg_reject_code();
        match reject_code {
            0 => {} // success
            2 | 6 => {
                // Double check that it exists - in case of SYS_TRANSIENT it may have completed.
                if TASKS.with_borrow(|tasks| tasks.contains_key(task_id)) {
                    // Try to execute the timer again later.
                    TIMERS.with_borrow_mut(|timers| timers.push(timer));
                    update_ic0_timer();
                }
                return;
            }
            x => {
                let reject_data_size = ic0::msg_reject_msg_size();
                let mut reject_data = Vec::with_capacity(reject_data_size);
                ic0::msg_reject_msg_copy_uninit(
                    &mut reject_data.spare_capacity_mut()[..reject_data_size],
                    0,
                );
                // SAFETY: ic0.msg_arg_data_copy fully initializes the vector up to reject_data_size.
                unsafe {
                    reject_data.set_len(reject_data_size);
                }
                ic0::debug_print(
                    format!(
                        "[ic-cdk-timers] timer failed (code {x}): {}",
                        String::from_utf8_lossy(&reject_data)
                    )
                    .as_bytes(),
                )
            }
        }
        TASKS.with_borrow_mut(|tasks| {
            if let Some(task) = tasks.get(task_id) {
                match task {
                    // duplicated on purpose - it must be removed in the function call, to access self by value;
                    // and it must be removed here, because it may have trapped and not actually been removed.
                    // Luckily slotmap ops are equivalent to simple vector indexing.
                    Task::Once(_) => {
                        tasks.remove(task_id);
                    }
                    Task::Repeated { .. } => {}
                    Task::Invalid => {
                        unreachable!(
                            "[ic-cdk-timers] internal error: invalid task state in global timer await callback"
                        )
                    }
                }
            }
        });
    });
}

/// # Safety
///
/// This function must only be passed to the IC with a pointer from Box::<CallEnv>::into_raw as userdata.
unsafe extern "C" fn timer_scope_cleanup(env: usize) {
    // SAFETY: This function is only ever called by the IC, and we only ever pass a Box<CallEnv> as userdata.
    unsafe {
        drop(Box::from_raw(env as *mut CallEnv));
    }
    ic0::debug_print(b"[ic-cdk-timers] internal error: trap in scope callback");
}

/// Sets `future` to be executed later, after `delay`. Panics if `delay` + [`time()`] is more than [`u64::MAX`] nanoseconds.
///
/// To cancel the timer before it executes, pass the returned `TimerId` to [`clear_timer`].
///
/// <div class="warning">
///
/// Timers are not persisted across canister upgrades.
///
/// </div>
///
/// # Examples
///
/// ```no_run
/// # use std::time::Duration;
/// ic_cdk_timers::set_timer(Duration::from_secs(1), async {
///     ic_cdk::println!("Hello from the future!");
/// });
/// ```
///
/// [`time()`]: https://docs.rs/ic-cdk/0.18.5/ic_cdk/api/fn.time.html
pub fn set_timer(delay: Duration, future: impl Future<Output = ()> + 'static) -> TimerId {
    let delay_ns = u64::try_from(delay.as_nanos()).expect(
        "delay out of bounds (must be within `u64::MAX - ic_cdk::api::time()` nanoseconds)",
    );
    let scheduled_time = ic0::time().checked_add(delay_ns).expect(
        "delay out of bounds (must be within `u64::MAX - ic_cdk::api::time()` nanoseconds)",
    );
    let key = TASKS.with_borrow_mut(|tasks| tasks.insert(Task::Once(Box::pin(future))));
    TIMERS.with_borrow_mut(|timers| {
        timers.push(Timer {
            task: key,
            time: scheduled_time,
            counter: next_counter(),
        })
    });
    update_ic0_timer();
    key
}

/// Sets `func` to be executed every `interval`. Panics if `interval` + [`time()`] is more than [`u64::MAX`] nanoseconds.
///
/// To cancel the interval timer, pass the returned `TimerId` to [`clear_timer`].
///
/// This is a closure returning a future (`|| async {`), not an async closure (`async || {`). The two syntaxes
/// are interchangeable *if* the closure does not capture anything. If it does, you will need the former syntax,
/// and it is almost certain that either your captures must be Copy or you must use e.g. Rc to share them, because
/// you cannot capture by reference, and referencing an owned capture in a returned async block is not possible.
///
/// <div class="warning">
///
/// Interval timers should be *idempotent* with respect to the canister's state, as during heavy network load,
/// timeouts may result in duplicate execution.
///
/// </div>
///
/// <div class="warning">
///
/// Timers are not persisted across canister upgrades.
///
/// </div>
///
/// # Examples
///
/// ```no_run
/// # use std::time::Duration;
/// ic_cdk_timers::set_timer_interval(Duration::from_secs(5), || async {
///     ic_cdk::println!("This will run every five seconds forever!");
/// });
/// ```
///
/// [`time()`]: https://docs.rs/ic-cdk/0.18.5/ic_cdk/api/fn.time.html
pub fn set_timer_interval<Fut>(interval: Duration, func: impl FnMut() -> Fut + 'static) -> TimerId
where
    Fut: Future<Output = ()> + 'static,
{
    let mut func = func;
    let interval_ns = u64::try_from(interval.as_nanos()).expect(
        "delay out of bounds (must be within `u64::MAX - ic_cdk::api::time()` nanoseconds)",
    );
    let scheduled_time = ic0::time().checked_add(interval_ns).expect(
        "delay out of bounds (must be within `u64::MAX - ic_cdk::api::time()` nanoseconds)",
    );
    let key = TASKS.with_borrow_mut(|tasks| {
        tasks.insert(Task::Repeated {
            func: Box::new(move || Box::pin(func())),
            interval,
        })
    });
    TIMERS.with_borrow_mut(|timers| {
        timers.push(Timer {
            task: key,
            time: scheduled_time,
            counter: next_counter(),
        });
    });
    update_ic0_timer();
    key
}

/// Cancels an existing timer. Does nothing if the timer has already been canceled.
///
/// # Examples
///
/// ```no_run
/// # use std::time::Duration;
/// let timer_id = ic_cdk_timers::set_timer(Duration::from_secs(60), async {
///     ic_cdk::println!("This will never run, because we cancel it!");
/// });
/// ic_cdk_timers::clear_timer(timer_id);
/// ```
pub fn clear_timer(id: TimerId) {
    TASKS.with_borrow_mut(|tasks| tasks.remove(id));
}

/// Calls `ic0.global_timer_set` with the soonest timer in [`TIMERS`]. This is needed after inserting a timer, and after executing one.
fn update_ic0_timer() {
    TIMERS.with_borrow(|timers| {
        let soonest_timer = timers.peek().map(|timer| timer.time);
        let should_change = match (soonest_timer, MOST_RECENT.get()) {
            (Some(timer), Some(recent)) => timer < recent,
            (Some(_), None) => true,
            _ => false,
        };
        if should_change {
            ic0::global_timer_set(soonest_timer.unwrap());
            MOST_RECENT.set(soonest_timer);
        }
    });
}

#[cfg_attr(
    target_family = "wasm",
    unsafe(export_name = "canister_update <ic-cdk internal> timer_executor")
)]
#[cfg_attr(
    not(target_family = "wasm"),
    unsafe(export_name = "canister_update_ic_cdk_internal.timer_executor")
)]
extern "C" fn timer_executor() {
    ic_cdk_executor::in_tracking_executor_context(|| {
        let mut caller = [0; 32];
        let caller = {
            let sz = ic0::msg_caller_size();
            ic0::msg_caller_copy(&mut caller[..sz], 0);
            &caller[..sz]
        };
        let mut canister_self = [0; 32];
        let canister_self = {
            let sz = ic0::canister_self_size();
            ic0::canister_self_copy(&mut canister_self[..sz], 0);
            &canister_self[..sz]
        };

        if caller != canister_self {
            ic0::trap(b"This function is internal to ic-cdk and should not be called externally.");
        }

        // timer_executor is only called by the canister itself (from global_timer),
        // so we can safely assume that the argument is a valid TimerId (u64).
        // And we don't need decode_one_with_config/DecoderConfig to defense against malicious payload.
        assert!(ic0::msg_arg_data_size() == 8);
        let mut arg_bytes = [0; 8];
        ic0::msg_arg_data_copy(&mut arg_bytes, 0);
        let task_id = u64::from_be_bytes(arg_bytes);
        let task_id = TimerId(KeyData::from_ffi(task_id));

        // We can't be holding `TASKS` when we call the function, because it may want to schedule more tasks.
        // Instead, we swap the task out in order to call it, and then either swap it back in, or remove it.
        let task = TASKS.with_borrow_mut(|tasks| {
            if let Some(task) = tasks.get_mut(task_id) {
                // Replace with Invalid to take ownership. The Invalid variant should not last past the end of this function.
                Some(mem::replace(task, Task::Invalid))
            } else {
                None
            }
        });
        if let Some(task) = task {
            match task {
                Task::Once(fut) => {
                    ic_cdk_executor::spawn_protected(async {
                        fut.await;
                        ic0::msg_reply();
                    });
                    // Invalid cleared in the same round
                    TASKS.with_borrow_mut(|tasks| tasks.remove(task_id));
                }
                Task::Repeated { mut func, interval } => {
                    let invocation = func();
                    // Invalid cleared in the same round
                    TASKS.with_borrow_mut(|tasks| {
                        tasks[task_id] = Task::Repeated { func, interval }
                    });
                    ic_cdk_executor::spawn_protected(async move {
                        invocation.await;
                        ic0::msg_reply();
                    });
                }
                Task::Invalid => {
                    // Invalid impossible
                    unreachable!(
                        "[ic-cdk-timers] internal error: invalid task state in executor method"
                    )
                }
            }
        } else {
            ic0::msg_reply();
        }
    });
}
