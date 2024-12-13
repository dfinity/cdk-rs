//! APIs to make and manage calls in the canister.
use crate::api::{msg_arg_data, msg_reject_code, msg_reject_msg};
use candid::utils::{decode_args_with_config_debug, ArgumentDecoder, ArgumentEncoder};
use candid::{decode_args, encode_args, CandidType, Deserialize, Principal};
use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::Ordering;
use std::sync::{Arc, RwLock, Weak};
use std::task::{Context, Poll, Waker};

/// Reject code explains why the inter-canister call is rejected.
///
/// See [here](https://internetcomputer.org/docs/current/references/ic-interface-spec/#reject-codes) for more details.
#[repr(u32)]
#[non_exhaustive]
#[derive(CandidType, Deserialize, Clone, Copy, Hash, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum RejectCode {
    /// Fatal system error, retry unlikely to be useful.
    SysFatal = 1,
    /// Transient system error, retry might be possible.
    SysTransient = 2,
    /// Invalid destination (e.g. canister/account does not exist).
    DestinationInvalid = 3,
    /// Explicit reject by the canister.
    CanisterReject = 4,
    /// Canister error (e.g., trap, no response).
    CanisterError = 5,
    /// Response unknown; system stopped waiting for it (e.g., timed out, or system under high load).
    SysUnknown = 6,

    /// Unrecognized reject code.
    ///
    /// Note that this variant is not part of the IC interface spec, and is used to represent
    /// reject codes that are not recognized by the library.
    Unrecognized,
}

impl TryFrom<u32> for RejectCode {
    type Error = ();
    fn try_from(code: u32) -> Result<Self, Self::Error> {
        match code {
            // 0 is a special code meaning "no error"
            0 => Err(()),
            1 => Ok(RejectCode::SysFatal),
            2 => Ok(RejectCode::SysTransient),
            3 => Ok(RejectCode::DestinationInvalid),
            4 => Ok(RejectCode::CanisterReject),
            5 => Ok(RejectCode::CanisterError),
            6 => Ok(RejectCode::SysUnknown),
            _ => Ok(RejectCode::Unrecognized),
        }
    }
}

/// Error codes from the ic0.call_perform system API.
///
/// See [here](https://internetcomputer.org/docs/current/references/ic-interface-spec/#system-api-call) for more details.
///
/// So far, the specified codes (1, 2, 3) share the same meaning as the corresponding [RejectCode]s.
#[repr(u32)]
#[non_exhaustive]
#[derive(CandidType, Deserialize, Clone, Copy, Hash, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum CallPerformErrorCode {
    /// Fatal system error, retry unlikely to be useful.
    SysFatal = 1,
    /// Transient system error, retry might be possible.
    SysTransient = 2,
    /// Invalid destination (e.g. canister/account does not exist).
    DestinationInvalid = 3,

    /// Unrecognized error code.
    ///
    /// Note that this variant is not part of the IC interface spec, and is used to represent
    /// rejection codes that are not recognized by the library.
    Unrecognized,
}

impl TryFrom<u32> for CallPerformErrorCode {
    type Error = ();
    fn try_from(code: u32) -> Result<Self, Self::Error> {
        match code {
            // 0 is a special error code meaning call_perform succeeded.
            0 => Err(()),
            // all other non-zero codes are currently unrecognized errors.
            _ => Ok(CallPerformErrorCode::Unrecognized),
        }
    }
}

/// The error type for inter-canister calls.
#[derive(thiserror::Error, Debug, Clone)]
pub enum CallError {
    /// The arguments could not be encoded.
    ///
    /// This can only happen when the arguments are provided using [Call::with_args].
    /// Though the type system guarantees that the arguments are valid Candid types,
    /// it is possible that the encoding fails for reasons such as memory allocation failure.
    #[error("Failed to encode the arguments: {0}")]
    CandidEncodeFailed(String),

    /// The call immediately failed when invoking the call_perform system API.
    #[error("The IC was not able to enqueue the call with code {0:?}")]
    CallPerformFailed(CallPerformErrorCode),

    /// The call was rejected.
    ///
    /// Please handle the error by matching on the rejection code.
    #[error("The call was rejected with code {0:?} and message: {1}")]
    CallRejected(RejectCode, String),

