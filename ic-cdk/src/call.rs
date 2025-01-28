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
///
/// See [`Call::call`] or [`Call::call_tuple`].
#[derive(thiserror::Error, Debug, Clone)]
pub enum CallError {
    /// The call was rejected.
    ///
    /// Please handle the error by matching on the rejection code.
    #[error("The call was rejected with code {0:?}")]
    CallRejected(CallRejected),

    /// The response could not be decoded.
    #[error("Failed to decode the response as {0}")]
    CandidDecodeFailed(String),
}

/// The error type for inter-canister calls.
///
/// See [`Call::call_raw`] and [`Call::call_oneway`].
///
/// This type is also the inner error type of [`CallError::CallRejected`].
#[derive(Debug, Clone)]
pub struct CallRejected {
    /// All fields are private so we will be able to change the implementation without breaking the API.
    /// Once we have `ic0.msg_error_code` system API, we will only store the `error_code` in this struct.
    /// It will still be possible to get the [`RejectCode`] using the public getter,
    /// because every `error_code` can map to a [`RejectCode`].
    reject_code: RejectCode,
    reject_message: String,
    sync: bool,
}

impl CallRejected {
    /// Gets the [`RejectCode`].
    pub fn reject_code(&self) -> RejectCode {
        self.reject_code
    }

    /// Retrieves the reject message associated with the call.
    ///
    /// - For an asynchronous rejection (when the IC rejects the call after it was enqueued),
    ///   the message is obtained from [`api::msg_reject_msg`](`msg_reject_msg`).
    /// - For a synchronous rejection (when `ic0.call_perform` returns a non-zero code),
    ///   the message is set to a fixed string: `"call_perform failed"`.
    pub fn reject_message(&self) -> &str {
        &self.reject_message
    }

    /// Checks if the call was rejected synchronously or asynchronously.
    ///
    /// A synchronous rejection occurs when `ic0.call_perform` returns a non-zero code immediately.
    /// An asynchronous rejection happens when the call is enqueued but later rejected by the IC.
    ///
    /// # Returns
    /// - `true` if the call was rejected synchronously.
    /// - `false` if the call was rejected asynchronously.
    pub fn is_sync(&self) -> bool {
        self.sync
    }
}

/// Result of a inter-canister call.
pub type SystemResult<R> = Result<R, CallRejected>;

/// Result of a inter-canister call and decoding the response.
pub type CallResult<R> = Result<R, CallError>;

/// Inter-canister Call.
///
/// This type enables the configuration and execution of inter-canister calls using a builder pattern.
///
/// # Configuration
///
/// Before sending the call, users can configure following aspects of the call:
///
/// * Arguments:
///   * Single `CandidType` value: [`with_arg`][Self::with_arg].
///   * Tuple of multiple `CandidType` values: [`with_args`][Self::with_args].
///   * Raw bytes without Candid encoding: [`with_raw_args`][Self::with_raw_args].
///   * *Note*: If no methods in this category are invoked, the `Call` defaults to sending a **Candid empty tuple `()`**.
/// * Cycles:
///   * [`with_cycles`][Self::with_cycles].
/// * Response delivery:
///   * Guaranteed response: [`with_guaranteed_response`][Self::with_guaranteed_response].
///   * Best-effort response with a timeout: [`change_timeout`][Self::change_timeout].
///   * *Note*: If no methods in this category are invoked, the `Call` defaults to a **10-second timeout for Best-effort responses**.
///
/// Please note that all the configuration methods are chainable and can be called multiple times.
/// For each **aspect** of the call, the **last** configuration takes effect.
///
/// ## Example
///
/// ```rust, no_run
/// # use ic_cdk::call::Call;
/// # async fn bar() {
/// # let canister_id = ic_cdk::api::canister_self();
/// # let method = "foo";
/// let call = Call::new(canister_id, method)
///     .with_raw_args(&[1,0])
///     .with_guaranteed_response()
///     .with_cycles(1000)
///     .change_timeout(5)
///     .with_arg(42)
///     .with_cycles(2000);
/// # }
/// ```
///
/// The `call` above will have the following configuration in effect:
/// * Arguments: `42` encoded as Candid bytes.
/// * Cycles: 2000 cycles.
/// * Response delivery: best-effort response with a 5-second timeout.
///
/// # Execution
///
/// The `Call` can be executed using the following methods:
/// * [`call`][Self::call]: Decodes the response to a single `CandidType` value.
/// * [`call_tuple`][Self::call_tuple]: Decodes the response to a tuple of `CandidType` values.
/// * [`call_raw`][Self::call_raw]: Returns the raw bytes of the response.
/// * [`call_oneway`][Self::call_oneway]: Ignores the response.
///
/// ## Example
///
/// ```rust, no_run
/// # use ic_cdk::call::Call;
/// # async fn bar() {
/// # let canister_id = ic_cdk::api::canister_self();
/// # let method = "foo";
/// let call = Call::new(canister_id, method)
///     .change_timeout(5)
///     .with_arg(42)
///     .with_cycles(2000);
/// let result: u32 = call.call().await.unwrap();
/// let result_tuple: (u32,) = call.call_tuple().await.unwrap();
/// let result_bytes: Vec<u8> = call.call_raw().await.unwrap();
/// call.call_oneway().unwrap();
/// # }
/// ```
#[derive(Debug)]
pub struct Call<'m, 'a> {
    canister_id: Principal,
    method: &'m str,
    cycles: Option<u128>,
    timeout_seconds: Option<u32>,
    encoded_args: EncodedArgs<'a>,
}

