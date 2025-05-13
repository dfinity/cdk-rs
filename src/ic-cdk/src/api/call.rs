//! APIs to make and manage calls in the canister.
use crate::api::trap;
use candid::utils::{decode_args_with_config_debug, ArgumentDecoder, ArgumentEncoder};
use candid::{
    decode_args, encode_args, write_args, CandidType, DecoderConfig, Deserialize, Principal,
};
use serde::ser::Error;
use std::future::Future;
use std::marker::PhantomData;
use std::pin::Pin;
use std::sync::atomic::Ordering;
use std::sync::{Arc, RwLock, Weak};
use std::task::{Context, Poll, Waker};

/// Rejection code from calling another canister.
///
/// These can be obtained either using `reject_code()` or `reject_result()`.
#[allow(missing_docs)]
#[repr(i32)]
#[derive(CandidType, Deserialize, Clone, Copy, Hash, Debug, PartialEq, Eq, PartialOrd, Ord)]
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
struct CallFutureState<T: AsRef<[u8]>> {
    result: Option<CallResult<Vec<u8>>>,
    waker: Option<Waker>,
    id: Principal,
    method: String,
    arg: T,
    payment: u128,
}

struct CallFuture<T: AsRef<[u8]>> {
    state: Arc<RwLock<CallFutureState<T>>>,
}

impl<T: AsRef<[u8]>> Future for CallFuture<T> {
    type Output = CallResult<Vec<u8>>;

