//! APIs to make and manage calls in the canister.
use crate::api::{msg_arg_data, msg_reject_code, msg_reject_msg};
use candid::utils::{encode_args_ref, ArgumentDecoder, ArgumentEncoder};
use candid::{decode_args, decode_one, encode_one, CandidType, Deserialize, Principal};
use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::Ordering;
use std::sync::{Arc, RwLock, Weak};
use std::task::{Context, Poll, Waker};

pub use ic_response_codes::RejectCode;

const CALL_PERFORM_REJECT_MESSAGE: &str = "call_perform failed";

/// The error type for inter-canister calls and decoding the response.
#[derive(thiserror::Error, Debug, Clone)]
pub enum CallError {
    /// The call was rejected.
    ///
    /// Please handle the error by matching on the rejection code.
    #[error("The call was rejected with code {0:?}")]
    CallRejected(CallRejected),

    /// The response could not be decoded.
    ///
    /// This can only happen when making the call using [`call`][SendableCall::call]
    /// or [`call_tuple`][SendableCall::call_tuple].
    /// Because they decode the response to a Candid type.
    #[error("Failed to decode the response as {0}")]
    CandidDecodeFailed(String),
}

/// The error type for inter-canister calls.
#[derive(Debug, Clone)]
pub struct CallRejected {
    // All fields are private so we will be able to change the implementation without breaking the API.
    // Once we have `ic0.msg_error_code` system API, we will only store the error_code in this struct.
    // It will still be possible to get the [`RejectCode`] using the public getter,
    // because every error_code can map to a [`RejectCode`].
    reject_code: RejectCode,
    reject_message: String,
    sync: bool,
}

impl CallRejected {
    /// Returns the [`RejectCode`].
    pub fn reject_code(&self) -> RejectCode {
        self.reject_code
    }

    /// Returns the reject message.
    ///
    /// When the call was rejected asynchronously (IC rejects the call after it was enqueued),
    /// this message is get from [`msg_reject_msg`].
    ///
    /// When the call was rejected synchronously (`ic0.call_preform` returns non-zero code),
    /// this message is set to a fixed string ("call_perform failed").
    pub fn reject_message(&self) -> &str {
        &self.reject_message
    }

    /// Returns whether the call was rejected synchronously (`ic0.call_perform` returned non-zero code)
    /// or asynchronously (IC rejects the call after it was enqueued).
    pub fn is_sync(&self) -> bool {
        self.sync
    }
}

/// Result of a inter-canister call.
pub type SystemResult<R> = Result<R, CallRejected>;

/// Result of a inter-canister call and decoding the response.
pub type CallResult<R> = Result<R, CallError>;

/// Inter-Canister Call.
///
/// # Note
///
/// The [`Call`] defaults to a 10-second timeout for Best-Effort Responses.
/// To change the timeout, use the [`change_timeout`][ConfigurableCall::change_timeout] method.
/// To get a guaranteed response, use the [`with_guaranteed_response`][ConfigurableCall::with_guaranteed_response] method.
#[derive(Debug)]
pub struct Call<'a> {
    canister_id: Principal,
    method: &'a str,
    cycles: Option<u128>,
    timeout_seconds: Option<u32>,
}

/// Inter-Canister Call with typed argument.
///
/// The argument must impl [`CandidType`].
#[derive(Debug)]
pub struct CallWithArg<'a, 'b, T> {
    call: Call<'a>,
    arg: &'b T,
}

/// Inter-Canister Call with typed arguments.
///
/// The arguments are a tuple of types, each implementing [`CandidType`].
#[derive(Debug)]
pub struct CallWithArgs<'a, 'b, T> {
    call: Call<'a>,
    args: &'b T,
}

/// Inter-Canister Call with raw arguments.
#[derive(Debug)]
pub struct CallWithRawArgs<'a, 'b, A> {
    call: Call<'a>,
    raw_args: &'b A,
}

impl<'a> Call<'a> {
    /// Constructs a new call with the Canister id and method name.
    ///
    /// # Note
    ///
    /// The [`Call`] defaults to a 10-second timeout for Best-Effort Responses.
    /// To change the timeout, use the [`change_timeout`][ConfigurableCall::change_timeout] method.
    /// To get a guaranteed response, use the [`with_guaranteed_response`][ConfigurableCall::with_guaranteed_response] method.
    pub fn new(canister_id: Principal, method: &'a str) -> Self {
        Self {
            canister_id,
            method,
            cycles: None,
            // Default to 10 seconds.
            timeout_seconds: Some(10),
        }
    }

