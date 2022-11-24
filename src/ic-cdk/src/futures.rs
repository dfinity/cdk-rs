use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::AtomicBool;
use std::task::Context;

/// Must be called on every top-level future corresponding to a method call of a
/// canister by the IC.
///
/// Saves the pointer to the future on the heap and kickstarts the future by
/// polling it once. During the polling we also need to provide the waker
/// callback which is triggered after the future made progress.
/// The waker would then poll the future one last time to advance it to
/// the final state. For that, we pass the future pointer to the waker, so that
/// it can be restored into a box from a raw pointer and then dropped if not
/// needed anymore.
///
/// Technically, we store 2 pointers on the heap: the pointer to the future
/// itself, and a pointer to that pointer. The reason for this is that the waker
/// API requires us to pass one thin pointer, while a a pointer to a `dyn Trait`
/// can only be fat. So we create one additional thin pointer, pointing to the
/// fat pointer and pass it instead.
pub fn spawn<F: 'static + Future<Output = ()>>(future: F) {
    let future_ptr = Box::into_raw(Box::new(future));
    let future_ptr_ptr: *mut *mut dyn Future<Output = ()> = Box::into_raw(Box::new(future_ptr));
    // SAFETY: The pointer is to a Box, which satisfies the pinning requirement.
    let mut pinned_future = unsafe { Pin::new_unchecked(&mut *future_ptr) };
    if pinned_future
        .as_mut()
        // SAFETY: future_ptr_ptr was constructed from a boxed box.
        // future_ptr_ptr is NOT unique (shared by pinned_future). This call is UNSOUND.
        // FIXME
        .poll(&mut Context::from_waker(&unsafe {
            waker::waker(future_ptr_ptr as *const ())
        }))
        .is_ready()
    {
        // SAFETY: These were created from boxes earlier in the function
        unsafe {
            let _ = Box::from_raw(future_ptr);
            let _ = Box::from_raw(future_ptr_ptr);
        }
    }
}

pub(crate) static CLEANUP: AtomicBool = AtomicBool::new(false);

// This module contains the implementation of a waker we're using for waking
// top-level futures (the ones returned by canister methods). The waker polls
// the future once and re-pins it on the heap, if it's pending. If the future is
// done, we do nothing. Hence, it will be unallocated once we exit the scope and
// we're not interested in the result, as it can only be a unit `()` if the
// waker was used as intended.
mod waker {
    use super::*;
    use std::{
        sync::atomic::Ordering,
        task::{RawWaker, RawWakerVTable, Waker},
    };
    type FuturePtr = *mut dyn Future<Output = ()>;

    static MY_VTABLE: RawWakerVTable = RawWakerVTable::new(clone, wake, wake_by_ref, drop);

    /// # Safety
    ///
    /// The pointer must be a unique Box-allocated pointer to a Box-allocated pointer to a `dyn Future<Output=()>`.
    unsafe fn raw_waker(ptr: *const ()) -> RawWaker {
        // SAFETY: All the function pointers in MY_VTABLE correctly operate on the pointer in question.
        RawWaker::new(ptr, &MY_VTABLE)
    }

    /// # Safety
    /// This function should only be called by a [Waker] created by [`waker`].
    unsafe fn clone(ptr: *const ()) -> RawWaker {
        // SAFETY: The pointer attached via `waker` satisfies its own contract.
        unsafe { raw_waker(ptr) }
    }

    // Our waker will be called only if one of the response callbacks is triggered.
    // Then, the waker will restore the future from the pointer we passed into the
    // waker inside the `kickstart` method and poll the future again. If the future
    // is pending, we leave it on the heap. If it's ready, we deallocate the
    // pointer. If CLEANUP is set, then we're recovering from a callback trap, and
    // want to drop the future without executing any more of it.
    /// # Safety
    /// This function should only be called by a [Waker] created by [`waker`].
    unsafe fn wake(ptr: *const ()) {
        // SAFETY: The function contract guarantees that the outer pointer is a Box of the FuturePtr.
        let boxed_future_ptr_ptr = unsafe { Box::from_raw(ptr as *mut FuturePtr) };
        let future_ptr: FuturePtr = *boxed_future_ptr_ptr;
        // SAFETY: The function contract guarantees that the inner pointer is a FuturePtr, and unique.
        let mut boxed_future = unsafe { Box::from_raw(future_ptr) };
        // SAFETY: Boxes satisfy the pinning contract and are sound to use in new_unchecked.
        // The box is never moved out of, but dropped in place.
        let mut pinned_future = unsafe { Pin::new_unchecked(&mut *boxed_future) };
        if !super::CLEANUP.load(Ordering::Relaxed)
            && pinned_future
                .as_mut()
                // SAFETY: The pointer attached via `waker` satisfies its own contract.
                .poll(&mut Context::from_waker(&unsafe { waker::waker(ptr) }))
                .is_pending()
        {
            Box::into_raw(boxed_future_ptr_ptr);
            Box::into_raw(boxed_future);
        }
    }

    fn wake_by_ref(_: *const ()) {}

    fn drop(_: *const ()) {}

    /// # Safety
    ///
    /// The pointer must be a unique Box-allocated pointer to a Box-allocated pointer to a `dyn Future<Output=()>`.
    pub unsafe fn waker(ptr: *const ()) -> Waker {
        // SAFETY:
        // raw_waker has the same safety requirement on ptr as this function
        // The functions in the vtable are passed the ptr that was passed to this function
        // The functions in the vtable uphold RawWaker's contract
        unsafe { Waker::from_raw(raw_waker(ptr)) }
    }
}