    fn poll(self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Self::Output> {
        let self_ref = Pin::into_inner(self);
        let mut state = self_ref.state.write().unwrap();

        if let Some(result) = state.result.take() {
            Poll::Ready(result)
        } else {
            if state.waker.is_none() {
                let callee = state.id.as_slice();
                let method = &state.method;
                let args = state.arg.as_ref();
                let payment = state.payment;
                let state_ptr = Weak::into_raw(Arc::downgrade(&self_ref.state));
                // SAFETY:
                // `callee`, being &[u8], is a readable sequence of bytes and therefore can be passed to ic0.call_new.
                // `method`, being &str, is a readable sequence of bytes and therefore can be passed to ic0.call_new.
                // `callback` is a function with signature (env : i32) -> () and therefore can be called as both reply and reject fn for ic0.call_new.
                // `state_ptr` is a pointer created via Weak::into_raw, and can therefore be passed as the userdata for `callback`.
                // `args`, being a &[u8], is a readable sequence of bytes and therefore can be passed to ic0.call_data_append.
                // `cleanup` is a function with signature (env : i32) -> () and therefore can be called as a cleanup fn for ic0.call_on_cleanup.
                // `state_ptr` is a pointer created via Weak::into_raw, and can therefore be passed as the userdata for `cleanup`.
                // ic0.call_perform is always safe to call.
                // callback and cleanup are safe to parameterize with T because:
                // - if the future is dropped before the callback is called, there will be no more strong references and the weak reference will fail to upgrade
                // - if the future is *not* dropped before the callback is called, the compiler will mandate that any data borrowed by T is still alive
                let err_code = unsafe {
                    ic0::call_new(
                        callee.as_ptr() as i32,
                        callee.len() as i32,
                        method.as_ptr() as i32,
                        method.len() as i32,
                        callback::<T> as usize as i32,
                        state_ptr as i32,
                        callback::<T> as usize as i32,
                        state_ptr as i32,
                    );

                    ic0::call_data_append(args.as_ptr() as i32, args.len() as i32);
                    add_payment(payment);
                    ic0::call_on_cleanup(cleanup::<T> as usize as i32, state_ptr as i32);
                    ic0::call_perform()
                };

                // 0 is a special error code meaning call succeeded.
                if err_code != 0 {
                    let result = Err((
                        RejectionCode::from(err_code),
                        "Couldn't send message".to_string(),
                    ));
                    state.result = Some(result.clone());
                    return Poll::Ready(result);
                }
            }
            state.waker = Some(context.waker().clone());
            Poll::Pending
        }
    }
}

/// The callback from IC dereferences the future from a raw pointer, assigns the
/// result and calls the waker. We cannot use a closure here because we pass raw
/// pointers to the System and back.
///
/// # Safety
///
/// This function must only be passed to the IC with a pointer from Weak::into_raw as userdata.
unsafe extern "C" fn callback<T: AsRef<[u8]>>(state_ptr: *const RwLock<CallFutureState<T>>) {
    // SAFETY: This function is only ever called by the IC, and we only ever pass a Weak as userdata.
    let state = unsafe { Weak::from_raw(state_ptr) };
    if let Some(state) = state.upgrade() {
        // Make sure to un-borrow_mut the state.
        {
            state.write().unwrap().result = Some(match reject_code() {
                RejectionCode::NoError => Ok(arg_data_raw()),
                n => Err((n, reject_message())),
            });
        }
        let w = state.write().unwrap().waker.take();
        if let Some(waker) = w {
            // This is all to protect this little guy here which will call the poll() which
            // borrow_mut() the state as well. So we need to be careful to not double-borrow_mut.
            waker.wake()
        }
    }
}

/// This function is called when [callback] was just called with the same parameter, and trapped.
/// We can't guarantee internal consistency at this point, but we can at least e.g. drop mutex guards.
/// Waker is a very opaque API, so the best we can do is set a global flag and proceed normally.
///
/// # Safety
///
/// This function must only be passed to the IC with a pointer from Weak::into_raw as userdata.
unsafe extern "C" fn cleanup<T: AsRef<[u8]>>(state_ptr: *const RwLock<CallFutureState<T>>) {
    // SAFETY: This function is only ever called by the IC, and we only ever pass a Weak as userdata.
    let state = unsafe { Weak::from_raw(state_ptr) };
    if let Some(state) = state.upgrade() {
        // We set the call result, even though it won't be read on the
        // default executor, because we can't guarantee it was called on
        // our executor. However, we are not allowed to inspect
        // reject_code() inside of a cleanup callback, so always set the
        // result to a reject.
        //
        // Borrowing does not trap - the rollback from the
        // previous trap ensures that the RwLock can be borrowed again.
        state.write().unwrap().result = Some(Err((RejectionCode::NoError, "cleanup".to_string())));
        let w = state.write().unwrap().waker.take();
        if let Some(waker) = w {
            // Flag that we do not want to actually wake the task - we
            // want to drop it *without* executing it.
            ic_cdk_executor::CLEANUP.store(true, Ordering::Relaxed);
            waker.wake();
            ic_cdk_executor::CLEANUP.store(false, Ordering::Relaxed);
        }
    }
}

fn add_payment(payment: u128) {
    if payment == 0 {
        return;
    }
    let high = (payment >> 64) as u64;
    let low = (payment & u64::MAX as u128) as u64;
    // SAFETY: ic0.call_cycles_add128 is always safe to call.
    unsafe {
        ic0::call_cycles_add128(high as i64, low as i64);
    }
}

/// Sends a one-way message with `payment` cycles attached to it that invokes `method` with
/// arguments `args` on the principal identified by `id`, ignoring the reply.
///
/// Returns `Ok(())` if the message was successfully enqueued, otherwise returns a reject code.
///
/// # Notes
///
///   * The caller has no way of checking whether the destination processed the notification.
///     The system can drop the notification if the destination does not have resources to
///     process the message (for example, if it's out of cycles or queue slots).
///
///   * The callee cannot tell whether the call is one-way or not.
///     The callee must produce replies for all incoming messages.
///
///   * It is safe to upgrade a canister without stopping it first if it sends out *only*
///     one-way messages.
///
///   * If the payment is non-zero and the system fails to deliver the notification, the behaviour
///     is unspecified: the funds can be either reimbursed or consumed irrevocably by the IC depending
///     on the underlying implementation of one-way calls.
pub fn notify_with_payment128<T: ArgumentEncoder>(
    id: Principal,
    method: &str,
    args: T,
    payment: u128,
) -> Result<(), RejectionCode> {
    let args_raw = encode_args(args).expect("failed to encode arguments");
    notify_raw(id, method, &args_raw, payment)
}

/// Like [notify_with_payment128], but sets the payment to zero.
pub fn notify<T: ArgumentEncoder>(
    id: Principal,
    method: &str,
    args: T,
) -> Result<(), RejectionCode> {
    notify_with_payment128(id, method, args, 0)
}

/// Like [notify], but sends the argument as raw bytes, skipping Candid serialization.
pub fn notify_raw(
    id: Principal,
    method: &str,
    args_raw: &[u8],
    payment: u128,
) -> Result<(), RejectionCode> {
    let callee = id.as_slice();
    // We set all callbacks to -1, which is guaranteed to be invalid callback index.
    // The system will still deliver the reply, but it will trap immediately because the callback
    // is not a valid function. See
    // https://www.joachim-breitner.de/blog/789-Zero-downtime_upgrades_of_Internet_Computer_canisters#one-way-calls
    // for more context.

    // SAFETY:
    // `callee`, being &[u8], is a readable sequence of bytes and therefore can be passed to ic0.call_new.
    // `method`, being &str, is a readable sequence of bytes and therefore can be passed to ic0.call_new.
    // -1, i.e. usize::MAX, is a function pointer the wasm module cannot possibly contain, and therefore can be passed as both reply and reject fn for ic0.call_new.
    // Since the callback function will never be called, any value can be passed as its context parameter, and therefore -1 can be passed for those values.
    // `args`, being a &[u8], is a readable sequence of bytes and therefore can be passed to ic0.call_data_append.
    // ic0.call_perform is always safe to call.
    let err_code = unsafe {
        ic0::call_new(
            callee.as_ptr() as i32,
            callee.len() as i32,
            method.as_ptr() as i32,
            method.len() as i32,
            /* reply_fun = */ -1,
            /* reply_env = */ -1,
            /* reject_fun = */ -1,
            /* reject_env = */ -1,
        );
        add_payment(payment);
        ic0::call_data_append(args_raw.as_ptr() as i32, args_raw.len() as i32);
        ic0::call_perform()
    };
    match err_code {
        0 => Ok(()),
        c => Err(RejectionCode::from(c)),
    }
}

/// Performs an asynchronous call to another canister and pay cycles at the same time.
///
/// Treats arguments and returns as raw bytes. No data serialization and deserialization is performed.
///
/// # Example
///
/// It can be called:
///
/// ```rust
/// # use ic_cdk::api::call::call_raw;
/// # fn callee_canister() -> candid::Principal { unimplemented!() }
/// async fn call_add_user() -> Vec<u8>{
///     call_raw(callee_canister(), "add_user", b"abcd", 1_000_000u64).await.unwrap()
/// }
/// ```
pub fn call_raw<'a, T: AsRef<[u8]> + Send + Sync + 'a>(
    id: Principal,
    method: &str,
    args_raw: T,
    payment: u64,
) -> impl Future<Output = CallResult<Vec<u8>>> + Send + Sync + 'a {
    call_raw_internal(id, method, args_raw, payment.into())
}

