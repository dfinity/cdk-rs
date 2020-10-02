//! APIs to make and manage calls in the canister.
use crate::api::{ic0, trap};
use crate::export::Principal;
use candid::de::ArgumentDecoder;
use candid::ser::ArgumentEncoder;
use candid::{decode_args, encode_args};
use std::cell::RefCell;
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;
use std::task::{Context, Poll, Waker};

/// Rejection code from calling another canister.
/// These can be obtained either using `reject_code()` or `reject_result()`.
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

/// The result of a Call. Errors on the IC have two components; a Code and a message
/// associated with it.
pub type CallResult<R> = Result<R, (RejectionCode, String)>;

/// Internal state for the Future when sending a call.
struct CallFutureState<R: serde::de::DeserializeOwned> {
    result: Option<CallResult<R>>,
    waker: Option<Waker>,
}

struct CallFuture<R: serde::de::DeserializeOwned> {
    // We basically use Rc instead of Arc (since we're single threaded), and use
    // RefCell instead of Mutex (because we cannot lock in WASM).
    state: Rc<RefCell<CallFutureState<R>>>,
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
fn callback(state_ptr: *const RefCell<CallFutureState<Vec<u8>>>) {
    let state = unsafe { Rc::from_raw(state_ptr) };
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

/// Same as 'call', but without serialization.
pub fn call_raw(
    id: Principal,
    method: &str,
    args_raw: Vec<u8>,
    payment: i64,
) -> impl Future<Output = CallResult<Vec<u8>>> {
    let callee = id.as_slice();
    let state = Rc::new(RefCell::new(CallFutureState {
        result: None,
        waker: None,
    }));
    let state_ptr = Rc::into_raw(state.clone());
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
            let bytes = vec![0u8];
            ic0::call_funds_add(bytes.as_ptr() as i32, bytes.len() as i32, payment as i64);
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

/// Perform an asynchronous call to another canister via ic0.
pub async fn call<T: ArgumentEncoder, R: for<'a> ArgumentDecoder<'a>>(
    id: Principal,
    method: &str,
    args: T,
) -> CallResult<R> {
    let args_raw = encode_args(args).expect("Failed to encode arguments.");
    let bytes = call_raw(id, method, args_raw, 0).await?;
    decode_args(&bytes).map_err(|err| trap(&format!("{:?}", err)))
}

pub async fn call_with_payment<T: ArgumentEncoder, R: for<'a> ArgumentDecoder<'a>>(
    id: Principal,
    method: &str,
    args: T,
    cycles: i64,
) -> CallResult<R> {
    let args_raw = encode_args(args).expect("Failed to encode arguments.");
    let bytes = call_raw(id, method, args_raw, cycles).await?;
    decode_args(&bytes).map_err(|err| trap(&format!("{:?}", err)))
}

/// Returns a result that maps over the call; it will be Ok(T) if
/// the call succeeded (with T being the arg_data), and [reject_message()] if it failed.
pub fn result<T: for<'a> ArgumentDecoder<'a>>() -> Result<T, String> {
    match reject_code() {
        RejectionCode::NoError => decode_args(&unsafe { arg_data_raw() })
            .map_err(|e| format!("Failed to decode arguments: {}", e)),
        _ => Err(reject_message()),
    }
}

/// Get the rejection code for the call.
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

/// Reject the current call with the message.
pub fn reject(message: &str) {
    let err_message = message.as_bytes();
    unsafe {
        ic0::msg_reject(err_message.as_ptr() as i32, err_message.len() as i32);
    }
}

/// Reply to the current call with a candid argument.
pub fn reply<T: ArgumentEncoder>(reply: T) {
    let bytes = encode_args(reply).expect("Could not encode reply.");
    unsafe {
        ic0::msg_reply_data_append(bytes.as_ptr() as i32, bytes.len() as i32);
        ic0::msg_reply();
    }
}

/// Economics.
///
/// # Warning
/// This section will be moved and breaking changes significantly before Mercury.
/// The APIs behind it will stay the same, so deployed canisters will keep working.
pub mod funds {
    use super::ic0;

    pub enum Unit {
        Cycle,
        IcpToken,
    }

    impl Unit {
        pub fn to_bytes(&self) -> Vec<u8> {
            match self {
                Unit::Cycle => vec![0],
                Unit::IcpToken => vec![1],
            }
        }
    }

    pub fn available(unit: Unit) -> i64 {
        let bytes = unit.to_bytes();
        unsafe { ic0::msg_funds_available(bytes.as_ptr() as i32, bytes.len() as i32) }
    }

    pub fn refunded(unit: Unit) -> i64 {
        let bytes = unit.to_bytes();
        unsafe { ic0::msg_funds_refunded(bytes.as_ptr() as i32, bytes.len() as i32) }
    }

    pub fn accept(unit: Unit, amount: i64) {
        let bytes = unit.to_bytes();
        unsafe { ic0::msg_funds_accept(bytes.as_ptr() as i32, bytes.len() as i32, amount) }
    }
}

pub(crate) unsafe fn arg_data_raw() -> Vec<u8> {
    let len: usize = ic0::msg_arg_data_size() as usize;
    let mut bytes = vec![0u8; len as usize];
    ic0::msg_arg_data_copy(bytes.as_mut_ptr() as i32, 0, len as i32);
    bytes
}

/// Get the argument data in the current call.
pub fn arg_data<R: for<'a> ArgumentDecoder<'a>>() -> R {
    let bytes = unsafe { arg_data_raw() };

    match decode_args(&bytes) {
        Err(e) => trap(&format!("{:?}", e)),
        Ok(r) => r,
    }
}
