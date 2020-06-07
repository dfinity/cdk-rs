use crate::{ic0, reflection};
use candid::{Decode, Encode};

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

/// Returns a result that maps over the call; it will be Ok(T) if
/// the call succeeded (with T being the arg_data), and [reject_message()] if it failed.
pub fn result<T: serde::de::DeserializeOwned>() -> Result<T, String> {
    match reject_code() {
        RejectionCode::NoError => Ok(arg_data::<T>()),
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
pub fn reject(message: &str) -> () {
    let err_message = message.as_bytes();
    unsafe {
        ic0::msg_reject(err_message.as_ptr() as i32, err_message.len() as i32);
    }
}

pub(crate) unsafe fn reply_raw(reply: &[u8]) {
    ic0::msg_reply_data_append(reply.as_ptr() as i32, reply.len() as i32);
    ic0::msg_reply();
}

pub fn reply<T: candid::CandidType>(reply: T) {
    let bytes = Encode!(&reply).expect("Could not encode reply.");
    unsafe {
        reply_raw(&bytes);
    }
}

pub fn reply_empty() {
    let bytes = Encode!().expect("Could not encode reply.");
    unsafe {
        reply_raw(&bytes);
    }
}

pub(crate) unsafe fn arg_data_raw() -> Vec<u8> {
    let len: usize = ic0::msg_arg_data_size() as usize;
    let mut bytes = vec![0u8; len as usize];
    ic0::msg_arg_data_copy(bytes.as_mut_ptr() as i32, 0, len as i32);
    bytes
}

pub fn arg_data<T: serde::de::DeserializeOwned>() -> T {
    unsafe { Decode!(&arg_data_raw(), T).unwrap() }
}

pub fn arg_data_empty() -> () {
    unsafe { Decode!(&arg_data_raw()).unwrap() }
}

pub fn sender() -> Vec<u8> {
    reflection::caller()
}