/// Performs an asynchronous call to another canister and pay cycles (in `u128`) at the same time.
///
/// Treats arguments and returns as raw bytes. No data serialization and deserialization is performed.
/// # Example
///
/// It can be called:
///
/// ```rust
/// # use ic_cdk::api::call::call_raw128;
/// # fn callee_canister() -> candid::Principal { unimplemented!() }
/// async fn call_add_user() -> Vec<u8>{
///     call_raw128(callee_canister(), "add_user", b"abcd", 1_000_000u128).await.unwrap()
/// }
/// ```
pub fn call_raw128<'a, T: AsRef<[u8]> + Send + Sync + 'a>(
    id: Principal,
    method: &str,
    args_raw: T,
    payment: u128,
) -> impl Future<Output = CallResult<Vec<u8>>> + Send + Sync + 'a {
    call_raw_internal(id, method, args_raw, payment)
}

fn call_raw_internal<'a, T: AsRef<[u8]> + Send + Sync + 'a>(
    id: Principal,
    method: &str,
    args_raw: T,
    payment: u128,
) -> impl Future<Output = CallResult<Vec<u8>>> + Send + Sync + 'a {
    let state = Arc::new(RwLock::new(CallFutureState {
        result: None,
        waker: None,
        id,
        method: method.to_string(),
        arg: args_raw,
        payment,
    }));
    CallFuture { state }
}