    /// The response could not be decoded.
    ///
    /// This can only happen when making the call using [call][SendableCall::call]
    /// or [call_with_decoder_config][SendableCall::call_with_decoder_config].
    /// Because they decode the response to a Candid type.
    #[error("Failed to decode the response as {0}")]
    CandidDecodeFailed(String),
}

/// Result of a inter-canister call.
pub type CallResult<R> = Result<R, CallError>;

/// Inter-Canister Call.
#[derive(Debug)]
pub struct Call<'a> {
    canister_id: Principal,
    method: &'a str,
    cycles: Option<u128>,
    timeout_seconds: Option<u32>,
}

/// Inter-Canister Call with typed arguments.
#[derive(Debug)]
pub struct CallWithArgs<'a, T> {
    call: Call<'a>,
    args: T,
}

/// Inter-Canister Call with raw arguments.
#[derive(Debug)]
pub struct CallWithRawArgs<'a, A> {
    call: Call<'a>,
    raw_args: A,
}

impl<'a> Call<'a> {
    /// Constructs a new call with the Canister id and method name.
    ///
    /// # Note
    /// The `Call` default to set a 10 seconds timeout for Best-Effort Responses.
    /// If you want to set a guaranteed response, you can use the `with_guaranteed_response` method.
    pub fn new(canister_id: Principal, method: &'a str) -> Self {
        Self {
            canister_id,
            method,
            cycles: None,
            // Default to 10 seconds.
            timeout_seconds: Some(10),
        }
    }

    /// Sets the arguments for the call.
    ///
    /// Another way to set the arguments is to use `with_raw_args`.
    /// If both are invoked, the last one is used.
    pub fn with_args<T>(self, args: T) -> CallWithArgs<'a, T> {
        CallWithArgs { call: self, args }
    }

    /// Sets the arguments for the call as raw bytes.    
    ///
    /// Another way to set the arguments is to use `with_raw_args`.
    /// If both are invoked, the last one is used.
    pub fn with_raw_args<A>(self, raw_args: A) -> CallWithRawArgs<'a, A> {
        CallWithRawArgs {
            call: self,
            raw_args,
        }
    }
}

/// Methods to configure a call.
pub trait ConfigurableCall {
    /// Sets the cycles payment for the call.
    ///
    /// If invoked multiple times, the last value is used.
    fn with_cycles(self, cycles: u128) -> Self;

    /// Sets the call to have a guaranteed response.
    ///
    /// If [change_timeout](ConfigurableCall::change_timeout) is invoked after this method,
    /// the call will instead be set with Best-Effort Responses.
    fn with_guaranteed_response(self) -> Self;

    /// Sets the timeout for the Best-Effort Responses.
    ///
    /// If not set, the call will default to a 10 seconds timeout.
    /// If invoked multiple times, the last value is used.
    /// If [with_guaranteed_response](ConfigurableCall::with_guaranteed_response) is invoked after this method,
    /// the timeout will be ignored.
    fn change_timeout(self, timeout_seconds: u32) -> Self;
}

impl<'a> ConfigurableCall for Call<'a> {
    fn with_cycles(mut self, cycles: u128) -> Self {
        self.cycles = Some(cycles);
        self
    }

    fn with_guaranteed_response(mut self) -> Self {
        self.timeout_seconds = None;
        self
    }

    fn change_timeout(mut self, timeout_seconds: u32) -> Self {
        self.timeout_seconds = Some(timeout_seconds);
        self
    }
}

impl<'a, T> ConfigurableCall for CallWithArgs<'a, T> {
    fn with_cycles(mut self, cycles: u128) -> Self {
        self.call.cycles = Some(cycles);
        self
    }

    fn with_guaranteed_response(mut self) -> Self {
        self.call.timeout_seconds = None;
        self
    }

    fn change_timeout(mut self, timeout_seconds: u32) -> Self {
        self.call.timeout_seconds = Some(timeout_seconds);
        self
    }
}

impl<'a, A> ConfigurableCall for CallWithRawArgs<'a, A> {
    fn with_cycles(mut self, cycles: u128) -> Self {
        self.call.cycles = Some(cycles);
        self
    }