/// Encoded arguments for the call.
#[derive(Debug)]
enum EncodedArgs<'a> {
    /// Owned bytes.
    ///
    /// For "no arg", [`Call::with_arg`] and [`Call::with_args`].
    Owned(Vec<u8>),
    /// Reference to raw bytes.
    ///
    /// For [`Call::with_raw_args`].
    Ref(&'a [u8]),
}

impl<'m, 'a> Call<'m, 'a> {
    /// Constructs a new best-effort responses [`Call`] with the Canister ID and method name.
    ///
    /// # Note
    ///
    /// The best-effort responses [`Call`] defaults to a 10-second timeout.
    /// To change the timeout, invoke the [`change_timeout`][Self::change_timeout] method.
    ///
    /// To get guaranteed responses, use the [`Call::guaranteed`] constructor instead.
    pub fn best_effort(canister_id: Principal, method: &'m str) -> Self {
        Self {
            canister_id,
            method,
            cycles: None,
            // Default to 10 seconds.
            timeout_seconds: Some(10),
            // Bytes for empty arguments.
            // `candid::Encode!(&()).unwrap()`
            encoded_args: EncodedArgs::Owned(vec![0x44, 0x49, 0x44, 0x4c, 0x00, 0x00]),
        }
    }

    /// Constructs a new guaranteed responses [`Call`] with the Canister ID and method name.
    ///
    /// To get best-effort responses, use the  [`Call::best_effort`] constructor instead.
    pub fn guaranteed(canister_id: Principal, method: &'m str) -> Self {
        Self {
            canister_id,
            method,
            cycles: None,
            timeout_seconds: None,
            // Bytes for empty arguments.
            // `candid::Encode!(&()).unwrap()`
            encoded_args: EncodedArgs::Owned(vec![0x44, 0x49, 0x44, 0x4c, 0x00, 0x00]),
        }
    }

    /// Sets the argument for the call.
    ///
    /// The argument must implement [`CandidType`].
    pub fn with_arg<A: CandidType>(self, arg: A) -> Self {
        Self {
            encoded_args: EncodedArgs::Owned(
                encode_one(&arg).unwrap_or_else(panic_when_encode_fails),
            ),
            ..self
        }
    }

    /// Sets the arguments for the call.
    ///
    /// The arguments are a tuple of types, each implementing [`CandidType`].
    pub fn with_args<A: ArgumentEncoder>(self, args: &A) -> Self {
        Self {
            encoded_args: EncodedArgs::Owned(
                encode_args_ref(args).unwrap_or_else(panic_when_encode_fails),
            ),
            ..self
        }
    }

    /// Sets the arguments for the call as raw bytes.
    pub fn with_raw_args(self, raw_args: &'a [u8]) -> Self {
        Self {
            encoded_args: EncodedArgs::Ref(raw_args),
            ..self
        }
    }

    /// Sets the cycles payment for the call.
    ///
    /// # Note
    ///
    /// The behavior of this method when invoked multiple times is as follows:
    /// - Overrides any previously set cycle value
    /// - Last invocation determines the final cycles amount
    /// - Does not accumulate cycles across multiple invocations
    pub fn with_cycles(mut self, cycles: u128) -> Self {
        self.cycles = Some(cycles);
        self
    }

    /// Sets the call to have a guaranteed response.
    ///
    /// If [`change_timeout`](Self::change_timeout) is invoked after this method,
    /// the call will instead be set with best-effort responses.
    pub fn with_guaranteed_response(mut self) -> Self {
        self.timeout_seconds = None;
        self
    }

