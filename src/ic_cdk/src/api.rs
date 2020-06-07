use crate::ic0;
use candid::Encode;
use std::cell::RefCell;
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;
use std::task::{Context, Poll, Waker};

pub mod call;
pub mod context;
pub mod reflection;

use context::*;

#[derive(PartialEq, Clone, Eq)]
#[repr(transparent)]
pub struct CanisterId(pub Vec<u8>);

// TODO: move this to using the ic_agent canister.
impl CanisterId {
    pub fn from_str_unchecked(s: &str) -> Result<Self, String> {
        // We don't validate the crc here.
        let s = s.split_at(3).1; // remove 'ic:'
        let s = s.split_at(s.len() - 2).0; // remove crc8
        if s.len() % 2 != 0 {
            return Err(format!("Invalid number of characters: {}", s.len()));
        }
        let s: &[u8] = s.as_bytes();

        fn val(a: u8, idx: usize) -> Result<u8, String> {
            match a {
                b'0'..=b'9' => Ok(a - b'0'),
                b'a'..=b'f' => Ok(a - b'a' + 10),
                b'A'..=b'F' => Ok(a - b'A' + 10),
                x => return Err(format!("Invalid character at pos {}: '{}'", idx, x)),
            }
        }

        let v: Result<Vec<u8>, String> = s
            .chunks(2)
            .enumerate()
            .map(|(i, pair)| Ok(val(pair[0], i)? << 4 | val(pair[1], i)?))
            .collect();

        Ok(CanisterId(v?))
    }
}

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

/// Calls another canister and returns a future.
pub fn call<T: candid::CandidType, R: serde::de::DeserializeOwned>(
    id: CanisterId,
    method_name: &str,
    arg: Option<T>,
) -> impl Future<Output = CallResult<R>> {
    // The callback from IC dereferences the future from a raw pointer, assigns the
    // result and calls the waker. We cannot use a closure here because we pass raw
    // pointers to the System and back.
    fn callback<R: serde::de::DeserializeOwned>(state_ptr: *const RefCell<CallFutureState<R>>) {
        let state = unsafe { Rc::from_raw(state_ptr) };

        // Make sure to un-borrow_mut the state.
        {
            state.borrow_mut().result = Some(match reject_code() {
                RejectionCode::NoError => Ok(arg_data::<R>()),
                n => Err((n, reject_message())),
            });
        }

        if let Some(waker) = (|| state.borrow_mut().waker.take())() {
            // This is all to protect this little guy here which will call the poll() which
            // borrow_mut() the state as well. So we need to be careful to not double-borrow_mut.
            waker.wake()
        }
    };

    let data = match arg {
        None => candid::Encode!(),
        Some(data) => candid::Encode!(&data),
    }
    .expect("Could not encode arguments.");

    let callee = id.0;
    let state = Rc::new(RefCell::new(CallFutureState {
        result: None,
        waker: None,
    }));
    let state_ptr = Rc::into_raw(state.clone());

    let err_code = unsafe {
        ic0::call_simple(
            callee.as_ptr() as i32,
            callee.len() as i32,
            method_name.as_ptr() as i32,
            method_name.len() as i32,
            callback::<R> as i32,
            state_ptr as i32,
            callback::<R> as i32,
            state_ptr as i32,
            data.as_ptr() as i32,
            data.len() as i32,
        )
    };

    // 0 is a special error code, meaning call_simple call succeeded
    if err_code != 0 {
        let mut state = state.borrow_mut();
        state.result = Some(Err((
            RejectionCode::from(err_code),
            "Couldn't send message".to_string(),
        )));
    }

    CallFuture { state }
}

/// Calls another canister and returns a future.
pub fn call_no_return<T: candid::CandidType>(
    id: CanisterId,
    method_name: &str,
    arg: Option<T>,
) -> impl Future<Output = CallResult<()>> {
    // The callback from IC dereferences the future from a raw pointer, assigns the
    // result and calls the waker. We cannot use a closure here because we pass raw
    // pointers to the System and back.
    fn callback(state_ptr: *const RefCell<CallFutureState<()>>) {
        let state = unsafe { Rc::from_raw(state_ptr) };

        // Make sure to un-borrow_mut the state.
        {
            state.borrow_mut().result = Some(match reject_code() {
                RejectionCode::NoError => Ok(arg_data_empty()),
                n => Err((n, reject_message())),
            });
        }

        if let Some(waker) = (|| state.borrow_mut().waker.take())() {
            // This is all to protect this little guy here which will call the poll() which
            // borrow_mut() the state as well. So we need to be careful to not double-borrow_mut.
            waker.wake()
        }
    };

    let data = match arg {
        None => candid::Encode!(),
        Some(data) => candid::Encode!(&data),
    }
    .expect("Could not encode arguments.");

    let callee = id.0;
    let state = Rc::new(RefCell::new(CallFutureState {
        result: None,
        waker: None,
    }));
    let state_ptr = Rc::into_raw(state.clone());

    let err_code = unsafe {
        ic0::call_simple(
            callee.as_ptr() as i32,
            callee.len() as i32,
            method_name.as_ptr() as i32,
            method_name.len() as i32,
            callback as i32,
            state_ptr as i32,
            callback as i32,
            state_ptr as i32,
            data.as_ptr() as i32,
            data.len() as i32,
        )
    };

    // 0 is a special error code, meaning call_simple call succeeded
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