fn decoder_error_to_reject<T>(err: candid::error::Error) -> (RejectionCode, String) {
    (
        RejectionCode::CanisterError,
        format!(
            "failed to decode canister response as {}: {}",
            std::any::type_name::<T>(),
            err
        ),
    )
}

/// Performs an asynchronous call to another canister.
///
/// # Example
///
/// Assuming that the callee canister has following interface:
///
/// ```text
/// service : {
///     add_user: (name: text) -> (nat64);
/// }
/// ```
///
/// It can be called:
///
/// ```rust
/// # use ic_cdk::api::call::call;
/// # fn callee_canister() -> candid::Principal { unimplemented!() }
/// async fn call_add_user() -> u64 {
///     let (user_id,) = call(callee_canister(), "add_user", ("Alice".to_string(),)).await.unwrap();
///     user_id
/// }
/// ```
///
/// # Note
///
/// * Both argument and return types are tuples even if it has only one value, e.g `(user_id,)`, `("Alice".to_string(),)`.
/// * The type annotation on return type is required. Or the return type can be inferred from the context.
/// * The asynchronous call must be awaited in order for the inter-canister call to be made.
/// * If the reply payload is not a valid encoding of the expected type `T`, the call results in [RejectionCode::CanisterError] error.
pub fn call<T: ArgumentEncoder, R: for<'a> ArgumentDecoder<'a>>(
    id: Principal,
    method: &str,
    args: T,
) -> impl Future<Output = CallResult<R>> + Send + Sync {
    let args_raw = encode_args(args).expect("Failed to encode arguments.");
    let fut = call_raw(id, method, args_raw, 0);
    async {
        let bytes = fut.await?;
        decode_args(&bytes).map_err(decoder_error_to_reject::<R>)
    }
}

/// Performs an asynchronous call to another canister and pay cycles at the same time.
///
/// # Example
///
/// Assuming that the callee canister has following interface:
///
/// ```text
/// service : {
///     add_user: (name: text) -> (nat64);
/// }
/// ```
///
/// It can be called:
///
/// ```rust
/// # use ic_cdk::api::call::call_with_payment;
/// # fn callee_canister() -> candid::Principal { unimplemented!() }
/// async fn call_add_user() -> u64 {
///     let (user_id,) = call_with_payment(callee_canister(), "add_user", ("Alice".to_string(),), 1_000_000u64).await.unwrap();
///     user_id
/// }
/// ```
///
/// # Note
///
/// * Both argument and return types are tuples even if it has only one value, e.g `(user_id,)`, `("Alice".to_string(),)`.
/// * The type annotation on return type is required. Or the return type can be inferred from the context.
/// * The asynchronous call must be awaited in order for the inter-canister call to be made.
/// * If the reply payload is not a valid encoding of the expected type `T`, the call results in [RejectionCode::CanisterError] error.
pub fn call_with_payment<T: ArgumentEncoder, R: for<'a> ArgumentDecoder<'a>>(
    id: Principal,
    method: &str,
    args: T,
    cycles: u64,
) -> impl Future<Output = CallResult<R>> + Send + Sync {
    let args_raw = encode_args(args).expect("Failed to encode arguments.");
    let fut = call_raw(id, method, args_raw, cycles);
    async {
        let bytes = fut.await?;
        decode_args(&bytes).map_err(decoder_error_to_reject::<R>)
    }
}