    fn with_guaranteed_response(mut self) -> Self {
        self.call.timeout_seconds = None;
        self
    }

    fn change_timeout(mut self, timeout_seconds: u32) -> Self {
        self.call.timeout_seconds = Some(timeout_seconds);
        self
    }
}

/// Methods to send a call.
pub trait SendableCall {
    /// Sends the call and gets the reply as raw bytes.
    fn call_raw(self) -> impl Future<Output = CallResult<Vec<u8>>> + Send + Sync;

    /// Sends the call and decodes the reply to a Candid type.
    fn call<R: for<'b> ArgumentDecoder<'b>>(
        self,
    ) -> impl Future<Output = CallResult<R>> + Send + Sync
    where
        Self: Sized,
    {
        let fut = self.call_raw();
        async {
            let bytes = fut.await?;
            decode_args(&bytes).map_err(decoder_error_to_call_error::<R>)
        }
    }

    /// Sends the call and decodes the reply to a Candid type with a decoding quota.
    fn call_with_decoder_config<R: for<'b> ArgumentDecoder<'b>>(
        self,
        decoder_config: &DecoderConfig,
    ) -> impl Future<Output = CallResult<R>> + Send + Sync
    where
        Self: Sized,
    {
        let fut = self.call_raw();
        async move {
            let bytes = fut.await?;
            let config = decoder_config.to_candid_config();
            let pre_cycles = if decoder_config.debug {
                Some(crate::api::performance_counter(0))
            } else {
                None
            };
            match decode_args_with_config_debug(&bytes, &config) {
                Err(e) => Err(decoder_error_to_call_error::<R>(e)),
                Ok((r, cost)) => {
                    if decoder_config.debug {
                        print_decoding_debug_info(std::any::type_name::<R>(), &cost, pre_cycles);
                    }
                    Ok(r)
                }
            }
        }
    }

    /// Sends the call and ignores the reply.
    fn call_and_forget(self) -> CallResult<()>;
}

impl SendableCall for Call<'_> {
    fn call_raw(self) -> impl Future<Output = CallResult<Vec<u8>>> + Send + Sync {
        let args_raw = vec![0x44, 0x49, 0x44, 0x4c, 0x00, 0x00];
        call_raw_internal::<Vec<u8>>(
            self.canister_id,
            self.method,
            args_raw,
            self.cycles,
            self.timeout_seconds,
        )
    }

    fn call_and_forget(self) -> CallResult<()> {
        let args_raw = vec![0x44, 0x49, 0x44, 0x4c, 0x00, 0x00];
        call_and_forget_internal::<Vec<u8>>(
            self.canister_id,
            self.method,
            args_raw,
            self.cycles,
            self.timeout_seconds,
        )
    }
}

impl<'a, T: ArgumentEncoder + Send + Sync> SendableCall for CallWithArgs<'a, T> {
    async fn call_raw(self) -> CallResult<Vec<u8>> {
        let args_raw = encode_args(self.args).map_err(encoder_error_to_call_error::<T>)?;
        call_raw_internal(
            self.call.canister_id,
            self.call.method,
            args_raw,
            self.call.cycles,
            self.call.timeout_seconds,
        )
        .await
    }

    fn call_and_forget(self) -> CallResult<()> {
        let args_raw = encode_args(self.args).map_err(encoder_error_to_call_error::<T>)?;
        call_and_forget_internal(
            self.call.canister_id,
            self.call.method,
            args_raw,
            self.call.cycles,
            self.call.timeout_seconds,
        )
    }
}

impl<'a, A: AsRef<[u8]> + Send + Sync + 'a> SendableCall for CallWithRawArgs<'a, A> {
    fn call_raw(self) -> impl Future<Output = CallResult<Vec<u8>>> + Send + Sync {
        call_raw_internal(
            self.call.canister_id,
            self.call.method,
            self.raw_args,
            self.call.cycles,
            self.call.timeout_seconds,
        )
    }

    fn call_and_forget(self) -> CallResult<()> {
        call_and_forget_internal(
            self.call.canister_id,
            self.call.method,
            self.raw_args,
            self.call.cycles,
            self.call.timeout_seconds,
        )
    }
}

