//! APIs to make and manage calls in the canister.
use crate::api::{msg_arg_data, msg_reject_code, msg_reject_msg};
use candid::utils::{ArgumentDecoder, ArgumentEncoder};
use candid::{
    decode_args, decode_one, encode_args, encode_one, CandidType, Deserialize, Principal,
};
use std::error::Error;
use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::Ordering;
use std::sync::{Arc, RwLock, Weak};
use std::task::{Context, Poll, Waker};

/// Reject code explains why the inter-canister call is rejected.
///
/// See [Reject codes](https://internetcomputer.org/docs/current/references/ic-interface-spec/#reject-codes) for more details.
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
}

/// Error type for [`RejectCode`] conversion.
///
/// A reject code is invalid if it is not one of the known reject codes.
#[derive(Clone, Copy, Debug)]
pub struct InvalidRejectCode(pub u32);

impl std::fmt::Display for InvalidRejectCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "invalid reject code: {}", self.0)
    }
}

impl Error for InvalidRejectCode {}

impl TryFrom<u32> for RejectCode {
    type Error = InvalidRejectCode;

    fn try_from(code: u32) -> Result<Self, Self::Error> {
        match code {
            1 => Ok(RejectCode::SysFatal),
            2 => Ok(RejectCode::SysTransient),
            3 => Ok(RejectCode::DestinationInvalid),
            4 => Ok(RejectCode::CanisterReject),
            5 => Ok(RejectCode::CanisterError),
            6 => Ok(RejectCode::SysUnknown),
            n => Err(InvalidRejectCode(n)),
        }
    }
}

impl From<RejectCode> for u32 {
    fn from(code: RejectCode) -> u32 {
        match code {
            RejectCode::SysFatal => 1,
            RejectCode::SysTransient => 2,
            RejectCode::DestinationInvalid => 3,
            RejectCode::CanisterReject => 4,
            RejectCode::CanisterError => 5,
            RejectCode::SysUnknown => 6,
        }
    }
}

impl PartialEq<u32> for RejectCode {
    fn eq(&self, other: &u32) -> bool {
        let self_as_u32: u32 = (*self).into();
        self_as_u32 == *other
    }
}

/// The error codes provide additional details for rejected messages.
///
/// See [Error codes](https://internetcomputer.org/docs/current/references/ic-interface-spec/#error-codes) for more details.
///
/// # Note
///
/// As of the current version of the IC, the error codes are not available in the system API.
/// There is a plan to add them in the short term.
/// To avoid breaking changes at that time, the [`SystemError`] struct start to include the error code.
/// Please DO NOT rely on the error codes until they are officially supported.
//
// The variants and their codes below are from [pocket-ic](https://docs.rs/pocket-ic/latest/pocket_ic/enum.ErrorCode.html).
#[allow(missing_docs)]
#[derive(Clone, Copy, Hash, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ErrorCode {
    // 1xx -- `RejectCode::SysFatal`
    SubnetOversubscribed = 101,
    MaxNumberOfCanistersReached = 102,
    // 2xx -- `RejectCode::SysTransient`
    CanisterQueueFull = 201,
    IngressMessageTimeout = 202,
    CanisterQueueNotEmpty = 203,
    IngressHistoryFull = 204,
    CanisterIdAlreadyExists = 205,
    StopCanisterRequestTimeout = 206,
    CanisterOutOfCycles = 207,
    CertifiedStateUnavailable = 208,
    CanisterInstallCodeRateLimited = 209,
    CanisterHeapDeltaRateLimited = 210,
    // 3xx -- `RejectCode::DestinationInvalid`
    CanisterNotFound = 301,
    CanisterSnapshotNotFound = 305,
    // 4xx -- `RejectCode::CanisterReject`
    InsufficientMemoryAllocation = 402,
    InsufficientCyclesForCreateCanister = 403,
    SubnetNotFound = 404,
    CanisterNotHostedBySubnet = 405,
    CanisterRejectedMessage = 406,
    UnknownManagementMessage = 407,
    InvalidManagementPayload = 408,
    // 5xx -- `RejectCode::CanisterError`
    CanisterTrapped = 502,
    CanisterCalledTrap = 503,
    CanisterContractViolation = 504,
    CanisterInvalidWasm = 505,
    CanisterDidNotReply = 506,
    CanisterOutOfMemory = 507,
    CanisterStopped = 508,
    CanisterStopping = 509,
    CanisterNotStopped = 510,
    CanisterStoppingCancelled = 511,
    CanisterInvalidController = 512,
    CanisterFunctionNotFound = 513,
    CanisterNonEmpty = 514,
    QueryCallGraphLoopDetected = 517,
    InsufficientCyclesInCall = 520,
    CanisterWasmEngineError = 521,
    CanisterInstructionLimitExceeded = 522,
    CanisterMemoryAccessLimitExceeded = 524,
    QueryCallGraphTooDeep = 525,
    QueryCallGraphTotalInstructionLimitExceeded = 526,
    CompositeQueryCalledInReplicatedMode = 527,
    QueryTimeLimitExceeded = 528,
    QueryCallGraphInternal = 529,
    InsufficientCyclesInComputeAllocation = 530,
    InsufficientCyclesInMemoryAllocation = 531,
    InsufficientCyclesInMemoryGrow = 532,
    ReservedCyclesLimitExceededInMemoryAllocation = 533,
    ReservedCyclesLimitExceededInMemoryGrow = 534,
    InsufficientCyclesInMessageMemoryGrow = 535,
    CanisterMethodNotFound = 536,
    CanisterWasmModuleNotFound = 537,
    CanisterAlreadyInstalled = 538,
    CanisterWasmMemoryLimitExceeded = 539,
    ReservedCyclesLimitIsTooLow = 540,
    // 6xx -- `RejectCode::SysUnknown`
    DeadlineExpired = 601,
    ResponseDropped = 602,
}

