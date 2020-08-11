use crate::ic0;
use candid::Encode;
use std::cell::RefCell;
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;
use std::task::{Context, Poll, Waker};

pub mod context;
pub mod reflection;

use context::*;

#[derive(PartialEq, Clone, Eq)]
#[repr(transparent)]
pub struct CanisterId(pub Vec<u8>);

// TODO: move this to using the ic_agent canister.
impl CanisterId {
    pub fn from_str(input: &str) -> Result<Self, CanisterIdParseError> {
        // Strategy: Parse very liberally, then pretty-print and compare output
        // This is both simpler and yields better error messages

        let mut s = input.to_string();
        s.make_ascii_lowercase();
        s.retain(|c| c.is_ascii_alphanumeric());
        match base32::decode(base32::Alphabet::RFC4648 { padding: false }, &s) {
            Some(mut bytes) => {
                if bytes.len() < 4 {
                    return Err(CanisterIdParseError::TooShort);
                }
                let result = CanisterId(bytes.split_off(4));
                let expected = format!("{}", result);
                if input != expected {
                    return Err(CanisterIdParseError::InvalidChecksum {
                        found: input.to_string(),
                        expected,
                    });
                }
                Ok(result)
            }
            None => Err(CanisterIdParseError::NotBase32),
        }
    }
}

impl std::fmt::Display for CanisterId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let blob = &self.0;
        // Calculate CRC32 digest of the Canister ID
        let mut hasher = crc32fast::Hasher::new();
        hasher.update(&blob);
        let checksum = hasher.finalize();

        // Append the Canister ID bytes to the calculated CRC32 digest
        let mut bytes = vec![];
        bytes.extend(&(checksum.to_be_bytes().to_vec()));
        bytes.extend_from_slice(&blob);

        // Base32-encode the concatenated bytes
        let mut s = base32::encode(base32::Alphabet::RFC4648 { padding: false }, &bytes);
        s.make_ascii_lowercase();

        // Print with a separator - (dash) inserted every 5 characters.
        while s.len() > 5 {
            // too bad split_off does not work the other way
            let rest = s.split_off(5);
            write!(f, "{}-", s)?;
            s = rest;
        }
        write!(f, "{}", s)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum CanisterIdParseError {
    TooShort,
    NotBase32,
    InvalidChecksum { found: String, expected: String },
}

impl std::fmt::Display for CanisterIdParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TooShort => write!(f, "CanisterId textual representation is too short"),
            Self::NotBase32 => write!(
                f,
                "Cannot decode CanisterId textual representation as base32"
            ),
            Self::InvalidChecksum { found, expected } => write!(
                f,
                "CanisterId {} failed checksum validation, expected {}",
                found, expected
            ),
        }
    }
}

impl std::fmt::Debug for CanisterId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "CanisterId {}", self)
    }
}

impl From<Vec<u8>> for CanisterId {
    fn from(item: Vec<u8>) -> CanisterId {
        CanisterId { 0: item }
    }
}

impl Into<std::vec::Vec<u8>> for CanisterId {
    fn into(self) -> std::vec::Vec<u8> {
        self.0
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
                RejectionCode::NoError => Ok(arg_data_1::<R>()),
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
                RejectionCode::NoError => Ok(arg_data_0()),
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

#[cfg(test)]
mod tests;