// # Internal =================================================================

// Internal state for the Future when sending a call.
struct CallFutureState<T: AsRef<[u8]>> {
    result: Option<CallResult<Vec<u8>>>,
    waker: Option<Waker>,
    id: Principal,
    method: String,
    arg: T,
    cycles: Option<u128>,
    timeout_seconds: Option<u32>,
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
                let state_ptr = Weak::into_raw(Arc::downgrade(&self_ref.state));
                // SAFETY:
                // `callee`, being &[u8], is a readable sequence of bytes and therefore can be passed to ic0.call_new.
                // `method`, being &str, is a readable sequence of bytes and therefore can be passed to ic0.call_new.
                // `callback` is a function with signature (env : usize) -> () and therefore can be called as both reply and reject fn for ic0.call_new.
                // `state_ptr` is a pointer created via Weak::into_raw, and can therefore be passed as the userdata for `callback`.
                // `args`, being a &[u8], is a readable sequence of bytes and therefore can be passed to ic0.call_data_append.
                // `cleanup` is a function with signature (env : usize) -> () and therefore can be called as a cleanup fn for ic0.call_on_cleanup.
                // `state_ptr` is a pointer created via Weak::into_raw, and can therefore be passed as the userdata for `cleanup`.
                // ic0.call_perform is always safe to call.
                // callback and cleanup are safe to parameterize with T because:
                // - if the future is dropped before the callback is called, there will be no more strong references and the weak reference will fail to upgrade
                // - if the future is *not* dropped before the callback is called, the compiler will mandate that any data borrowed by T is still alive
                let err_code = unsafe {
                    ic0::call_new(
                        callee.as_ptr() as usize,
                        callee.len(),
                        method.as_ptr() as usize,
                        method.len(),
                        callback::<T> as usize,
                        state_ptr as usize,
                        callback::<T> as usize,
                        state_ptr as usize,
                    );
                    let arg = state.arg.as_ref();
                    if !arg.is_empty() {
                        ic0::call_data_append(arg.as_ptr() as usize, arg.len());
                    }
                    if let Some(cycles) = state.cycles {
                        call_cycles_add(cycles);
                    }
                    if let Some(timeout_seconds) = state.timeout_seconds {
                        ic0::call_with_best_effort_response(timeout_seconds);
                    }
                    ic0::call_on_cleanup(cleanup::<T> as usize, state_ptr as usize);
                    ic0::call_perform()
                };

