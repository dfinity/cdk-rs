//! Inter-canister Call API
//!
//! This module provides the necessary APIs to make and manage inter-canister calls within a canister.
//! It offers a builder pattern to configure and execute calls, allowing for flexible and customizable interactions
//! between canisters.
//!
//! # Overview
//!
//! The primary type in this module is [`Call`], which represents an inter-canister call. For detailed usage and examples,
//! refer to the [`Call`] type documentation.
//!
//! ```rust, no_run
//! # use ic_cdk::call::Call;
//! # async fn bar() {
//! # let canister_id = ic_cdk::api::canister_self();
//! # let method = "foo";
//! let result: u32 = Call::bounded_wait(canister_id, method).await.unwrap().candid().unwrap();
//! # }
//! ```
//!
//! # Error Handling
//!
//! The module defines various error types to handle different failure scenarios during inter-canister calls:
//!
//! - [`enum@Error`]: The top-level error type encapsulating all possible errors.
//! - [`CallFailed`]: Errors related to the execution of the call itself.
//! - [`PreExecutionFailure`]: Errors during the pre-execution phase of an inter-canister call.
//! - [`InsufficientLiquidCycleBalance`]: Errors when the liquid cycle balance is insufficient to perform the call.
//! - [`CallPerformFailed`]: Errors when the `ic0.call_perform` operation fails.
//! - [`CallRejected`]: Errors when an inter-canister call is rejected.
//! - [`CandidDecodeFailed`]: Errors when the response cannot be decoded as Candid.
//!
//! ```text
//! Error
//! ├── CallFailed
//! │   ├── PreExecutionFailure
//! │   │   ├── InsufficientLiquidCycleBalance
//! │   │   └── CallPerformFailed
//! │   └── CallRejected
//! └── CandidDecodeFailed
//! ```
//!
//! # Internal Details
//!
//! The module also includes internal types and functions to manage the state and execution of inter-canister calls,
//! such as [`CallFuture`] and its associated state management.

use crate::api::{cost_call, msg_arg_data, msg_reject_code, msg_reject_msg};
use crate::{futures::is_recovering_from_trap, trap};
use candid::utils::{encode_args_ref, ArgumentDecoder, ArgumentEncoder};
use candid::{decode_args, decode_one, encode_one, CandidType, Deserialize, Principal};
use std::borrow::Cow;
use std::future::IntoFuture;
use std::mem;
use std::pin::Pin;
use std::sync::atomic::Ordering;
use std::sync::{Arc, RwLock};
use std::task::{Context, Poll, Waker};
use thiserror::Error;

pub use ic_error_types::RejectCode;

/// Inter-canister Call.
///
/// This type enables the configuration and execution of inter-canister calls using a builder pattern.
///
/// # Constructors
///
/// [`Call`] has two constructors that differentiate whether the call's response is waited for an unbounded amount of time or not.
/// - [`bounded_wait`][Self::bounded_wait]: wait boundedly (defaults with 300-second timeout).
/// - [`unbounded_wait`][Self::unbounded_wait]: wait unboundedly.
///
/// # Configuration
///
/// Before execution, a [`Call`] can be configured in following aspects:
///
/// - Arguments:
///   - [`with_arg`][Self::with_arg]: single `CandidType` value that will be encoded.
///   - [`with_args`][Self::with_args]: a tuple of multiple `CandidType` values that will be encoded.
///   - [`with_raw_args`][Self::with_raw_args]: raw bytes that won't be encoded.
///   - *Note*: If no methods in this category are invoked, the [`Call`] defaults to sending a **Candid empty tuple `()`**.
/// - Cycles:
///   - [`with_cycles`][Self::with_cycles]: set the cycles attached in this call.
/// - Response waiting timeout:
///   - [`change_timeout`][Self::change_timeout]: change the timeout for **bounded_wait** call.
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
/// let call = Call::bounded_wait(canister_id, method)
///     .with_raw_args(&[1,0])
///     .with_cycles(1000)
///     .change_timeout(5)
///     .with_arg(42)
///     .with_cycles(2000);
/// # }
/// ```
///
/// The `call` above will have the following configuration in effect:
/// - Arguments: `42` encoded as Candid bytes.
/// - Attach 2000 cycles.
/// - Boundedly waiting for response with a 5-second timeout.
///
/// # Execution
///
/// A [`Call`] can be executed in two ways:
/// - `.await`: convert into a future, execute asynchronously and wait for response.
/// - [`oneway`][Self::oneway]: send a oneway call and not wait for the response.
///
/// ## Example
///
/// ```rust, no_run
/// # use ic_cdk::call::Call;
/// # async fn bar() {
/// # let canister_id = ic_cdk::api::canister_self();
/// # let method = "foo";
/// let call = Call::bounded_wait(canister_id, method);
/// let response = call.clone().await.unwrap();
/// call.oneway().unwrap();
/// # }
/// ```
///
/// # Decoding the response
///
/// If an asynchronous [`Call`] succeeds, the response can be decoded in two ways:
/// - [`candid`][Response::candid]: decode the response as a single Candid type.
/// - [`candid_tuple`][Response::candid_tuple]: decode the response as a tuple of Candid types.
///
/// ## Example
///
/// ```rust, no_run
/// # use ic_cdk::call::{Call, Response};
/// # async fn bar() {
/// # let canister_id = ic_cdk::api::canister_self();
/// # let method = "foo";
/// let res: Response = Call::bounded_wait(canister_id, method).await.unwrap();
/// let result: u32 = res.candid().unwrap();
/// let result_tuple: (u32,) = res.candid_tuple().unwrap();
/// # }
/// ```
///
/// <div class="warning">
///
/// Using an inter-canister call creates the possibility that your async function will be canceled partway through.
/// Read the [`futures`](crate::futures) module docs for why and how this happens.
///
/// </div>
#[derive(Debug, Clone)]
pub struct Call<'m, 'a> {
    canister_id: Principal,
    method: &'m str,
    cycles: u128,
    timeout_seconds: Option<u32>,
    encoded_args: Cow<'a, [u8]>,
}

