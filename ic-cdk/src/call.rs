//! APIs to make and manage calls in the canister.
use crate::api::{msg_arg_data, msg_reject_msg};
use candid::utils::{decode_args_with_config_debug, ArgumentDecoder, ArgumentEncoder};
use candid::{decode_args, encode_args, CandidType, DecoderConfig, Deserialize, Principal};
use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::Ordering;
use std::sync::{Arc, RwLock, Weak};
use std::task::{Context, Poll, Waker};

/// Rejection code from calling another canister.
///
/// These can be obtained using [reject_code].
#[repr(i32)]
#[derive(CandidType, Deserialize, Clone, Copy, Hash, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum RejectionCode {
    /// No error.
    NoError = 0,

    /// Fatal system error, retry unlikely to be useful.
    SysFatal = 1,
    /// Transient system error, retry might be possible.
    SysTransient = 2,
    /// Invalid destination (e.g. canister/account does not exist)
    DestinationInvalid = 3,
    /// Explicit reject by the canister.
    CanisterReject = 4,
    /// Canister error (e.g., trap, no response)
    CanisterError = 5,
    /// Response unknown; system stopped waiting for it (e.g., timed out, or system under high load).
    SysUnknown = 6,

    /// Unknown rejection code.
    ///
    /// Note that this variant is not part of the IC interface spec, and is used to represent
    /// rejection codes that are not recognized by the library.
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
            6 => RejectionCode::SysUnknown,
            _ => RejectionCode::Unknown,
        }
    }
}

impl From<u32> for RejectionCode {
    fn from(code: u32) -> Self {
        RejectionCode::from(code as i32)
    }
}

/// The error type for inter-canister calls.
#[derive(thiserror::Error, Debug, Clone)]
pub enum CallError {
    /// The call immediately failed when invoking the call_perform system API.
    #[error("The IC was not able to enqueue the call")]
    PerformFailed,

    /// The call was rejected.
    ///
    /// Please handle the error by matching on the rejection code.
    #[error("The call was rejected with code {0:?} and message: {1}")]
    CallRejected(RejectionCode, String),

    /// The cleanup callback was executed.
    //
    // TODO: Is this really an error?
    #[error("The cleanup callback was executed")]
    CleanupExecuted,

    /// The response could not be decoded.
    #[error("Failed to decode the response as {0}")]
    CandidDecodeFailed(String),
}

/// The result of a Call.
///
/// Errors on the IC have two components; a Code and a message associated with it.
pub type CallResult<R> = Result<R, CallError>;

// Internal state for the Future when sending a call.
struct CallFutureState<T: AsRef<[u8]>> {
    result: Option<CallResult<Vec<u8>>>,
    waker: Option<Waker>,
    id: Principal,
    method: String,
    arg: Option<T>,
    payment: Option<u128>,
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
                    if let Some(args) = &state.arg {
                        ic0::call_data_append(args.as_ref().as_ptr() as usize, args.as_ref().len());
                    }
                    if let Some(payment) = state.payment {
                        add_payment(payment);
                    }
                    if let Some(timeout_seconds) = state.timeout_seconds {
                        ic0::call_with_best_effort_response(timeout_seconds);
                    }
                    ic0::call_on_cleanup(cleanup::<T> as usize, state_ptr as usize);
                    ic0::call_perform()
                };

                // 0 is a special error code meaning call succeeded.
                if err_code != 0 {
                    let result = Err(CallError::PerformFailed);
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
                RejectionCode::NoError => Ok(msg_arg_data()),
                n => Err(CallError::CallRejected(n, msg_reject_msg())),
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
        // TODO: Should we have this?
        state.write().unwrap().result = Some(Err(CallError::CleanupExecuted));
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

/// Inter-Canister Call.
#[derive(Debug)]
pub struct Call<'a> {
    canister_id: Principal,
    method: &'a str,
    payment: Option<u128>,
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
            payment: None,
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
        self.payment = Some(cycles);
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
        self.call.payment = Some(cycles);
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
        self.call.payment = Some(cycles);
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
        decoder_config: &ArgDecoderConfig,
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
    fn call_and_forget(self) -> Result<(), RejectionCode>;
}