/// Performs an asynchronous call to another canister and pay cycles (in `u128`) at the same time.
///
/// # Example
///
/// Assuming that the callee canister has following interface:
///
/// ```text
/// service : {
///     add_user: (name: text) -> (nat64);
/// }
/// ```
///
/// It can be called:
///
/// ```rust
/// # use ic_cdk::api::call::call_with_payment128;
/// # fn callee_canister() -> candid::Principal { unimplemented!() }
/// async fn call_add_user() -> u64 {
///     let (user_id,) = call_with_payment128(callee_canister(), "add_user", ("Alice".to_string(),), 1_000_000u128).await.unwrap();
///     user_id
/// }
/// ```
///
/// # Note
///
/// * Both argument and return types are tuples even if it has only one value, e.g `(user_id,)`, `("Alice".to_string(),)`.
/// * The type annotation on return type is required. Or the return type can be inferred from the context.
/// * The asynchronous call must be awaited in order for the inter-canister call to be made.
/// * If the reply payload is not a valid encoding of the expected type `T`, the call results in [RejectionCode::CanisterError] error.
pub fn call_with_payment128<T: ArgumentEncoder, R: for<'a> ArgumentDecoder<'a>>(
    id: Principal,
    method: &str,
    args: T,
    cycles: u128,
) -> impl Future<Output = CallResult<R>> + Send + Sync {
    let args_raw = encode_args(args).expect("Failed to encode arguments.");
    let fut = call_raw128(id, method, args_raw, cycles);
    async {
        let bytes = fut.await?;
        decode_args(&bytes).map_err(decoder_error_to_reject::<R>)
    }
}

/// Performs an asynchronous call to another canister and pay cycles (in `u128`).
/// It also allows setting a quota for decoding the return values.
/// The decoding quota is strongly recommended when calling third-party or untrusted canisters.
///
/// # Example
///
/// Assuming that the callee canister has following interface:
///
/// ```text
/// service : {
///     add_user: (name: text) -> (nat64);
/// }
/// ```
///
/// It can be called:
///
/// ```rust
/// # use ic_cdk::api::call::{call_with_config, ArgDecoderConfig};
/// # fn callee_canister() -> candid::Principal { unimplemented!() }
/// async fn call_add_user() -> u64 {
///     let config = ArgDecoderConfig {
///         // The function only returns a nat64, to accomodate future upgrades, we set a larger decoding_quota.
///         decoding_quota: Some(10_000),
///         // To accomodate future upgrades, reserve some skipping_quota.
///         skipping_quota: Some(100),
///         // Enable debug mode to print decoding instructions and cost to the replica log.
///         debug: true,
///     };
///     let (user_id,) = call_with_config(callee_canister(), "add_user", ("Alice".to_string(),), 1_000_000u128, &config).await.unwrap();
///     user_id
/// }
/// ```
pub fn call_with_config<'b, T: ArgumentEncoder, R: for<'a> ArgumentDecoder<'a>>(
    id: Principal,
    method: &'b str,
    args: T,
    cycles: u128,
    arg_config: &'b ArgDecoderConfig,
) -> impl Future<Output = CallResult<R>> + Send + Sync + 'b {
    let args_raw = encode_args(args).expect("Failed to encode arguments.");
    let fut = call_raw128(id, method, args_raw, cycles);
    async move {
        let bytes = fut.await?;
        let config = arg_config.to_candid_config();
        let pre_cycles = if arg_config.debug {
            Some(crate::api::performance_counter(0))
        } else {
            None
        };
        match decode_args_with_config_debug(&bytes, &config) {
            Err(e) => Err(decoder_error_to_reject::<R>(e)),
            Ok((r, cost)) => {
                if arg_config.debug {
                    print_decoding_debug_info(&format!("{method} return"), &cost, pre_cycles);
                }
                Ok(r)
            }
        }
    }
}

fn print_decoding_debug_info(title: &str, cost: &DecoderConfig, pre_cycles: Option<u64>) {
    use crate::api::{performance_counter, print};
    let pre_cycles = pre_cycles.unwrap_or(0);
    let instrs = performance_counter(0) - pre_cycles;
    print(format!("[Debug] {title} decoding instructions: {instrs}"));
    if let Some(n) = cost.decoding_quota {
        print(format!("[Debug] {title} decoding cost: {n}"));
    }
    if let Some(n) = cost.skipping_quota {
        print(format!("[Debug] {title} skipping cost: {n}"));
    }
}

