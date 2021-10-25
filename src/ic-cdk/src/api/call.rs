//! APIs to make and manage calls in the canister.
use crate::api::{ic0, trap};
use crate::export::Principal;
use candid::utils::{ArgumentDecoder, ArgumentEncoder};
use candid::{decode_args, encode_args, write_args};
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll, Waker};

#[cfg(target_arch = "wasm32-unknown-unknown")]
#[allow(dead_code)]
mod rc {
    use std::cell::{RefCell, RefMut};
    use std::future::Future;
    use std::pin::Pin;
    use std::rc::Rc;
    use std::task::{Context, Poll};

    pub(crate) type InnerCell<T> = RefCell<T>;

    /// A reference counted cell. This is a specific implementation that is
    /// both Send and Sync, but does not rely on Mutex and Arc in WASM as
    /// the actual implementation of Mutex can break in async flows.
    pub(crate) struct WasmCell<T>(Rc<InnerCell<T>>);

    /// In order to be able to have an async method that returns the
    /// result of a call to another canister, we need that result to
    /// be Send + Sync, but Rc and RefCell are not.
    ///
    /// Since inside a canister there isn't actual concurrent access to
    /// the referenced cell or the reference counted container, it is
    /// safe to force these to be Send/Sync.
    unsafe impl<T> Send for WasmCell<T> {}
    unsafe impl<T> Sync for WasmCell<T> {}

    impl<T> WasmCell<T> {
        pub fn new(val: T) -> Self {
            WasmCell(Rc::new(InnerCell::new(val)))
        }
        pub fn into_raw(self) -> *const InnerCell<T> {
            Rc::into_raw(self.0)
        }
        #[allow(clippy::missing_safety_doc)]
        pub unsafe fn from_raw(ptr: *const InnerCell<T>) -> Self {
            Self(Rc::from_raw(ptr))
        }
        pub fn borrow_mut(&self) -> RefMut<'_, T> {
            self.0.borrow_mut()
        }
        pub fn as_ptr(&self) -> *const InnerCell<T> {
            self.0.as_ptr() as *const _
        }
    }

    impl<O, T: Future<Output = O>> Future for WasmCell<T> {
        type Output = O;

        #[allow(unused_mut)]
        fn poll(mut self: Pin<&mut Self>, ctx: &mut Context<'_>) -> Poll<Self::Output> {
            unsafe { Pin::new_unchecked(&mut *self.0.borrow_mut()) }.poll(ctx)
        }
    }

    impl<T> Clone for WasmCell<T> {
        fn clone(&self) -> Self {
            WasmCell(Rc::clone(&self.0))
        }
    }
}

#[cfg(not(target_arch = "wasm32-unknown-unknown"))]
#[allow(dead_code)]
mod rc {
    use std::future::Future;
    use std::pin::Pin;
    use std::sync::{Arc, Mutex, MutexGuard};
    use std::task::{Context, Poll};

    pub(crate) type InnerCell<T> = Mutex<T>;

    /// A reference counted cell. This is a specific implementation that is
    /// both Send and Sync, but does not rely on Mutex and Arc in WASM as
    /// the actual implementation of Mutex can break in async flows.
    ///
    /// The RefCell is for
    pub(crate) struct WasmCell<T>(Arc<InnerCell<T>>);

    impl<T> WasmCell<T> {
        pub fn new(val: T) -> Self {
            WasmCell(Arc::new(InnerCell::new(val)))
        }
        pub fn into_raw(self) -> *const InnerCell<T> {
            Arc::into_raw(self.0)
        }
        #[allow(clippy::missing_safety_doc)]
        pub unsafe fn from_raw(ptr: *const InnerCell<T>) -> Self {
            Self(Arc::from_raw(ptr))
        }
        pub fn borrow_mut(&self) -> MutexGuard<'_, T> {
            self.0.lock().unwrap()
        }
        pub fn as_ptr(&self) -> *const InnerCell<T> {
            Arc::<_>::as_ptr(&self.0)
        }
    }

    impl<O, T: Future<Output = O>> Future for WasmCell<T> {
        type Output = O;

        #[allow(unused_mut)]
        fn poll(mut self: Pin<&mut Self>, ctx: &mut Context<'_>) -> Poll<Self::Output> {
            unsafe { Pin::new_unchecked(&mut *self.0.lock().unwrap()) }.poll(ctx)
        }
    }

    impl<T> Clone for WasmCell<T> {
        fn clone(&self) -> Self {
            WasmCell(Arc::clone(&self.0))
        }
    }
}

use rc::{InnerCell, WasmCell};

/// Rejection code from calling another canister.
///
/// These can be obtained either using `reject_code()` or `reject_result()`.
#[allow(missing_docs)]
#[repr(i32)]
#[derive(Debug)]
pub enum RejectionCode {
    NoError = 0,