impl SendableCall for Call<'_> {
    fn call_raw(self) -> impl Future<Output = CallResult<Vec<u8>>> + Send + Sync {
        call_raw_internal::<Vec<u8>>(
            self.canister_id,
            self.method,
            None,
            self.payment,
            self.timeout_seconds,
        )
    }

    fn call_and_forget(self) -> Result<(), RejectionCode> {
        notify_raw_internal::<Vec<u8>>(
            self.canister_id,
            self.method,
            None,
            self.payment,
            self.timeout_seconds,
        )
    }
}

impl<'a, T: ArgumentEncoder> SendableCall for CallWithArgs<'a, T> {
    fn call_raw(self) -> impl Future<Output = CallResult<Vec<u8>>> + Send + Sync {
        let args = encode_args(self.args).expect("failed to encode arguments");
        call_raw_internal(
            self.call.canister_id,
            self.call.method,
            Some(args),
            self.call.payment,
            self.call.timeout_seconds,
        )
    }

    fn call_and_forget(self) -> Result<(), RejectionCode> {
        let args = encode_args(self.args).expect("failed to encode arguments");
        notify_raw_internal(
            self.call.canister_id,
            self.call.method,
            Some(args),
            self.call.payment,
            self.call.timeout_seconds,
        )
    }
}

impl<'a, A: AsRef<[u8]> + Send + Sync + 'a> SendableCall for CallWithRawArgs<'a, A> {
    fn call_raw(self) -> impl Future<Output = CallResult<Vec<u8>>> + Send + Sync {
        call_raw_internal(
            self.call.canister_id,
            self.call.method,
            Some(self.raw_args),
            self.call.payment,
            self.call.timeout_seconds,
        )
    }

    fn call_and_forget(self) -> Result<(), RejectionCode> {
        notify_raw_internal(
            self.call.canister_id,
            self.call.method,
            Some(self.raw_args),
            self.call.payment,
            self.call.timeout_seconds,
        )
    }
}

fn call_raw_internal<'a, T: AsRef<[u8]> + Send + Sync + 'a>(
    id: Principal,
    method: &str,
    args_raw: Option<T>,
    payment: Option<u128>,
    timeout_seconds: Option<u32>,
) -> impl Future<Output = CallResult<Vec<u8>>> + Send + Sync + 'a {
    let state = Arc::new(RwLock::new(CallFutureState {
        result: None,
        waker: None,
        id,
        method: method.to_string(),
        arg: args_raw,
        payment,
        timeout_seconds,
    }));
    CallFuture { state }
}

fn notify_raw_internal<T: AsRef<[u8]>>(
    id: Principal,
    method: &str,
    args_raw: Option<T>,
    payment: Option<u128>,
    timeout_seconds: Option<u32>,
) -> Result<(), RejectionCode> {
    let callee = id.as_slice();
    // We set all callbacks to -1, which is guaranteed to be invalid callback index.
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
        if let Some(args) = args_raw {
            ic0::call_data_append(args.as_ref().as_ptr() as usize, args.as_ref().len());
        }
        if let Some(payment) = payment {
            add_payment(payment);
        }
        if let Some(timeout_seconds) = timeout_seconds {
            ic0::call_with_best_effort_response(timeout_seconds);
        }
        ic0::call_perform()
    };
    match err_code {
        0 => Ok(()),
        c => Err(RejectionCode::from(c)),
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
        ic0::call_cycles_add128(high, low);
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

/// Returns the rejection code for the call.
fn reject_code() -> RejectionCode {
    // SAFETY: ic0.msg_reject_code is always safe to call.
    let code = unsafe { ic0::msg_reject_code() };
    RejectionCode::from(code)
}

/// Converts a decoder error to a CallError.
fn decoder_error_to_call_error<T>(err: candid::error::Error) -> CallError {
    CallError::CandidDecodeFailed(format!("{}: {}", std::any::type_name::<T>(), err))
}