/// Error type for [`ErrorCode`] conversion.
///
/// An error code is invalid if it is not one of the known error codes.
#[derive(Clone, Copy, Debug)]
pub struct InvalidErrorCode(pub u32);

impl std::fmt::Display for InvalidErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "invalid error code: {}", self.0)
    }
}

impl Error for InvalidErrorCode {}

impl TryFrom<u32> for ErrorCode {
    type Error = InvalidErrorCode;
    fn try_from(code: u32) -> Result<ErrorCode, Self::Error> {
        match code {
            // 1xx -- `RejectCode::SysFatal`
            101 => Ok(ErrorCode::SubnetOversubscribed),
            102 => Ok(ErrorCode::MaxNumberOfCanistersReached),
            // 2xx -- `RejectCode::SysTransient`
            201 => Ok(ErrorCode::CanisterQueueFull),
            202 => Ok(ErrorCode::IngressMessageTimeout),
            203 => Ok(ErrorCode::CanisterQueueNotEmpty),
            204 => Ok(ErrorCode::IngressHistoryFull),
            205 => Ok(ErrorCode::CanisterIdAlreadyExists),
            206 => Ok(ErrorCode::StopCanisterRequestTimeout),
            207 => Ok(ErrorCode::CanisterOutOfCycles),
            208 => Ok(ErrorCode::CertifiedStateUnavailable),
            209 => Ok(ErrorCode::CanisterInstallCodeRateLimited),
            210 => Ok(ErrorCode::CanisterHeapDeltaRateLimited),
            // 3xx -- `RejectCode::DestinationInvalid`
            301 => Ok(ErrorCode::CanisterNotFound),
            305 => Ok(ErrorCode::CanisterSnapshotNotFound),
            // 4xx -- `RejectCode::CanisterReject`
            402 => Ok(ErrorCode::InsufficientMemoryAllocation),
            403 => Ok(ErrorCode::InsufficientCyclesForCreateCanister),
            404 => Ok(ErrorCode::SubnetNotFound),
            405 => Ok(ErrorCode::CanisterNotHostedBySubnet),
            406 => Ok(ErrorCode::CanisterRejectedMessage),
            407 => Ok(ErrorCode::UnknownManagementMessage),
            408 => Ok(ErrorCode::InvalidManagementPayload),
            // 5xx -- `RejectCode::CanisterError`
            502 => Ok(ErrorCode::CanisterTrapped),
            503 => Ok(ErrorCode::CanisterCalledTrap),
            504 => Ok(ErrorCode::CanisterContractViolation),
            505 => Ok(ErrorCode::CanisterInvalidWasm),
            506 => Ok(ErrorCode::CanisterDidNotReply),
            507 => Ok(ErrorCode::CanisterOutOfMemory),
            508 => Ok(ErrorCode::CanisterStopped),
            509 => Ok(ErrorCode::CanisterStopping),
            510 => Ok(ErrorCode::CanisterNotStopped),
            511 => Ok(ErrorCode::CanisterStoppingCancelled),
            512 => Ok(ErrorCode::CanisterInvalidController),
            513 => Ok(ErrorCode::CanisterFunctionNotFound),
            514 => Ok(ErrorCode::CanisterNonEmpty),
            517 => Ok(ErrorCode::QueryCallGraphLoopDetected),
            520 => Ok(ErrorCode::InsufficientCyclesInCall),
            521 => Ok(ErrorCode::CanisterWasmEngineError),
            522 => Ok(ErrorCode::CanisterInstructionLimitExceeded),
            524 => Ok(ErrorCode::CanisterMemoryAccessLimitExceeded),
            525 => Ok(ErrorCode::QueryCallGraphTooDeep),
            526 => Ok(ErrorCode::QueryCallGraphTotalInstructionLimitExceeded),
            527 => Ok(ErrorCode::CompositeQueryCalledInReplicatedMode),
            528 => Ok(ErrorCode::QueryTimeLimitExceeded),
            529 => Ok(ErrorCode::QueryCallGraphInternal),
            530 => Ok(ErrorCode::InsufficientCyclesInComputeAllocation),
            531 => Ok(ErrorCode::InsufficientCyclesInMemoryAllocation),
            532 => Ok(ErrorCode::InsufficientCyclesInMemoryGrow),
            533 => Ok(ErrorCode::ReservedCyclesLimitExceededInMemoryAllocation),
            534 => Ok(ErrorCode::ReservedCyclesLimitExceededInMemoryGrow),
            535 => Ok(ErrorCode::InsufficientCyclesInMessageMemoryGrow),
            536 => Ok(ErrorCode::CanisterMethodNotFound),
            537 => Ok(ErrorCode::CanisterWasmModuleNotFound),
            538 => Ok(ErrorCode::CanisterAlreadyInstalled),
            539 => Ok(ErrorCode::CanisterWasmMemoryLimitExceeded),
            540 => Ok(ErrorCode::ReservedCyclesLimitIsTooLow),
            // 6xx -- `RejectCode::SysUnknown`
            601 => Ok(ErrorCode::DeadlineExpired),
            602 => Ok(ErrorCode::ResponseDropped),
            _ => Err(InvalidErrorCode(code)),
        }
    }
}

