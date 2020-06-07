use crate::api::CanisterId;
use crate::ic0;

/// Returns the caller of the current call.
pub fn caller() -> Vec<u8> {
    let len: u32 = unsafe { ic0::msg_caller_size() as u32 };
    let mut bytes = vec![0; len as usize];
    unsafe {
        ic0::msg_caller_copy(bytes.as_mut_ptr() as i32, 0, len as i32);
    }
    bytes
}

/// Returns the canister id as a blob.
pub fn id() -> CanisterId {
    let len: u32 = unsafe { ic0::canister_self_size() as u32 };
    let mut bytes = vec![0; len as usize];
    unsafe {
        ic0::canister_self_copy(bytes.as_mut_ptr() as i32, 0, len as i32);
    }
    CanisterId(bytes)
}