// Constructors
impl<'m> Call<'m, '_> {
    /// Constructs a [`Call`] which will **boundedly** wait for response.
    ///
    /// # Note
    ///
    /// The bounded waiting is set with a default 300-second timeout.
    /// It aligns with the `MAX_CALL_TIMEOUT` constant in the current IC implementation.
    /// The timeout can be changed using the [`change_timeout`][Self::change_timeout] method.
    ///
    /// To unboundedly wait for response, use the [`Call::unbounded_wait`] constructor instead.
    pub fn bounded_wait(canister_id: Principal, method: &'m str) -> Self {
        Self {
            canister_id,
            method,
            cycles: 0,
            // Default to 300-second timeout.
            timeout_seconds: Some(300),
            // Bytes for empty arguments.
            // `candid::Encode!(&()).unwrap()`
            encoded_args: Cow::Owned(vec![0x44, 0x49, 0x44, 0x4c, 0x00, 0x00]),
        }
    }

    /// Constructs a [`Call`] which will **unboundedly** wait for response.
    ///
    /// To boundedly wait for response, use the  [`Call::bounded_wait`] constructor instead.
    pub fn unbounded_wait(canister_id: Principal, method: &'m str) -> Self {
        Self {
            canister_id,
            method,
            cycles: 0,
            timeout_seconds: None,
            // Bytes for empty arguments.
            // `candid::Encode!(&()).unwrap()`
            encoded_args: Cow::Owned(vec![0x44, 0x49, 0x44, 0x4c, 0x00, 0x00]),
        }
    }
}