    /// Sets the timeout for best-effort responses.
    ///
    /// If not set, the call defaults to a 10-second timeout.
    /// If invoked multiple times, the last value takes effect.
    /// If [`with_guaranteed_response`](Self::with_guaranteed_response) is invoked after this method,
    /// the timeout will be ignored.
    ///
    /// # Note
    ///
    /// A timeout of 0 second **DOES NOT** mean guranteed response.
    /// The call would most likely time out (result in a `SysUnknown` reject).
    /// Unless it's a call to the canister on the same subnet,
    /// and the execution manages to schedule both the request and the response in the same round.
    ///
    /// To make the call with a guaranteed response,
    /// use the [`with_guaranteed_response`](Self::with_guaranteed_response) method.
    pub fn change_timeout(mut self, timeout_seconds: u32) -> Self {
        self.timeout_seconds = Some(timeout_seconds);
        self
    }

    /// Sends the call and gets the reply as raw bytes.
    pub fn call_raw(&self) -> impl Future<Output = SystemResult<Vec<u8>>> + Send + Sync + '_ {
        let state = Arc::new(RwLock::new(CallFutureState {
            result: None,
            waker: None,
            call: self,
        }));
        CallFuture { state }
    }

    /// Sends the call and decodes the reply to a Candid type.
    pub fn call<R>(&self) -> impl Future<Output = CallResult<R>> + Send + Sync + '_
    where
        Self: Sized,
        R: CandidType + for<'d> Deserialize<'d>,
    {
        let fut = self.call_raw();
        async {
            let bytes = fut.await.map_err(CallError::CallRejected)?;
            decode_one(&bytes).map_err(decoder_error_to_call_error::<R>)
        }
    }

    /// Sends the call and decodes the reply to a Candid type.
    pub fn call_tuple<R>(&self) -> impl Future<Output = CallResult<R>> + Send + Sync + '_
    where
        Self: Sized,
        R: for<'d> ArgumentDecoder<'d>,
    {
        let fut = self.call_raw();
        async {
            let bytes = fut.await.map_err(CallError::CallRejected)?;
            decode_args(&bytes).map_err(decoder_error_to_call_error::<R>)
        }
    }

    /// Sends the call and ignores the reply.
    pub fn call_oneway(&self) -> SystemResult<()> {
        // The conversion fails only when the err_code is 0, which means the call was successfully enqueued.
        match self.perform(None) {
            0 => Ok(()),
            code => {
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

    /// Performs the call.
    ///
    /// This is an internal helper function only for [`Self::call_oneway`] and [`CallFuture::poll`].
    ///
    /// # Arguments
    ///
    /// * `state_ptr`: An optional pointer to the internal state of the [`CallFuture`].
    ///   * If `Some`, the call will be prepared for asynchronous execution:
    ///     * `ic0.call_new` will be invoked with [`callback`] and state pointer.
    ///     * `ic0.call_on_cleanup` will be invoked with [`cleanup`].
    ///   * If `None`, the call will be prepared for oneway execution:
    ///     * `ic0.call_new` will be invoked with invalid callback functions.
    ///     * `ic0.call_on_cleanup` won't be invoked.
    ///
    /// # Returns
    ///
    /// The return value of `ic0.call_perform`.
    fn perform(&self, state_ptr_opt: Option<*const RwLock<CallFutureState<'_, '_>>>) -> u32 {
        let callee = self.canister_id.as_slice();
        let method = self.method;
        let arg = match &self.encoded_args {
            EncodedArgs::Owned(vec) => vec,
            EncodedArgs::Ref(r) => *r,
        };

        let (reply_fun, reply_env, reject_fun, reject_env) = match state_ptr_opt {
            // asynchronous execution
            //
            // # SAFETY:
            // ic0.call_new
            // * `callback` is a function with signature `(env : usize) -> ()` and therefore can be called as both reply and reject fn for ic0.call_new.
            // * `state_ptr` is a pointer created via Weak::into_raw, and can therefore be passed as the userdata for `callback`.
            Some(state_ptr) => (
                callback as usize,
                state_ptr as usize,
                cleanup as usize,
                state_ptr as usize,
            ),
            // oneway execution
            //
            // # SAFETY:
            // ic0.call_new
            // * `reply_fun` and `reject_fun`: In "oneway" style call, we want these callback functions to not be called.
            //    So pass `usize::MAX` which is a function pointer the wasm module cannot possibly contain.
            // * `reply_env` and `reject_env`: Since the callback functions will never be called, any value can be passed as its context parameter.
            //
            // See https://www.joachim-breitner.de/blog/789-Zero-downtime_upgrades_of_Internet_Computer_canisters#one-way-calls for more context.
            None => (usize::MAX, usize::MAX, usize::MAX, usize::MAX),
        };
        // SAFETY:
        // `callee_src` and `callee_size`: `callee` being &[u8], is a readable sequence of bytes.
        // `name_src` and `name_size`: `method`, being &str, is a readable sequence of bytes.
        // reply and reject callbacks are discussed above.
        unsafe {
            ic0::call_new(
                callee.as_ptr() as usize,
                callee.len(),
                method.as_ptr() as usize,
                method.len(),
                reply_fun,
                reply_env,
                reject_fun,
                reject_env,
            );
        }
        if !arg.is_empty() {
            // SAFETY: `args`, being a &[u8], is a readable sequence of bytes.
            unsafe { ic0::call_data_append(arg.as_ptr() as usize, arg.len()) };
        }
        if let Some(cycles) = self.cycles {
            let high = (cycles >> 64) as u64;
            let low = (cycles & u64::MAX as u128) as u64;
            // SAFETY: ic0.call_cycles_add128 is always safe to call.
            unsafe { ic0::call_cycles_add128(high, low) };
        }
        if let Some(timeout_seconds) = self.timeout_seconds {
            // SAFETY: ic0.call_with_best_effort_response is always safe to call.
            unsafe { ic0::call_with_best_effort_response(timeout_seconds) };
        }
        if let Some(state_ptr) = state_ptr_opt {
            // Only invoke `ic0.call_on_cleanup` for asynchronous (non-oneway) calls.
            // For oneway calls, since the reply/reject callback always traps (the callbacks were set to `usize::MAX`),
            // the cleanup callback must not be set to ensure the "oneway" semantics.
            //
            // SAFETY:
            // `cleanup` is a function with signature `(env : usize) -> ()` and therefore can be passed as a cleanup callback ptr.
            // `state_ptr` is a pointer created via Weak::into_raw, and can therefore be passed as the userdata for `cleanup`.
            unsafe { ic0::call_on_cleanup(cleanup as usize, state_ptr as usize) };
        }
        // SAFETY: ic0.call_perform is always safe to call
        unsafe { ic0::call_perform() }
    }
}

// # Internal =================================================================

/// Internal state for the Future when sending a call.
struct CallFutureState<'m, 'a> {
    result: Option<SystemResult<Vec<u8>>>,
    waker: Option<Waker>,
    call: &'m Call<'m, 'a>,
}

struct CallFuture<'m, 'a> {
    state: Arc<RwLock<CallFutureState<'m, 'a>>>,
}

impl<'m, 'a> Future for CallFuture<'m, 'a> {
    type Output = SystemResult<Vec<u8>>;

    fn poll(self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Self::Output> {
        let self_ref = Pin::into_inner(self);
        let mut state = self_ref.state.write().unwrap();

        if let Some(result) = state.result.take() {
            Poll::Ready(result)
        } else {
            if state.waker.is_none() {
                let state_ptr = Weak::into_raw(Arc::downgrade(&self_ref.state));

                match state.call.perform(Some(state_ptr)) {
                    0 => {
                        // call_perform returns 0 means the call was successfully enqueued.
                    }
                    code => {
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

/// The reply/reject callback for `ic0.call_new`.
///
/// It dereferences the future from a raw pointer, assigns the result and calls the waker.
/// We cannot use a closure here because we pass raw pointers to the System and back.
///
/// # Safety
///
/// This function must only be passed to the IC with a pointer from `Weak::into_raw` as userdata.
unsafe extern "C" fn callback(state_ptr: *const RwLock<CallFutureState<'_, '_>>) {
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

/// The cleanup callback for `ic0.call_on_cleanup`.
///
/// This function is called when [`callback`] was just called with the same parameter, and trapped.
/// We can't guarantee internal consistency at this point, but we can at least e.g. drop mutex guards.
/// Waker is a very opaque API, so the best we can do is set a global flag and proceed normally.
///
/// # Safety
///
/// This function must only be passed to the IC with a pointer from Weak::into_raw as userdata.
unsafe extern "C" fn cleanup(state_ptr: *const RwLock<CallFutureState<'_, '_>>) {
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

// # Internal END =============================================================

/// Converts a decoder error to a [`CallError`].
fn decoder_error_to_call_error<T>(err: candid::error::Error) -> CallError {
    CallError::CandidDecodeFailed(format!("{}: {}", std::any::type_name::<T>(), err))
}

/// Panics with an informative message when argument encoding fails.
///
/// Currently, Candid encoding only fails when heap memory is exhausted,
/// in which case execution would trap before reaching the unwrap.
///
/// However, since future implementations might introduce other failure cases,
/// we provide an informative panic message for better debuggability.
fn panic_when_encode_fails(err: candid::error::Error) -> Vec<u8> {
    panic!("failed to encode args: {}", err)
}
