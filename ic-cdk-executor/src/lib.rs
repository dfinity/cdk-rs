//! An async executor for [`ic-cdk`](https://docs.rs/ic-cdk). Most users should not use this crate directly.

use std::cell::{Cell, RefCell};
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;
use std::sync::atomic::AtomicBool;
use std::task::Context;

use self::waker::WakerState;

/// Must be called on every top-level future corresponding to a method call of a
/// canister by the IC, other than async functions marked `#[update]` or similar.
#[cfg_attr(not(target_arch = "wasm32"), allow(unused_variables, unreachable_code))]
pub fn spawn<F: 'static + Future<Output = ()>>(future: F) {
    #[cfg(not(target_arch = "wasm32"))]
    panic!("Cannot be run outside of wasm!"); // really, just cannot be run in a multi-threaded environment
    let pinned_future = Box::pin(future);
    let waker_state = Rc::new(WakerState {
        future: RefCell::new(pinned_future),
        previous_trap: Cell::new(false),
    });
    let waker = waker::waker(Rc::clone(&waker_state));
    let _ = waker_state
        .future
        .borrow_mut()
        .as_mut()
        .poll(&mut Context::from_waker(&waker));
}

/// In a cleanup callback, this is set to `true` before calling `wake`, and `false` afterwards.
/// This ensures that `wake` will not actually run the future, but instead cancel it and run its destructor.
pub static CLEANUP: AtomicBool = AtomicBool::new(false);

// This module contains the implementation of a waker we're using for waking
// top-level futures (the ones returned by canister methods). Rc handles the
// heap management for us. Hence, it will be deallocated once we exit the scope and
// we're not interested in the result, as it can only be a unit `()` if the
// waker was used as intended.
// Sizable unsafe code is mandatory here; Future::poll cannot be executed without implementing
// RawWaker in terms of raw pointers.
mod waker {
    use super::*;
    use std::{
        rc::Rc,
        sync::atomic::Ordering,
        task::{RawWaker, RawWakerVTable, Waker},
    };

    // The fields have separate RefCells in order to be modified separately.
    pub(crate) struct WakerState {
        pub future: RefCell<Pin<Box<dyn Future<Output = ()>>>>,
        pub previous_trap: Cell<bool>,
    }

    static MY_VTABLE: RawWakerVTable = RawWakerVTable::new(clone, wake, wake_by_ref, drop);

    /// # Safety
    ///
    /// The pointer must be an owning (i.e. represented in the refcount), Rc-allocated pointer to a `WakerState`.
    unsafe fn raw_waker(ptr: *const ()) -> RawWaker {
        // SAFETY: All the function pointers in MY_VTABLE correctly operate on the pointer in question.
        RawWaker::new(ptr, &MY_VTABLE)
    }

    /// # Safety
    ///
    /// This function should only be called by a [Waker] created by [`waker`].
    unsafe fn clone(ptr: *const ()) -> RawWaker {
        // SAFETY: The function's contract guarantees that this pointer is an Rc to a WakerState, and borrows the data from ptr.
        unsafe {
            Rc::increment_strong_count(ptr);
            raw_waker(ptr)
        }
    }

    // Our waker will be called if one of the response callbacks is triggered.
    // Then, the waker will restore the future from the pointer we passed into the
    // waker inside the `spawn` function and poll the future again. Rc takes care of
    // the heap management for us. If CLEANUP is set, then we're recovering from
    // a callback trap, and want to drop the future without executing any more of it;
    // if previous_trap is set, then we already recovered from a callback trap in a
    // different callback, and should immediately trap again in this one.
    //
    /// # Safety
    ///
    /// This function should only be called by a [Waker] created by [`waker`].
    unsafe fn wake(ptr: *const ()) {
        // SAFETY: The function's contract guarantees that the pointer is an Rc to a WakerState, and that this call takes ownership of the data.
        let state = unsafe { Rc::from_raw(ptr as *const WakerState) };
        // Must check CLEANUP *before* previous_trap, as we may be recovering from the following immediate trap.
        if super::CLEANUP.load(Ordering::Relaxed) {
            state.previous_trap.set(true);
        } else if state.previous_trap.get() {
            panic!("Call already trapped");
        } else {
            let waker = waker(Rc::clone(&state));
            let Ok(mut borrow) = state.future.try_borrow_mut() else {
                // If this is already borrowed, then wake was called from inside poll. There's not a lot we can do about this - we are not
                // a true scheduler and so cannot immediately schedule another poll, nor can we reentrantly lock the future. So we ignore it.
                // This will be disappointing to types like FuturesUnordered that expected this to work, but since the only source of asynchrony
                // and thus a guaranteed source of wakeup notifications is the ic0.call_new callback, this shouldn't cause any actual problems.
                return;
            };
            let pinned_future = borrow.as_mut();
            let _ = pinned_future.poll(&mut Context::from_waker(&waker));
        }
    }

    /// # Safety
    ///
    /// This function should only be called by a [Waker] created by [waker].
    unsafe fn wake_by_ref(ptr: *const ()) {
        // SAFETY:
        // The function's contract guarantees that the pointer is an Rc to a WakerState, and that this call borrows the data.
        // wake has the same contract, except it takes ownership instead of borrowing. Which just requires incrementing the refcount.
        unsafe {
            Rc::increment_strong_count(ptr as *const WakerState);
            wake(ptr);
        }
    }

    /// # Safety
    ///
    /// This function should only be called by a [Waker] created by [waker].
    unsafe fn drop(ptr: *const ()) {
        // SAFETY: The function contract guarantees that the pointer is an Rc to a WakerState, and that this call takes ownership of the data.
        unsafe {
            Rc::from_raw(ptr as *const WakerState);
        }
    }

    /// Creates a new Waker.
    pub(crate) fn waker(state: Rc<WakerState>) -> Waker {
        let ptr = Rc::into_raw(state);
        // SAFETY:
        // The pointer is an owning, Rc-allocated pointer to a WakerState, and therefore can be passed to raw_waker
        // The functions in the vtable are passed said ptr
        // The functions in the vtable uphold RawWaker's contract
        unsafe { Waker::from_raw(raw_waker(ptr as *const ())) }
    }
}