// Configuration
impl<'a> Call<'_, 'a> {
    /// Sets the argument for the call.
    ///
    /// The argument must implement [`CandidType`].
    pub fn with_arg<A: CandidType>(self, arg: A) -> Self {
        Self {
            encoded_args: Cow::Owned(encode_one(&arg).unwrap_or_else(panic_when_encode_fails)),
            ..self
        }
    }

    /// Sets the arguments for the call.
    ///
    /// The arguments are a tuple of types, each implementing [`CandidType`].
    pub fn with_args<A: ArgumentEncoder>(self, args: &A) -> Self {
        Self {
            encoded_args: Cow::Owned(encode_args_ref(args).unwrap_or_else(panic_when_encode_fails)),
            ..self
        }
    }

    /// Sets the arguments for the call as raw bytes.
    pub fn with_raw_args(self, raw_args: &'a [u8]) -> Self {
        Self {
            encoded_args: Cow::Borrowed(raw_args),
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
        self.cycles = cycles;
        self
    }

    /// Changes the timeout for bounded response waiting.
    ///
    /// If invoked multiple times, the last value takes effect.
    ///
    /// The timeout value is silently capped by the `MAX_CALL_TIMEOUT` constant which is currently set to 300 seconds.
    /// Therefore, setting a timeout greater than 300 seconds will actually result in a 300-second timeout.
    ///
    /// # Panics
    ///
    /// This method will panic if invoked on an unbounded response waiting call constructed by [`Call::unbounded_wait`] .
    ///
    /// # Note
    ///
    /// A timeout of 0 second **DOES NOT** mean unbounded response waiting.
    /// The call would most likely time out (result in a [`SysUnknown`](RejectCode::SysUnknown) reject).
    /// Unless it's a call to the canister on the same subnet,
    /// and the execution manages to schedule both the request and the response in the same round.
    ///
    /// To unboundedly wait for response, use the [`Call::unbounded_wait`] constructor instead.
    pub fn change_timeout(mut self, timeout_seconds: u32) -> Self {
        match self.timeout_seconds {
            Some(_) => self.timeout_seconds = Some(timeout_seconds),
            None => {
                panic!("Cannot set a timeout for an instance created with Call::unbounded_wait")
            }
        }
        self
    }

    /// Returns the amount of cycles a canister needs to be above the freezing threshold in order to
    /// successfully perform this call. Takes into account the attached cycles ([`with_cycles`](Self::with_cycles))
    /// as well as
    /// - the method name byte length
    /// - the payload length
    /// - the cost of transmitting the request
    /// - the cost for the reservation of response transmission (may be partially refunded)
    /// - the cost for the reservation of callback execution (may be partially refunded).
    pub fn get_cost(&self) -> u128 {
        self.cycles.saturating_add(cost_call(
            self.method.len() as u64,
            self.encoded_args.len() as u64,
        ))
    }
}

/// Response of a successful call.
#[derive(Debug)]
pub struct Response(Vec<u8>);

impl Response {
    /// Gets the raw bytes of the response.
    pub fn into_bytes(self) -> Vec<u8> {
        self.0
    }

    /// Decodes the response as a single Candid type.
    pub fn candid<R>(&self) -> Result<R, CandidDecodeFailed>
    where
        R: CandidType + for<'de> Deserialize<'de>,
    {
        decode_one(&self.0).map_err(|e| CandidDecodeFailed {
            type_name: std::any::type_name::<R>().to_string(),
            candid_error: e.to_string(),
        })
    }

    /// Decodes the response as a tuple of Candid types.
    pub fn candid_tuple<R>(&self) -> Result<R, CandidDecodeFailed>
    where
        R: for<'de> ArgumentDecoder<'de>,
    {
        decode_args(&self.0).map_err(|e| CandidDecodeFailed {
            type_name: std::any::type_name::<R>().to_string(),
            candid_error: e.to_string(),
        })
    }
}

impl PartialEq<&[u8]> for Response {
    fn eq(&self, other: &&[u8]) -> bool {
        self.0 == *other
    }
}

impl PartialEq<Vec<u8>> for Response {
    fn eq(&self, other: &Vec<u8>) -> bool {
        self.0 == *other
    }
}

