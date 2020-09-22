use crate::ic0;
use ic_types::Principal;
use std::cell::RefCell;
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;
use std::task::{Context, Poll, Waker};

#[cfg(feature = "experimental")]
use crate::ic1;
use candid::ser::IDLBuilder;
use candid::{Decode, Encode};

pub mod context;
pub mod reflection;

use context::*;

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

// The callback from IC dereferences the future from a raw pointer, assigns the
// result and calls the waker. We cannot use a closure here because we pass raw
// pointers to the System and back.
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

/// Perfrom an asynchronous call to another canister via ic0.
pub async fn call<T: candid::CandidType, R: serde::de::DeserializeOwned>(
    id: Principal,
    method: &str,
    args: Option<T>,
) -> CallResult<R> {
    let args_raw = match args {
        None => candid::Encode!(),
        Some(args_raw) => candid::Encode!(&args_raw),
    }
    .expect("Failed to encode arguments.");
    let bytes = call_raw(id, method, args_raw).await?;
    Ok(Decode!(&bytes, R).unwrap())
}

/// Same as 'call', but without a return value.
pub async fn call_no_return<T: candid::CandidType>(
    id: Principal,
    method: &str,
    args: Option<T>,
) -> CallResult<()> {
    let args_raw = match args {
        None => candid::Encode!(),
        Some(args_raw) => candid::Encode!(&args_raw),
    }
    .expect("Failed to encode arguments.");
    let expect = IDLBuilder::new().serialize_to_vec().unwrap();
    let actual = call_raw(id, method, args_raw).await?;
    assert!(expect == actual);
    Ok(())
}

/// Same as 'call', but without serialization.
pub fn call_raw(
    id: Principal,
    method: &str,
    args_raw: Vec<u8>,
) -> impl Future<Output = CallResult<Vec<u8>>> {
    let callee = id.as_slice();
    let state = Rc::new(RefCell::new(CallFutureState {
        result: None,
        waker: None,
    }));
    let state_ptr = Rc::into_raw(state.clone());
    let err_code = unsafe {
        ic0::call_simple(
            callee.as_ptr() as i32,
            callee.len() as i32,
            method.as_ptr() as i32,
            method.len() as i32,
            callback as usize as i32,
            state_ptr as i32,
            callback as usize as i32,
            state_ptr as i32,
            args_raw.as_ptr() as i32,
            args_raw.len() as i32,
        )
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

/// Perfrom an asynchronous call to another canister via ic1.
#[cfg(feature = "experimental")]
pub async fn call_1<T: candid::CandidType, R: serde::de::DeserializeOwned>(
    id: Principal,
    method: String,
    args: Option<T>,
    amount: u64,
) -> CallResult<R> {
    let args_raw = match args {
        None => candid::Encode!(),
        Some(args_raw) => candid::Encode!(&args_raw),
    }
    .expect("Failed to encode arguments.");
    let bytes = call_raw_1(id, method, args_raw, amount).await?;
    Ok(Decode!(&bytes, R).unwrap())
}

/// Same as 'call_1', but without a return value.
#[cfg(feature = "experimental")]
pub async fn call_no_return_1<T: candid::CandidType>(
    id: Principal,
    method: String,
    args: Option<T>,
    amount: u64,
) -> CallResult<()> {
    let args_raw = match args {
        None => candid::Encode!(),
        Some(args_raw) => candid::Encode!(&args_raw),
    }
    .expect("Failed to encode arguments.");
    let expect = IDLBuilder::new().serialize_to_vec().unwrap();
    let actual = call_raw_1(id, method, args_raw, amount).await?;
    assert!(expect == actual);
    Ok(())
}

/// Same as 'call_1', but without serialization.
#[cfg(feature = "experimental")]
pub fn call_raw_1(
    id: Principal,
    method: String,
    args_raw: Vec<u8>,
    amount: u64,
) -> impl Future<Output = CallResult<Vec<u8>>> {
    let callee = id.as_slice();
    let state = Rc::new(RefCell::new(CallFutureState {
        result: None,
        waker: None,
    }));
    let state_ptr = Rc::into_raw(state.clone());
    let err_code = unsafe {
        ic1::call_simple(
            callee.as_ptr() as i32,
            callee.len() as i32,
            method.as_ptr() as i32,
            method.len() as i32,
            callback as i32,
            state_ptr as i32,
            callback as i32,
            state_ptr as i32,
            args_raw.as_ptr() as i32,
            args_raw.len() as i32,
            amount as i64,
        )
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

/// Prints the given message.
pub fn print<S: std::convert::AsRef<str>>(s: S) {
    let s = s.as_ref();
    unsafe {
        ic0::debug_print(s.as_ptr() as i32, s.len() as i32);
    }
}

/// Traps with the given message.
pub fn trap(message: &str) {
    unsafe {
        ic0::trap(message.as_ptr() as i32, message.len() as i32);
    }
}