/// Get an [`ErrorCode`] from a [`RejectCode`].
///
/// Currently, there is no system API to get the error code.
/// This function is a temporary workaround.
/// We set the error code to the first code in the corresponding reject code group.
/// For example, the reject code `SysFatal` (1) is mapped to the error code `SubnetOversubscribed` (101).
fn reject_to_error(reject_code: RejectCode) -> ErrorCode {
    match reject_code {
        RejectCode::SysFatal => ErrorCode::SubnetOversubscribed,
        RejectCode::SysTransient => ErrorCode::CanisterQueueFull,
        RejectCode::DestinationInvalid => ErrorCode::CanisterNotFound,
        RejectCode::CanisterReject => ErrorCode::InsufficientMemoryAllocation,
        RejectCode::CanisterError => ErrorCode::CanisterTrapped,
        RejectCode::SysUnknown => ErrorCode::DeadlineExpired,
    }
}

/// The error type for inter-canister calls and decoding the response.
#[derive(thiserror::Error, Debug, Clone)]
pub enum CallError {
    /// The call was rejected.
    ///
    /// Please handle the error by matching on the rejection code.
    #[error("The call was rejected with code {0:?}")]
    CallRejected(SystemError),

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
pub struct SystemError {
    /// See [`RejectCode`].
    pub reject_code: RejectCode,
    /// The reject message.
    ///
    /// When the call was rejected asynchronously (IC rejects the call after it was enqueued),
    /// this message is set with [`msg_reject`](crate::api::msg_reject).
    ///
    /// When the call was rejected synchronously (`ic0.call_preform` returns non-zero code),
    /// this message is set to a fixed string ("failed to enqueue the call").
    pub reject_message: String,
    /// See [`ErrorCode`].
    ///
    /// # Note
    ///
    /// As of the current version of the IC, the error codes are not available in the system API.
    /// Please DO NOT rely on the error codes until they are officially supported.
    pub error_code: ErrorCode,
    /// Whether the call was rejected synchronously (`ic0.call_perform` returned non-zero code)
    /// or asynchronously (IC rejects the call after it was enqueued).
    pub sync: bool,
}

/// Result of a inter-canister call.
pub type SystemResult<R> = Result<R, SystemError>;

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
pub struct CallWithArg<'a, T> {
    call: Call<'a>,
    arg: T,
}