    SysFatal = 1,
    SysTransient = 2,
    DestinationInvalid = 3,
    CanisterReject = 4,
    CanisterError = 5,

    Unknown,
}

impl From<i32> for RejectionCode {
    fn from(code: i32) -> Self {
        match code {
            0 => RejectionCode::NoError,
            1 => RejectionCode::SysFatal,
            2 => RejectionCode::SysTransient,
            3 => RejectionCode::DestinationInvalid,
            4 => RejectionCode::CanisterReject,
            5 => RejectionCode::CanisterError,
            _ => RejectionCode::Unknown,
        }
    }
}

impl From<u32> for RejectionCode {
    fn from(code: u32) -> Self {
        RejectionCode::from(code as i32)
    }
}

/// The result of a Call.
///
/// Errors on the IC have two components; a Code and a message associated with it.
pub type CallResult<R> = Result<R, (RejectionCode, String)>;

// Internal state for the Future when sending a call.
struct CallFutureState<R: serde::de::DeserializeOwned> {
    result: Option<CallResult<R>>,
    waker: Option<Waker>,
}

struct CallFuture<R: serde::de::DeserializeOwned> {
    // We basically use Rc instead of Arc (since we're single threaded), and use
    // RefCell instead of Mutex (because we cannot lock in WASM).
    state: rc::WasmCell<CallFutureState<R>>,
}

impl<R: serde::de::DeserializeOwned> Future for CallFuture<R> {
    type Output = Result<R, (RejectionCode, String)>;