/// Returns a result that maps over the call
///
/// It will be Ok(T) if the call succeeded (with T being the arg_data),
/// and [reject_message()] if it failed.
pub fn result<T: for<'a> ArgumentDecoder<'a>>() -> Result<T, String> {
    match reject_code() {
        RejectionCode::NoError => {
            decode_args(&arg_data_raw()).map_err(|e| format!("Failed to decode arguments: {}", e))
        }
        _ => Err(reject_message()),
    }
}

/// Returns the rejection code for the call.
pub fn reject_code() -> RejectionCode {
    // SAFETY: ic0.msg_reject_code is always safe to call.
    let code = unsafe { ic0::msg_reject_code() };
    RejectionCode::from(code)
}

/// Returns the rejection message.
pub fn reject_message() -> String {
    // SAFETY: ic0.msg_reject_msg_size is always safe to call.
    let len: u32 = unsafe { ic0::msg_reject_msg_size() as u32 };
    let mut bytes = vec![0u8; len as usize];
    // SAFETY: `bytes`, being mutable and allocated to `len` bytes, is safe to pass to ic0.msg_reject_msg_copy with no offset
    unsafe {
        ic0::msg_reject_msg_copy(bytes.as_mut_ptr() as i32, 0, len as i32);
    }
    String::from_utf8_lossy(&bytes).into_owned()
}

/// Rejects the current call with the message.
pub fn reject(message: &str) {
    let err_message = message.as_bytes();
    // SAFETY: `err_message`, being &[u8], is a readable sequence of bytes, and therefore valid to pass to ic0.msg_reject.
    unsafe {
        ic0::msg_reject(err_message.as_ptr() as i32, err_message.len() as i32);
    }
}

/// An io::Write for message replies.
#[derive(Debug, Copy, Clone)]
pub struct CallReplyWriter;

impl std::io::Write for CallReplyWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        // SAFETY: buf, being &[u8], is a readable sequence of bytes, and therefore valid to pass to ic0.msg_reply_data_append.
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
    // SAFETY: ic0.msg_reply is always safe to call.
    unsafe {
        ic0::msg_reply();
    }
}

/// Returns the amount of cycles that were transferred by the caller
/// of the current call, and is still available in this message.
pub fn msg_cycles_available() -> u64 {
    // SAFETY: ic0.msg_cycles_available is always safe to call.
    unsafe { ic0::msg_cycles_available() as u64 }
}

/// Returns the amount of cycles that were transferred by the caller
/// of the current call, and is still available in this message.
pub fn msg_cycles_available128() -> u128 {
    let mut recv = 0u128;
    // SAFETY: recv is writable and sixteen bytes wide, and therefore is safe to pass to ic0.msg_cycles_available128
    unsafe {
        ic0::msg_cycles_available128(&mut recv as *mut u128 as i32);
    }
    recv
}

/// Returns the amount of cycles that came back with the response as a refund.
///
/// The refund has already been added to the canister balance automatically.
pub fn msg_cycles_refunded() -> u64 {
    // SAFETY: ic0.msg_cycles_refunded is always safe to call
    unsafe { ic0::msg_cycles_refunded() as u64 }
}

/// Returns the amount of cycles that came back with the response as a refund.
///
/// The refund has already been added to the canister balance automatically.
pub fn msg_cycles_refunded128() -> u128 {
    let mut recv = 0u128;
    // SAFETY: recv is writable and sixteen bytes wide, and therefore is safe to pass to ic0.msg_cycles_refunded128
    unsafe {
        ic0::msg_cycles_refunded128(&mut recv as *mut u128 as i32);
    }
    recv
}

/// Moves cycles from the call to the canister balance.
///
/// The actual amount moved will be returned.
pub fn msg_cycles_accept(max_amount: u64) -> u64 {
    // SAFETY: ic0.msg_cycles_accept is always safe to call.
    unsafe { ic0::msg_cycles_accept(max_amount as i64) as u64 }
}