/// Inter-Canister Call with typed arguments.
///
/// The arguments are a tuple of types, each implementing [`CandidType`].
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
    pub fn with_arg<T>(self, arg: T) -> CallWithArg<'a, T> {
        CallWithArg { call: self, arg }
    }

    /// Sets the arguments for the call.
    ///
    /// The arguments are a tuple of types, each implementing [`CandidType`].
    pub fn with_args<T>(self, args: T) -> CallWithArgs<'a, T> {
        CallWithArgs { call: self, args }
    }

    /// Sets the arguments for the call as raw bytes.
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

impl<'a, T> ConfigurableCall for CallWithArg<'a, T> {
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
    fn call_raw(self) -> impl Future<Output = SystemResult<Vec<u8>>> + Send + Sync;

    /// Sends the call and decodes the reply to a Candid type.
    fn call<R>(self) -> impl Future<Output = CallResult<R>> + Send + Sync
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
    fn call_tuple<R>(self) -> impl Future<Output = CallResult<R>> + Send + Sync
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
    fn call_oneway(self) -> SystemResult<()>;
}

impl SendableCall for Call<'_> {
    fn call_raw(self) -> impl Future<Output = SystemResult<Vec<u8>>> + Send + Sync {
        let args_raw = vec![0x44, 0x49, 0x44, 0x4c, 0x00, 0x00];
        call_raw_internal::<Vec<u8>>(
            self.canister_id,
            self.method,
            args_raw,
            self.cycles,
            self.timeout_seconds,
        )
    }

    fn call_oneway(self) -> SystemResult<()> {
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

impl<'a, T: ArgumentEncoder + Send + Sync> SendableCall for CallWithArgs<'a, T> {
    async fn call_raw(self) -> SystemResult<Vec<u8>> {
        // Candid Encoding can only fail if heap memory is exhausted.
        // That is not a recoverable error, so we panic.
        let args_raw =
            encode_args(self.args).unwrap_or_else(|e| panic!("Failed to encode args: {}", e));
        call_raw_internal(
            self.call.canister_id,
            self.call.method,
            args_raw,
            self.call.cycles,
            self.call.timeout_seconds,
        )
        .await
    }

    fn call_oneway(self) -> SystemResult<()> {
        // Candid Encoding can only fail if heap memory is exhausted.
        // That is not a recoverable error, so we panic.
        let args_raw =
            encode_args(self.args).unwrap_or_else(|e| panic!("Failed to encode args: {}", e));
        call_oneway_internal(
            self.call.canister_id,
            self.call.method,
            args_raw,
            self.call.cycles,
            self.call.timeout_seconds,
        )
    }
}

impl<'a, T: CandidType + Send + Sync> SendableCall for CallWithArg<'a, T> {
    async fn call_raw(self) -> SystemResult<Vec<u8>> {
        // Candid Encoding can only fail if heap memory is exhausted.
        // That is not a recoverable error, so we panic.
        let args_raw =
            encode_one(self.arg).unwrap_or_else(|e| panic!("Failed to encode arg: {}", e));
        call_raw_internal(
            self.call.canister_id,
            self.call.method,
            args_raw,
            self.call.cycles,
            self.call.timeout_seconds,
        )
        .await
    }

    fn call_oneway(self) -> SystemResult<()> {
        // Candid Encoding can only fail if heap memory is exhausted.
        // That is not a recoverable error, so we panic.
        let args_raw =
            encode_one(self.arg).unwrap_or_else(|e| panic!("Failed to encode arg: {}", e));
        call_oneway_internal(
            self.call.canister_id,
            self.call.method,
            args_raw,
            self.call.cycles,
            self.call.timeout_seconds,
        )
    }
}

impl<'a, A: AsRef<[u8]> + Send + Sync + 'a> SendableCall for CallWithRawArgs<'a, A> {
    fn call_raw(self) -> impl Future<Output = SystemResult<Vec<u8>>> + Send + Sync {
        call_raw_internal(
            self.call.canister_id,
            self.call.method,
            self.raw_args,
            self.call.cycles,
            self.call.timeout_seconds,
        )
    }

    fn call_oneway(self) -> SystemResult<()> {
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
                        let reject_code = RejectCode::try_from(code).unwrap();
                        let result = Err(SystemError {
                            reject_code,
                            reject_message: "failed to enqueue the call".to_string(),
                            error_code: reject_to_error(reject_code),
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
                    let reject_code = RejectCode::try_from(code).unwrap();
                    Err(SystemError {
                        reject_code,
                        reject_message: msg_reject_msg(),
                        error_code: reject_to_error(reject_code),
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
            let reject_code = RejectCode::try_from(code).unwrap();
            Err(SystemError {
                reject_code,
                reject_message: "failed to enqueue the call".to_string(),
                error_code: reject_to_error(reject_code),
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