    fn poll(self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Self::Output> {
        let self_ref = Pin::into_ref(self);

        let mut state = self_ref.state.borrow_mut();

        if let Some(result) = state.result.take() {
            Poll::Ready(result)
        } else {
            state.waker = Some(context.waker().clone());
            Poll::Pending
        }
    }
}

/// The callback from IC dereferences the future from a raw pointer, assigns the
/// result and calls the waker. We cannot use a closure here because we pass raw
/// pointers to the System and back.
fn callback(state_ptr: *const InnerCell<CallFutureState<Vec<u8>>>) {
    let state = unsafe { WasmCell::from_raw(state_ptr) };
    // Make sure to un-borrow_mut the state.
    {
        state.borrow_mut().result = Some(match reject_code() {
            RejectionCode::NoError => unsafe { Ok(arg_data_raw()) },
            n => Err((n, reject_message())),
        });
    }
    let w = state.borrow_mut().waker.take();
    if let Some(waker) = w {
        // This is all to protect this little guy here which will call the poll() which
        // borrow_mut() the state as well. So we need to be careful to not double-borrow_mut.
        waker.wake()
    }
}

/// Similar to `call`, but without serialization.
pub fn call_raw(
    id: Principal,
    method: &str,
    args_raw: Vec<u8>,
    payment: u64,
) -> impl Future<Output = CallResult<Vec<u8>>> {
    let callee = id.as_slice();
    let state = WasmCell::new(CallFutureState {
        result: None,
        waker: None,
    });
    let state_ptr = WasmCell::into_raw(state.clone());
    let err_code = unsafe {
        ic0::call_new(
            callee.as_ptr() as i32,
            callee.len() as i32,
            method.as_ptr() as i32,
            method.len() as i32,
            callback as usize as i32,
            state_ptr as i32,
            callback as usize as i32,
            state_ptr as i32,
        );

        ic0::call_data_append(args_raw.as_ptr() as i32, args_raw.len() as i32);
        if payment > 0 {
            ic0::call_cycles_add(payment as i64);
        }
        ic0::call_perform()
    };

    // 0 is a special error code meaning call_simple call succeeded.
    if err_code != 0 {
        let mut state = state.borrow_mut();
        state.result = Some(Err((
            RejectionCode::from(err_code),
            "Couldn't send message".to_string(),
        )));
    }
    CallFuture { state }
}

/// Performs an asynchronous call to another canister via ic0.
pub async fn call<T: ArgumentEncoder, R: for<'a> ArgumentDecoder<'a>>(
    id: Principal,
    method: &str,
    args: T,
) -> CallResult<R> {
    let args_raw = encode_args(args).expect("Failed to encode arguments.");
    let bytes = call_raw(id, method, args_raw, 0).await?;
    decode_args(&bytes).map_err(|err| trap(&format!("{:?}", err)))
}

/// Performs an asynchronous call to another canister and pay cycles at the same time
pub async fn call_with_payment<T: ArgumentEncoder, R: for<'a> ArgumentDecoder<'a>>(
    id: Principal,
    method: &str,
    args: T,
    cycles: u64,
) -> CallResult<R> {
    let args_raw = encode_args(args).expect("Failed to encode arguments.");
    let bytes = call_raw(id, method, args_raw, cycles).await?;
    decode_args(&bytes).map_err(|err| trap(&format!("{:?}", err)))
}

/// Returns a result that maps over the call
///
/// It will be Ok(T) if the call succeeded (with T being the arg_data),
/// and [reject_message()] if it failed.
pub fn result<T: for<'a> ArgumentDecoder<'a>>() -> Result<T, String> {
    match reject_code() {
        RejectionCode::NoError => decode_args(&unsafe { arg_data_raw() })
            .map_err(|e| format!("Failed to decode arguments: {}", e)),
        _ => Err(reject_message()),
    }
}

/// Returns the rejection code for the call.
pub fn reject_code() -> RejectionCode {
    let code = unsafe { ic0::msg_reject_code() };
    RejectionCode::from(code)
}

/// Returns the rejection message.
pub fn reject_message() -> String {
    let len: u32 = unsafe { ic0::msg_reject_msg_size() as u32 };
    let mut bytes = vec![0; len as usize];
    unsafe {
        ic0::msg_reject_msg_copy(bytes.as_mut_ptr() as i32, 0, len as i32);
    }
    String::from_utf8_lossy(&bytes).to_string()
}

/// Rejects the current call with the message.
pub fn reject(message: &str) {
    let err_message = message.as_bytes();
    unsafe {
        ic0::msg_reject(err_message.as_ptr() as i32, err_message.len() as i32);
    }
}

/// An io::Writer for message replies.
pub struct CallReplyWriter;

impl std::io::Write for CallReplyWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        unsafe {
            ic0::msg_reply_data_append(buf.as_ptr() as i32, buf.len() as i32);
        }
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

/// Replies to the current call with a candid argument.
pub fn reply<T: ArgumentEncoder>(reply: T) {
    write_args(&mut CallReplyWriter, reply).expect("Could not encode reply.");
    unsafe {
        ic0::msg_reply();
    }
}

/// Returns the amount of cycles that were transferred by the caller
/// of the current call, and is still available in this message.
pub fn msg_cycles_available() -> u64 {
    unsafe { ic0::msg_cycles_available() as u64 }
}

/// Similar to [`msg_cycles_available`] but returns in 128-bit.
///
/// *Note*: Cycles are represented by 128-bit values.
/// The amount of cycles transferred can be obtained by
/// combining the return values: high * 264 + low.
pub fn msg_cycles_available128() -> (u64, u64) {
    let (high, low) = unsafe { ic0::msg_cycles_available128() };
    (high as u64, low as u64)
}

/// Returns the amount of cycles that came back with the response as a refund.
///
/// The refund has already been added to the canister balance automatically.
pub fn msg_cycles_refunded() -> u64 {
    unsafe { ic0::msg_cycles_refunded() as u64 }
}

/// Similar to [`msg_cycles_refunded`] but returns in 128-bit.
///
/// *Note*: Cycles are represented by 128-bit values.
/// The amount of cycles transferred can be obtained by
/// combining the return values: high * 264 + low.
pub fn msg_cycles_refunded128() -> (u64, u64) {
    let (high, low) = unsafe { ic0::msg_cycles_refunded128() };
    (high as u64, low as u64)
}

/// Moves cycles from the call to the canister balance.
///
/// The actual amounts moved will be returned
pub fn msg_cycles_accept(max_amount: u64) -> u64 {
    // TODO: should we assert the u64 input is within the range of i64?
    unsafe { ic0::msg_cycles_accept(max_amount as i64) as u64 }
}

/// Similar to [`msg_cycles_accept`] but the inputs and returns are in 128-bit.
pub fn msg_cycles_accept128(max_amount_high: u64, max_amount_low: u64) -> (u64, u64) {
    // TODO: should we assert the u64 input is within the range of i64?
    let (amount_high, amount_low) =
        unsafe { ic0::msg_cycles_accept128(max_amount_high as i64, max_amount_low as i64) };
    (amount_high as u64, amount_low as u64)
}

/// Returns the argument data as bytes.
pub(crate) unsafe fn arg_data_raw() -> Vec<u8> {
    let len: usize = ic0::msg_arg_data_size() as usize;
    let mut bytes = vec![0u8; len as usize];
    ic0::msg_arg_data_copy(bytes.as_mut_ptr() as i32, 0, len as i32);
    bytes
}

/// Returns the argument data in the current call.
pub fn arg_data<R: for<'a> ArgumentDecoder<'a>>() -> R {
    let bytes = unsafe { arg_data_raw() };

    match decode_args(&bytes) {
        Err(e) => trap(&format!("{:?}", e)),
        Ok(r) => r,
    }
}

/// Accepts the ingress message.
pub fn accept_message() {
    unsafe {
        ic0::accept_message();
    }
}

/// Returns the name of current canister method.
pub fn method_name() -> String {
    let len: u32 = unsafe { ic0::msg_method_name_size() as u32 };
    let mut bytes = vec![0; len as usize];
    unsafe {
        ic0::msg_method_name_copy(bytes.as_mut_ptr() as i32, 0, len as i32);
    }
    String::from_utf8_lossy(&bytes).to_string()
}