    /// Sets the argument for the call.
    ///
    /// The argument must implement [`CandidType`].
    pub fn with_arg<'b, T>(self, arg: &'b T) -> CallWithArg<'a, 'b, T> {
        CallWithArg { call: self, arg }
    }

    /// Sets the arguments for the call.
    ///
    /// The arguments are a tuple of types, each implementing [`CandidType`].
    pub fn with_args<'b, T>(self, args: &'b T) -> CallWithArgs<'a, 'b, T> {
        CallWithArgs { call: self, args }
    }

    /// Sets the arguments for the call as raw bytes.
    pub fn with_raw_args<'b, A>(self, raw_args: &'b A) -> CallWithRawArgs<'a, 'b, A> {
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
    /// If invoked multiple times, the last value takes effect.
    fn with_cycles(self, cycles: u128) -> Self;

    /// Sets the call to have a guaranteed response.
    ///
    /// If [`change_timeout`](ConfigurableCall::change_timeout) is invoked after this method,
    /// the call will instead be set with Best-Effort Responses.
    fn with_guaranteed_response(self) -> Self;

    /// Sets the timeout for the Best-Effort Responses.
    ///
    /// If not set, the call defaults to a 10-second timeout.
    /// If invoked multiple times, the last value takes effect.
    /// If [`with_guaranteed_response`](ConfigurableCall::with_guaranteed_response) is invoked after this method,
    /// the timeout will be ignored.
    ///
    /// # Note
    ///
    /// A timeout of 0 second DOES NOT mean guranteed response.
    /// The call would most likely time out (result in a `SysUnknown` reject).
    /// Unless it's a call to the canister on the same subnet,
    /// and the execution manages to schedule both the request and the response in the same round.
    ///
    /// To make the call with a guaranteed response,
    /// use the [`with_guaranteed_response`](ConfigurableCall::with_guaranteed_response) method.
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

impl<'a, 'b, T> ConfigurableCall for CallWithArg<'a, 'b, T> {
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

impl<'a, 'b, T> ConfigurableCall for CallWithArgs<'a, 'b, T> {
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

impl<'a, 'b, A> ConfigurableCall for CallWithRawArgs<'a, 'b, A> {
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
    fn call_raw(&self) -> impl Future<Output = SystemResult<Vec<u8>>> + Send + Sync;

    /// Sends the call and decodes the reply to a Candid type.
    fn call<R>(&self) -> impl Future<Output = CallResult<R>> + Send + Sync
    where
        Self: Sized,
        R: CandidType + for<'b> Deserialize<'b>,
    {
        let fut = self.call_raw();
        async {
            let bytes = fut.await.map_err(CallError::CallRejected)?;
            decode_one(&bytes).map_err(decoder_error_to_call_error::<R>)
        }
    }

    /// Sends the call and decodes the reply to a Candid type.
    fn call_tuple<R>(&self) -> impl Future<Output = CallResult<R>> + Send + Sync
    where
        Self: Sized,
        R: for<'b> ArgumentDecoder<'b>,
    {
        let fut = self.call_raw();
        async {
            let bytes = fut.await.map_err(CallError::CallRejected)?;
            decode_args(&bytes).map_err(decoder_error_to_call_error::<R>)
        }
    }

    /// Sends the call and ignores the reply.
    fn call_oneway(&self) -> SystemResult<()>;
}

impl SendableCall for Call<'_> {
    fn call_raw(&self) -> impl Future<Output = SystemResult<Vec<u8>>> + Send + Sync {
        let args_raw = vec![0x44, 0x49, 0x44, 0x4c, 0x00, 0x00];
        call_raw_internal::<Vec<u8>>(
            self.canister_id,
            self.method,
            args_raw,
            self.cycles,
            self.timeout_seconds,
        )
    }

    fn call_oneway(&self) -> SystemResult<()> {
        let args_raw = vec![0x44, 0x49, 0x44, 0x4c, 0x00, 0x00];
        call_oneway_internal::<Vec<u8>>(
            self.canister_id,
            self.method,
            args_raw,
            self.cycles,
            self.timeout_seconds,
        )
    }
}

impl<'a,'b, T: ArgumentEncoder + Send + Sync> SendableCall for CallWithArgs<'a,'b, T> {
    async fn call_raw(&self) -> SystemResult<Vec<u8>> {
        let args_raw = encode_args_ref(self.args).unwrap_or_else(panic_when_encode_fails);
        call_raw_internal(
            self.call.canister_id,
            self.call.method,
            args_raw,
            self.call.cycles,
            self.call.timeout_seconds,
        )
        .await
    }

    fn call_oneway(&self) -> SystemResult<()> {
        let args_raw = encode_args_ref(self.args).unwrap_or_else(panic_when_encode_fails);
        call_oneway_internal(
            self.call.canister_id,
            self.call.method,
            args_raw,
            self.call.cycles,
            self.call.timeout_seconds,
        )
    }
}

impl<'a, 'b, T: CandidType + Send + Sync> SendableCall for CallWithArg<'a, 'b, T> {
    async fn call_raw(&self) -> SystemResult<Vec<u8>> {
        let args_raw = encode_one(self.arg).unwrap_or_else(panic_when_encode_fails);
        call_raw_internal(
            self.call.canister_id,
            self.call.method,
            args_raw,
            self.call.cycles,
            self.call.timeout_seconds,
        )
        .await
    }

    fn call_oneway(&self) -> SystemResult<()> {
        let args_raw = encode_one(self.arg).unwrap_or_else(panic_when_encode_fails);
        call_oneway_internal(
            self.call.canister_id,
            self.call.method,
            args_raw,
            self.call.cycles,
            self.call.timeout_seconds,
        )
    }
}

impl<'a, 'b, A: AsRef<[u8]> + Send + Sync + 'b> SendableCall for CallWithRawArgs<'a, 'b, A> {
    fn call_raw(&self) -> impl Future<Output = SystemResult<Vec<u8>>> + Send + Sync {
        call_raw_internal(
            self.call.canister_id,
            self.call.method,
            self.raw_args,
            self.call.cycles,
            self.call.timeout_seconds,
        )
    }

    fn call_oneway(&self) -> SystemResult<()> {
        call_oneway_internal(
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
    result: Option<SystemResult<Vec<u8>>>,
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
    type Output = SystemResult<Vec<u8>>;

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
                let code = unsafe {
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

                match code {
                    0 => {
                        // call_perform returns 0 means the call was successfully enqueued.
                    }
                    _ => {
                        // SAFETY: The conversion is safe because the code is not 0.
                        let reject_code = RejectCode::try_from(code).unwrap();
                        let result = Err(CallRejected {
                            reject_code,
                            reject_message: CALL_PERFORM_REJECT_MESSAGE.to_string(),
                            sync: true,
                        });
                        state.result = Some(result.clone());
                        return Poll::Ready(result);
                    }
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
            state.write().unwrap().result = Some(match msg_reject_code() {
                0 => Ok(msg_arg_data()),
                code => {
                    // SAFETY: The conversion is safe because the code is not 0.
                    let reject_code = RejectCode::try_from(code).unwrap();
                    Err(CallRejected {
                        reject_code,
                        reject_message: msg_reject_msg(),
                        sync: false,
                    })
                }
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

/// This function is called when [`callback`] was just called with the same parameter, and trapped.
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
) -> impl Future<Output = SystemResult<Vec<u8>>> + Send + Sync + 'a {
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

fn call_oneway_internal<T: AsRef<[u8]>>(
    id: Principal,
    method: &str,
    args_raw: T,
    cycles: Option<u128>,
    timeout_seconds: Option<u32>,
) -> SystemResult<()> {
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
    let code = unsafe {
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
    match code {
        0 => Ok(()),
        _ => {
            // SAFETY: The conversion is safe because the code is not 0.
            let reject_code = RejectCode::try_from(code).unwrap();
            Err(CallRejected {
                reject_code,
                reject_message: CALL_PERFORM_REJECT_MESSAGE.to_string(),
                sync: true,
            })
        }
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

/// Converts a decoder error to a CallError.
fn decoder_error_to_call_error<T>(err: candid::error::Error) -> CallError {
    CallError::CandidDecodeFailed(format!("{}: {}", std::any::type_name::<T>(), err))
}

/// When args encoding fails, we panic with an informative message.
///
/// Currently, Candid encoding only fails when heap memory is exhausted,
/// in which case execution would trap before reaching the unwrap.
///
/// However, since future implementations might introduce other failure cases,
/// we provide an informative panic message for better debuggability.
fn panic_when_encode_fails(err: candid::error::Error) -> Vec<u8> {
    panic!("failed to encode args: {}", err)
}