                // The conversion fails only when the err_code is 0, which means the call was successfully enqueued.
                if let Ok(c) = CallPerformErrorCode::try_from(err_code) {
                    let result = Err(CallError::CallPerformFailed(c));
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
            state.write().unwrap().result = Some(match RejectCode::try_from(msg_reject_code()) {
                Err(_) => Ok(msg_arg_data()),
                Ok(n) => Err(CallError::CallRejected(n, msg_reject_msg())),
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
        let w = state.write().unwrap().waker.take();
        if let Some(waker) = w {
            // Flag that we do not want to actually wake the task - we
            // want to drop it *without* executing it.
            crate::futures::CLEANUP.store(true, Ordering::Relaxed);
            waker.wake();
            crate::futures::CLEANUP.store(false, Ordering::Relaxed);
        }
    }
}

fn call_raw_internal<'a, T: AsRef<[u8]> + Send + Sync + 'a>(
    id: Principal,
    method: &str,
    args_raw: T,
    cycles: Option<u128>,
    timeout_seconds: Option<u32>,
) -> impl Future<Output = CallResult<Vec<u8>>> + Send + Sync + 'a {
    let state = Arc::new(RwLock::new(CallFutureState {
        result: None,
        waker: None,
        id,
        method: method.to_string(),
        arg: args_raw,
        cycles,
        timeout_seconds,
    }));
    CallFuture { state }
}

fn call_and_forget_internal<T: AsRef<[u8]>>(
    id: Principal,
    method: &str,
    args_raw: T,
    cycles: Option<u128>,
    timeout_seconds: Option<u32>,
) -> CallResult<()> {
    let callee = id.as_slice();
    // We set all callbacks to usize::MAX, which is guaranteed to be invalid callback index.
    // The system will still deliver the reply, but it will trap immediately because the callback
    // is not a valid function. See
    // https://www.joachim-breitner.de/blog/789-Zero-downtime_upgrades_of_Internet_Computer_canisters#one-way-calls
    // for more context.

    // SAFETY:
    // ic0.call_new:
    //   `callee_src` and `callee_size`: `callee` being &[u8], is a readable sequence of bytes.
    //   `name_src` and `name_size`: `method`, being &str, is a readable sequence of bytes.
    //   `reply_fun` and `reject_fun`: In "notify" style call, we want these callback functions to not be called. So pass `usize::MAX` which is a function pointer the wasm module cannot possibly contain.
    //   `reply_env` and `reject_env`: Since the callback functions will never be called, any value can be passed as its context parameter.
    // ic0.call_data_append:
    //   `args`, being a &[u8], is a readable sequence of bytes.
    // ic0.call_with_best_effort_response is always safe to call.
    // ic0.call_perform is always safe to call.
    let err_code = unsafe {
        ic0::call_new(
            callee.as_ptr() as usize,
            callee.len(),
            method.as_ptr() as usize,
            method.len(),
            usize::MAX,
            usize::MAX,
            usize::MAX,
            usize::MAX,
        );
        let arg = args_raw.as_ref();
        if !arg.is_empty() {
            ic0::call_data_append(arg.as_ptr() as usize, arg.len());
        }
        if let Some(cycles) = cycles {
            call_cycles_add(cycles);
        }
        if let Some(timeout_seconds) = timeout_seconds {
            ic0::call_with_best_effort_response(timeout_seconds);
        }
        ic0::call_perform()
    };
    // The conversion fails only when the err_code is 0, which means the call was successfully enqueued.
    match CallPerformErrorCode::try_from(err_code) {
        Ok(c) => Err(CallError::CallPerformFailed(c)),
        Err(_) => Ok(()),
    }
}

// # Internal END =============================================================

fn call_cycles_add(cycles: u128) {
    if cycles == 0 {
        return;
    }
    let high = (cycles >> 64) as u64;
    let low = (cycles & u64::MAX as u128) as u64;
    // SAFETY: ic0.call_cycles_add128 is always safe to call.
    unsafe {
        ic0::call_cycles_add128(high, low);
    }
}

fn print_decoding_debug_info(
    title: &str,
    cost: &candid::de::DecoderConfig,
    pre_cycles: Option<u64>,
) {
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

#[derive(Debug)]
/// Config to control the behavior of decoding canister endpoint arguments.
pub struct DecoderConfig {
    /// Limit the total amount of work the deserializer can perform. See [docs on the Candid library](https://docs.rs/candid/latest/candid/de/struct.DecoderConfig.html#method.set_decoding_quota) to understand the cost model.
    pub decoding_quota: Option<usize>,
    /// Limit the total amount of work for skipping unneeded data on the wire. See [docs on the Candid library](https://docs.rs/candid/latest/candid/de/struct.DecoderConfig.html#method.set_skipping_quota) to understand the skipping cost.
    pub skipping_quota: Option<usize>,
    /// When set to true, print instruction count and the decoding/skipping cost to the replica log.
    pub debug: bool,
}

impl DecoderConfig {
    fn to_candid_config(&self) -> candid::de::DecoderConfig {
        let mut config = candid::de::DecoderConfig::new();
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

impl Default for DecoderConfig {
    fn default() -> Self {
        Self {
            decoding_quota: None,
            skipping_quota: Some(10_000),
            debug: false,
        }
    }
}

/// Converts a decoder error to a CallError.
fn decoder_error_to_call_error<T>(err: candid::error::Error) -> CallError {
    CallError::CandidDecodeFailed(format!("{}: {}", std::any::type_name::<T>(), err))
}

/// Converts a encoder error to a CallError.
fn encoder_error_to_call_error<T>(err: candid::error::Error) -> CallError {
    CallError::CandidEncodeFailed(format!("{}: {}", std::any::type_name::<T>(), err))
}