impl PartialEq for Response {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl std::ops::Deref for Response {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AsRef<[u8]> for Response {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl std::borrow::Borrow<[u8]> for Response {
    fn borrow(&self) -> &[u8] {
        &self.0
    }
}

// Errors ---------------------------------------------------------------------

/// Represents errors that can occur during inter-canister calls.
///
/// This is the top-level error type for the inter-canister call API.
///
/// This encapsulates all possible errors that can arise, including:
/// - Call failures (e.g., `ic0.call_perform` failed or asynchronously rejected).
/// - Candid decoding failures (e.g.,  the response cannot be decoded).
#[derive(Error, Debug, Clone)]
pub enum Error {
    /// The inter-canister call failed.
    ///
    /// This variant wraps errors related to the execution of the call itself.
    #[error(transparent)]
    CallFailed(#[from] CallFailed),

    /// The response from the inter-canister call could not be decoded as Candid.
    ///
    /// This variant wraps errors that occur when attempting to decode the response
    /// into the expected Candid type.
    #[error(transparent)]
    CandidDecodeFailed(#[from] CandidDecodeFailed),
}

/// Represents errors that can occur during the execution of an inter-canister call.
///
/// This is the error type when awaiting a [`CallFuture`].
///
/// This is wrapped by the top-level [`Error::CallFailed`] variant.
#[derive(Error, Debug, Clone)]
pub enum CallFailed {
    /// The `ic0.call_perform` operation returned a non-zero code, indicating a failure.
    #[error(transparent)]
    PreExecutionFailure(#[from] PreExecutionFailure),

    /// The inter-canister call is rejected.
    #[error(transparent)]
    CallRejected(#[from] CallRejected),
}

/// Represents errors that can occur during the pre-execution phase (before or at `ic0.call_perform) of an inter-canister call.
///
/// This is the error type of [`Call::oneway`].
///
/// This is wrapped by the [`CallFailed::PreExecutionFailure`] variant.
#[derive(Error, Debug, Clone)]
pub enum PreExecutionFailure {
    /// The liquid cycle balance is insufficient to perform the call.
    #[error(transparent)]
    InsufficientLiquidCycleBalance(#[from] InsufficientLiquidCycleBalance),
    /// The `ic0.call_perform` operation failed.
    #[error(transparent)]
    CallPerformFailed(#[from] CallPerformFailed),
}

/// Represents an error that occurs when the liquid cycle balance is insufficient to perform the call.
///
/// The liquid cycle balance is determined by [`canister_liquid_cycle_balance`](crate::api::canister_liquid_cycle_balance).
/// The cost of the call is determined by [`Call::get_cost`].
///
/// The call won't be performed if the former is less than the latter.
#[derive(Error, Debug, Clone)]
#[error("Insufficient liquid cycles balance")]
pub struct InsufficientLiquidCycleBalance;

/// Represents an error that occurs when the `ic0.call_perform` operation fails.
///
/// This error type indicates that the underlying `ic0.call_perform` operation
/// returned a non-zero code, signaling a failure.
///
/// This is wrapped by the [`PreExecutionFailure::CallPerformFailed`] variant.
#[derive(Error, Debug, Clone)]
#[error("Call perform failed")]
pub struct CallPerformFailed;

/// Represents an error that occurs when an inter-canister call is rejected.
///
/// The [`reject_code`][`Self::reject_code`] and [`reject_message`][`Self::reject_message`]
/// are exposed to provide details of the rejection.
///
/// This is wrapped by the [`CallFailed::CallRejected`] variant.
#[derive(Error, Debug, Clone)]
#[error("Call rejected: {raw_reject_code} - {reject_message}")]
pub struct CallRejected {
    /// All fields are private so we will be able to change the implementation without breaking the API.
    /// Once we have `ic0.msg_error_code` system API, we will only store the `error_code` in this struct.
    /// It will still be possible to get the [`RejectCode`] using the public getter,
    /// because every `error_code` can map to a [`RejectCode`].
    raw_reject_code: u32,
    reject_message: String,
}

/// The error type for when an unrecognized reject code is encountered.
#[derive(Error, Debug, Clone, PartialEq, Eq)]
#[error("Unrecognized reject code: {0}")]
pub struct UnrecognizedRejectCode(u32);

impl CallRejected {
    /// Gets the [`RejectCode`].
    ///
    /// The value is converted from [`api::msg_reject_code`](`msg_reject_code`).
    ///
    /// # Errors
    ///
    /// If the raw reject code is not recognized, this method will return an [`UnrecognizedRejectCode`] error.
    /// This can happen if the IC produces a new reject code that hasn't been included in [`ic_error_types::RejectCode`].
    /// Please check if your `ic-error-types` dependency is up-to-date.
    /// If the latest version of `ic-error-types` doesn't include the new reject code, please report it to the `ic-cdk` maintainers.
    pub fn reject_code(&self) -> Result<RejectCode, UnrecognizedRejectCode> {
        RejectCode::try_from(self.raw_reject_code as u64)
            .map_err(|_| UnrecognizedRejectCode(self.raw_reject_code))
    }

    /// Gets the raw numeric [`RejectCode`] value.
    ///
    /// This is a "never-fail" version of [`reject_code`](Self::reject_code) that returns the raw numeric value.
    pub fn raw_reject_code(&self) -> u32 {
        self.raw_reject_code
    }

    /// Retrieves the reject message associated with the call.
    ///
    /// This message is obtained from [`api::msg_reject_msg`](`msg_reject_msg`).
    pub fn reject_message(&self) -> &str {
        &self.reject_message
    }
}

/// Represents an error that occurs when the response from an inter-canister call
/// cannot be decoded as Candid.
///
/// This error type provides details about the Candid decoding failure, including
/// the type that was being decoded and the specific Candid error that occurred.
///
/// This is the only possible error that can occur in [`Response::candid`] and [`Response::candid_tuple`].
///
/// It is wrapped by the top-level [`Error::CandidDecodeFailed`] variant.
#[derive(Error, Debug, Clone)]
#[error("Candid decode failed for type: {type_name}, candid error: {candid_error}")]
pub struct CandidDecodeFailed {
    type_name: String,
    candid_error: String,
}

/// Extension trait for error types to provide additional methods.
pub trait CallErrorExt {
    /// Checks if the error is a clean reject.
    /// A clean reject means that there must be no state changes on the callee side.
    fn is_clean_reject(&self) -> bool;
    /// Determines if the failed call can be retried immediately within the update method
    /// that's handling the error, as opposed to relying on a background timer or heartbeat.
    ///
    /// A return value of `true` indicates that an immediate retry *might* succeed, i.e., not result in another error.
    /// However, the caller is responsible for ensuring that retries are safe in their specific context.
    /// For idempotent endpoints, immediate retries are generally safe. For non-idempotent ones,
    /// checking [`is_clean_reject`](CallErrorExt::is_clean_reject) before retrying is recommended.
    fn is_immediately_retryable(&self) -> bool;
}

impl CallErrorExt for InsufficientLiquidCycleBalance {
    fn is_clean_reject(&self) -> bool {
        // The call was not performed.
        true
    }

    fn is_immediately_retryable(&self) -> bool {
        // Caller should top up cycles before retrying.
        false
    }
}

impl CallErrorExt for CallPerformFailed {
    fn is_clean_reject(&self) -> bool {
        true
    }

    fn is_immediately_retryable(&self) -> bool {
        false
    }
}

impl CallErrorExt for PreExecutionFailure {
    fn is_clean_reject(&self) -> bool {
        match self {
            PreExecutionFailure::InsufficientLiquidCycleBalance(e) => e.is_clean_reject(),
            PreExecutionFailure::CallPerformFailed(e) => e.is_clean_reject(),
        }
    }

    fn is_immediately_retryable(&self) -> bool {
        match self {
            PreExecutionFailure::InsufficientLiquidCycleBalance(e) => e.is_immediately_retryable(),
            PreExecutionFailure::CallPerformFailed(e) => e.is_immediately_retryable(),
        }
    }
}

impl CallErrorExt for CallRejected {
    fn is_clean_reject(&self) -> bool {
        // Here we apply a conservative whitelist of reject codes that are considered clean.
        // Once finer `error_code` is available, we can allow more cases to be clean.
        let clean_reject_codes: Vec<u32> = vec![
            RejectCode::SysFatal as u32,
            RejectCode::SysTransient as u32,
            RejectCode::DestinationInvalid as u32,
        ];
        clean_reject_codes.contains(&self.raw_reject_code)
    }

    fn is_immediately_retryable(&self) -> bool {
        // Here we apply a conservative whitelist of reject codes that are considered immediately retryable.
        // Once finer `error_code` is available, we can allow more cases to be immediately retryable.
        let immediately_retryable_codes: Vec<u32> = vec![
            RejectCode::SysTransient as u32,
            RejectCode::SysUnknown as u32,
        ];
        immediately_retryable_codes.contains(&self.raw_reject_code)
    }
}

impl CallErrorExt for CandidDecodeFailed {
    fn is_clean_reject(&self) -> bool {
        // Decoding failure suggests that the inter-canister call was successfully processed by the callee.
        // Therefore, the callee state is likely changed (unless the callee endpoint doesn't change its own state).
        false
    }

    fn is_immediately_retryable(&self) -> bool {
        // Decoding failure suggests a mismatch between the expected and actual response types.
        // Either the callee or the caller has a bug, and retrying the call immediately is unlikely to succeed.
        false
    }
}

impl CallErrorExt for Error {
    fn is_clean_reject(&self) -> bool {
        match self {
            Error::CallFailed(e) => e.is_clean_reject(),
            Error::CandidDecodeFailed(e) => e.is_clean_reject(),
        }
    }

    fn is_immediately_retryable(&self) -> bool {
        match self {
            Error::CallFailed(e) => e.is_immediately_retryable(),
            Error::CandidDecodeFailed(e) => e.is_immediately_retryable(),
        }
    }
}

impl CallErrorExt for CallFailed {
    fn is_clean_reject(&self) -> bool {
        match self {
            CallFailed::PreExecutionFailure(e) => e.is_clean_reject(),
            CallFailed::CallRejected(e) => e.is_clean_reject(),
        }
    }

    fn is_immediately_retryable(&self) -> bool {
        match self {
            CallFailed::PreExecutionFailure(e) => e.is_immediately_retryable(),
            CallFailed::CallRejected(e) => e.is_immediately_retryable(),
        }
    }
}

// Errors END -----------------------------------------------------------------

/// Result of a inter-canister call.
pub type CallResult<R> = Result<R, Error>;

impl<'m, 'a> IntoFuture for Call<'m, 'a> {
    type Output = Result<Response, CallFailed>;
    type IntoFuture = CallFuture<'m, 'a>;

    fn into_future(self) -> Self::IntoFuture {
        CallFuture {
            state: Arc::new(RwLock::new(CallFutureState::Prepared { call: self })),
        }
    }
}

// Execution
impl Call<'_, '_> {
    /// Sends the call and ignores the reply.
    pub fn oneway(&self) -> Result<(), PreExecutionFailure> {
        self.check_liquid_cycle_balance_sufficient()?;
        match self.perform(None) {
            0 => Ok(()),
            _ => Err(CallPerformFailed.into()),
        }
    }

    /// Checks if the liquid cycle balance is sufficient to perform the call.
    fn check_liquid_cycle_balance_sufficient(&self) -> Result<(), InsufficientLiquidCycleBalance> {
        let cost = self.get_cost();
        let liquid_balance = crate::api::canister_liquid_cycle_balance();
        if liquid_balance >= cost {
            Ok(())
        } else {
            Err(InsufficientLiquidCycleBalance)
        }
    }

    /// Performs the call.
    ///
    /// This is an internal helper function only for [`Self::call_oneway`] and [`CallFuture::poll`].
    ///
    /// # Arguments
    ///
    /// - `state_ptr`: An optional pointer to the internal state of the [`CallFuture`].
    ///   - If `Some`, the call will be prepared for asynchronous execution:
    ///     - `ic0.call_new` will be invoked with [`callback`] and state pointer.
    ///     - `ic0.call_on_cleanup` will be invoked with [`cleanup`].
    ///   - If `None`, the call will be prepared for oneway execution:
    ///     - `ic0.call_new` will be invoked with invalid callback functions.
    ///     - `ic0.call_on_cleanup` won't be invoked.
    ///
    /// # Returns
    ///
    /// The return value of `ic0.call_perform`.
    fn perform(&self, state_opt: Option<Arc<RwLock<CallFutureState<'_, '_>>>>) -> u32 {
        let callee = self.canister_id.as_slice();
        let method = self.method;
        let arg = match &self.encoded_args {
            Cow::Owned(vec) => vec,
            Cow::Borrowed(r) => *r,
        };
        let state_ptr_opt = state_opt.map(Arc::into_raw);
        match state_ptr_opt {
            Some(state_ptr) => {
                // asynchronous execution
                //
                // # SAFETY:
                // - `callee_src` and `callee_size`: `callee` being &[u8], is a readable sequence of bytes.
                // - `name_src` and `name_size`: `method`, being &str, is a readable sequence of bytes.
                // - `callback` is a function with signature `(env : usize) -> ()` and therefore can be called as
                //      both reply and reject fn for ic0.call_new.
                // - `cleanup` is a function with signature `(env : usize) -> ()` and therefore can be called as
                //      cleanup fn for ic0.call_on_cleanup.
                // - `state_ptr` is a pointer created via Arc::into_raw, and can therefore be passed as the userdata for
                //      `callback` and `cleanup`.
                // - if-and-only-if ic0.call_perform returns 0, exactly one of `callback` or `cleanup` will be called, exactly once,
                //      and therefore `state_ptr`'s ownership can be passed to both functions.
                // - both functions deallocate `state_ptr`, and this enclosing function deallocates `state_ptr` if ic0.call_perform
                //      returns 0, and therefore `state_ptr`'s ownership can be passed to FFI without leaking memory.
                unsafe {
                    ic0::call_new(
                        callee.as_ptr() as usize,
                        callee.len(),
                        method.as_ptr() as usize,
                        method.len(),
                        callback as usize,
                        state_ptr as usize,
                        callback as usize,
                        state_ptr as usize,
                    );
                    ic0::call_on_cleanup(cleanup as usize, state_ptr as usize);
                }
            }

            None => {
                // oneway execution
                //
                // # SAFETY:
                // - `callee_src` and `callee_size`: `callee` being &[u8], is a readable sequence of bytes.
                // - `name_src` and `name_size`: `method`, being &str, is a readable sequence of bytes.
                // - `reply_fun` and `reject_fun`: `usize::MAX` is a function pointer the wasm module cannot possibly contain.
                // - `reply_env` and `reject_env`: Since the callback functions will never be called, any value can be passed
                //      as their context parameters.
                //
                // See https://www.joachim-breitner.de/blog/789-Zero-downtime_upgrades_of_Internet_Computer_canisters#one-way-calls for more context.
                unsafe {
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
                }
            }
        };
        if !arg.is_empty() {
            // SAFETY: `args`, being a &[u8], is a readable sequence of bytes.
            unsafe { ic0::call_data_append(arg.as_ptr() as usize, arg.len()) };
        }
        if self.cycles > 0 {
            let high = (self.cycles >> 64) as u64;
            let low = (self.cycles & u64::MAX as u128) as u64;
            // SAFETY: ic0.call_cycles_add128 is always safe to call.
            unsafe { ic0::call_cycles_add128(high, low) };
        }
        if let Some(timeout_seconds) = self.timeout_seconds {
            // SAFETY: ic0.call_with_best_effort_response is always safe to call.
            unsafe { ic0::call_with_best_effort_response(timeout_seconds) };
        }
        // SAFETY: ic0.call_perform is always safe to call
        let res = unsafe { ic0::call_perform() };
        if res != 0 {
            if let Some(state_ptr) = state_ptr_opt {
                // SAFETY:
                // - `state_ptr_opt` is `Some` if-and-only-if ic0.call_new was called with ownership of `state`
                // - by returning !=0, ic0.call_new relinquishes ownership of `state_ptr`; it will never be passed
                //      to any functions
                // therefore, there is an outstanding handle to `state`, which it is safe to deallocate
                unsafe {
                    Arc::from_raw(state_ptr);
                }
            }
        }
        res
    }
}

// # Internal =================================================================

/// Internal state for the Future when sending a call.
#[derive(Debug, Default)]
enum CallFutureState<'m, 'a> {
    /// The future has been constructed, and the call has not yet been performed.
    /// Needed because futures are supposed to do nothing unless polled.
    /// Polling will attempt to fire off the request. Success returns `Pending` and transitions to `Executing`,
    /// failure returns `Ready` and transitions to `PostComplete.`
    Prepared { call: Call<'m, 'a> },
    /// The call has been performed and the message is in flight. Neither callback has been called. Polling will return `Pending`.
    /// This state will transition to `Trapped` if the future is canceled because of a trap in another future.
    Executing { waker: Waker },
    /// `callback` has been called, so the call has been completed. This completion state has not yet been read by the user.
    /// Polling will return `Ready` and transition to `PostComplete`.
    Complete {
        result: Result<Response, CallFailed>,
    },
    /// The completion state of `Complete` has been returned from `poll` as `Poll::Ready`. Polling again will trap.
    #[default]
    PostComplete,
    /// The future (*not* the state) was canceled because of a trap in another future during `Executing`. Polling will trap.
    Trapped,
}

/// Represents a future that resolves to the result of an inter-canister call.
///
/// This type is returned by [`IntoFuture::into_future`] when called on a [`Call`].
/// The [`Call`] type implements the [`IntoFuture`] trait, allowing it to be converted
/// into a [`CallFuture`]. The future can be awaited to retrieve the result of the call.
#[derive(Debug)]
pub struct CallFuture<'m, 'a> {
    state: Arc<RwLock<CallFutureState<'m, 'a>>>,
}

impl std::future::Future for CallFuture<'_, '_> {
    type Output = Result<Response, CallFailed>;

    fn poll(self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Self::Output> {
        let self_ref = Pin::into_inner(self);
        let mut state = self_ref.state.write().unwrap();
        match mem::take(&mut *state) {
            CallFutureState::Prepared { call } => {
                if let Err(e) = call.check_liquid_cycle_balance_sufficient() {
                    let pre_execution_failure: PreExecutionFailure = e.into();
                    let result = Err(pre_execution_failure.into());
                    *state = CallFutureState::PostComplete;
                    Poll::Ready(result)
                } else {
                    match call.perform(Some(self_ref.state.clone())) {
                        0 => {
                            // call_perform returns 0 means the call was successfully enqueued.
                            *state = CallFutureState::Executing {
                                waker: context.waker().clone(),
                            };
                            Poll::Pending
                        }
                        _ => {
                            let pre_execution_failure: PreExecutionFailure =
                                CallPerformFailed.into();
                            let result = Err(pre_execution_failure.into());
                            *state = CallFutureState::PostComplete;
                            Poll::Ready(result)
                        }
                    }
                }
            }
            CallFutureState::Executing { .. } => {
                *state = CallFutureState::Executing {
                    waker: context.waker().clone(),
                };
                Poll::Pending
            }
            CallFutureState::Complete { result } => {
                *state = CallFutureState::PostComplete;
                Poll::Ready(result)
            }
            CallFutureState::Trapped => trap("Call already trapped"),
            CallFutureState::PostComplete => trap("CallFuture polled after completing"),
        }
    }
}

impl Drop for CallFuture<'_, '_> {
    fn drop(&mut self) {
        // If this future is dropped while is_recovering_from_trap is true,
        // then it has been canceled due to a trap in another future.
        if is_recovering_from_trap() {
            *self.state.write().unwrap() = CallFutureState::Trapped;
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
/// This function must only be passed to the IC with a pointer from `Arc::into_raw` as userdata.
unsafe extern "C" fn callback(state_ptr: *const RwLock<CallFutureState<'_, '_>>) {
    crate::futures::in_callback_executor_context(|| {
        // SAFETY: This function is only ever called by the IC, and we only ever pass an Arc as userdata.
        let state = unsafe { Arc::from_raw(state_ptr) };
        let completed_state = CallFutureState::Complete {
            result: match msg_reject_code() {
                0 => Ok(Response(msg_arg_data())),
                code => {
                    // The conversion is safe because the code is not 0.
                    Err(CallFailed::CallRejected(CallRejected {
                        raw_reject_code: code,
                        reject_message: msg_reject_msg(),
                    }))
                }
            },
        };
        let waker = match mem::replace(&mut *state.write().unwrap(), completed_state) {
            CallFutureState::Executing { waker } => waker,
            // This future has already been cancelled and waking it will do nothing.
            // All that's left is to explicitly trap in case this is the last call being multiplexed,
            // to replace an automatic trap from not replying.
            CallFutureState::Trapped => trap("Call already trapped"),
            _ => unreachable!(
                "CallFutureState for in-flight calls should only be Executing or Trapped"
            ),
        };
        waker.wake();
    });
}

/// The cleanup callback for `ic0.call_on_cleanup`.
///
/// This function is called when [`callback`] was just called with the same parameter, and trapped.
/// We can't guarantee internal consistency at this point, but we can at least e.g. drop mutex guards.
/// Waker is a very opaque API, so the best we can do is set a global flag and proceed normally.
///
/// # Safety
///
/// This function must only be passed to the IC with a pointer from Arc::into_raw as userdata.
unsafe extern "C" fn cleanup(state_ptr: *const RwLock<CallFutureState<'_, '_>>) {
    // SAFETY: This function is only ever called by the IC, and we only ever pass a Arc as userdata.
    let state = unsafe { Arc::from_raw(state_ptr) };
    // We set the call result, even though it won't be read on the
    // default executor, because we can't guarantee it was called on
    // our executor. However, we are not allowed to inspect
    // reject_code() inside of a cleanup callback, so always set the
    // result to a reject.
    //
    // Borrowing does not trap - the rollback from the
    // previous trap ensures that the RwLock can be borrowed again.
    let err_state = CallFutureState::Complete {
        result: Err(CallFailed::CallRejected(CallRejected {
            raw_reject_code: RejectCode::CanisterReject as u32,
            reject_message: "cleanup".into(),
        })),
    };
    let waker = match mem::replace(&mut *state.write().unwrap(), err_state) {
        CallFutureState::Executing { waker } => waker,
        CallFutureState::Trapped => {
            // The future has already been canceled and dropped. There is nothing
            // more to clean up except for the CallFutureState.
            return;
        }
        _ => {
            unreachable!("CallFutureState for in-flight calls should only be Executing or Trapped")
        }
    };
    // Flag that we do not want to actually wake the task - we
    // want to drop it *without* executing it.
    crate::futures::CLEANUP.store(true, Ordering::Relaxed);
    waker.wake();
    crate::futures::CLEANUP.store(false, Ordering::Relaxed);
}

// # Internal END =============================================================

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