/// Moves cycles from the call to the canister balance.
///
/// The actual amount moved will be returned.
pub fn msg_cycles_accept128(max_amount: u128) -> u128 {
    let high = (max_amount >> 64) as u64;
    let low = (max_amount & u64::MAX as u128) as u64;
    let mut recv = 0u128;
    // SAFETY: `recv` is writable and sixteen bytes wide, and therefore safe to pass to ic0.msg_cycles_accept128
    unsafe {
        ic0::msg_cycles_accept128(high as i64, low as i64, &mut recv as *mut u128 as i32);
    }
    recv
}

/// Returns the argument data as bytes.
pub fn arg_data_raw() -> Vec<u8> {
    // SAFETY: ic0.msg_arg_data_size is always safe to call.
    let len: usize = unsafe { ic0::msg_arg_data_size() as usize };
    let mut bytes = Vec::with_capacity(len);
    // SAFETY:
    // `bytes`, being mutable and allocated to `len` bytes, is safe to pass to ic0.msg_arg_data_copy with no offset
    // ic0.msg_arg_data_copy writes to all of `bytes[0..len]`, so `set_len` is safe to call with the new len.
    unsafe {
        ic0::msg_arg_data_copy(bytes.as_mut_ptr() as i32, 0, len as i32);
        bytes.set_len(len);
    }
    bytes
}

/// Gets the len of the raw-argument-data-bytes.
pub fn arg_data_raw_size() -> usize {
    // SAFETY: ic0.msg_arg_data_size is always safe to call.
    unsafe { ic0::msg_arg_data_size() as usize }
}

/// Replies with the bytes passed
pub fn reply_raw(buf: &[u8]) {
    if !buf.is_empty() {
        // SAFETY: `buf`, being &[u8], is a readable sequence of bytes, and therefore valid to pass to ic0.msg_reject.
        unsafe { ic0::msg_reply_data_append(buf.as_ptr() as i32, buf.len() as i32) }
    };
    // SAFETY: ic0.msg_reply is always safe to call.
    unsafe { ic0::msg_reply() };
}

#[derive(Debug)]
/// Config to control the behavior of decoding canister endpoint arguments.
pub struct ArgDecoderConfig {
    /// Limit the total amount of work the deserializer can perform. See [docs on the Candid library](https://docs.rs/candid/latest/candid/de/struct.DecoderConfig.html#method.set_decoding_quota) to understand the cost model.
    pub decoding_quota: Option<usize>,
    /// Limit the total amount of work for skipping unneeded data on the wire. See [docs on the Candid library](https://docs.rs/candid/latest/candid/de/struct.DecoderConfig.html#method.set_skipping_quota) to understand the skipping cost.
    pub skipping_quota: Option<usize>,
    /// When set to true, print instruction count and the decoding/skipping cost to the replica log.
    pub debug: bool,
}
impl ArgDecoderConfig {
    fn to_candid_config(&self) -> DecoderConfig {
        let mut config = DecoderConfig::new();
        if let Some(n) = self.decoding_quota {
            config.set_decoding_quota(n);
        }
        if let Some(n) = self.skipping_quota {
            config.set_skipping_quota(n);
        }
        if self.debug {
            config.set_full_error_message(true);
        }
        config
    }
}
impl Default for ArgDecoderConfig {
    fn default() -> Self {
        Self {
            decoding_quota: None,
            skipping_quota: Some(10_000),
            debug: false,
        }
    }
}

/// Returns the argument data in the current call. Traps if the data cannot be
/// decoded.
pub fn arg_data<R: for<'a> ArgumentDecoder<'a>>(arg_config: ArgDecoderConfig) -> R {
    let bytes = arg_data_raw();

    let config = arg_config.to_candid_config();
    let res = decode_args_with_config_debug(&bytes, &config);
    match res {
        Err(e) => trap(&format!("failed to decode call arguments: {:?}", e)),
        Ok((r, cost)) => {
            if arg_config.debug {
                print_decoding_debug_info("Argument", &cost, None);
            }
            r
        }
    }
}

/// Accepts the ingress message.
pub fn accept_message() {
    // SAFETY: ic0.accept_message is always safe to call.
    unsafe {
        ic0::accept_message();
    }
}

/// Returns the name of current canister method.
pub fn method_name() -> String {
    // SAFETY: ic0.msg_method_name_size is always safe to call.
    let len: u32 = unsafe { ic0::msg_method_name_size() as u32 };
    let mut bytes = vec![0u8; len as usize];
    // SAFETY: `bytes` is writable and allocated to `len` bytes, and therefore can be safely passed to ic0.msg_method_name_copy
    unsafe {
        ic0::msg_method_name_copy(bytes.as_mut_ptr() as i32, 0, len as i32);
    }
    String::from_utf8_lossy(&bytes).into_owned()
}

/// Gets the value of specified performance counter
///
/// See [`crate::api::performance_counter`].
#[deprecated(
    since = "0.11.3",
    note = "This method conceptually doesn't belong to this module. Please use `ic_cdk::api::performance_counter` instead."
)]
pub fn performance_counter(counter_type: u32) -> u64 {
    // SAFETY: ic0.performance_counter is always safe to call.
    unsafe { ic0::performance_counter(counter_type as i32) as u64 }
}

/// Pretends to have the Candid type `T`, but unconditionally errors
/// when serialized.
///
/// Usable, but not required, as metadata when using `#[query(manual_reply = true)]`,
/// so an accurate Candid file can still be generated.
#[derive(Debug, Copy, Clone, Default)]
pub struct ManualReply<T: ?Sized>(PhantomData<T>);

impl<T: ?Sized> ManualReply<T> {
    /// Constructs a new `ManualReply`.
    #[allow(clippy::self_named_constructors)]
    pub const fn empty() -> Self {
        Self(PhantomData)
    }
    /// Replies with the given value and returns a new `ManualReply`,
    /// for a useful reply-then-return shortcut.
    pub fn all<U>(value: U) -> Self
    where
        U: ArgumentEncoder,
    {
        reply(value);
        Self::empty()
    }
    /// Replies with a one-element tuple around the given value and returns
    /// a new `ManualReply`, for a useful reply-then-return shortcut.
    pub fn one<U>(value: U) -> Self
    where
        U: CandidType,
    {
        reply((value,));
        Self::empty()
    }

    /// Rejects the call with the specified message and returns a new
    /// `ManualReply`, for a useful reply-then-return shortcut.
    pub fn reject(message: impl AsRef<str>) -> Self {
        reject(message.as_ref());
        Self::empty()
    }
}

impl<T> CandidType for ManualReply<T>
where
    T: CandidType + ?Sized,
{
    fn _ty() -> candid::types::Type {
        T::_ty()
    }
    /// Unconditionally errors.
    fn idl_serialize<S>(&self, _: S) -> Result<(), S::Error>
    where
        S: candid::types::Serializer,
    {
        Err(S::Error::custom("`Empty` cannot be serialized"))
    }
}

/// Tells you whether the current async fn is being canceled due to a trap/panic.
///
/// If a function traps/panics, then the canister state is rewound to the beginning of the function.
/// However, due to the way async works, the beginning of the function as the IC understands it is actually
/// the most recent `await` from an inter-canister-call. This means that part of the function will have executed,
/// and part of it won't.
///
/// When this happens the CDK will cancel the task, causing destructors to be run. If you need any functions to be run
/// no matter what happens, they should happen in a destructor; the [`scopeguard`](https://docs.rs/scopeguard) crate
/// provides a convenient wrapper for this. In a destructor, `is_recovering_from_trap` serves the same purpose as
/// [std::thread::panicking] - it tells you whether the destructor is executing *because* of a trap,
/// as opposed to just because the scope was exited, so you could e.g. implement mutex poisoning.
pub fn is_recovering_from_trap() -> bool {
    ic_cdk_executor::CLEANUP.load(Ordering::Relaxed)
}
