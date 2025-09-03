use std::{
    borrow::Cow,
    marker::PhantomData,
    pin::Pin,
    task::{Context, Poll},
};

use candid::{
    CandidType, Principal,
    utils::{ArgumentDecoder, ArgumentEncoder, encode_args_ref},
};
use ic_error_types::RejectCode;
use pin_project_lite::pin_project;
use serde::de::DeserializeOwned;

use crate::call::{Call, CallFailed, CallFuture, Response, panic_when_encode_fails};

#[derive(Debug)]
pub struct CandidCall<'m, 'a, TParam, TReturn, TMode> {
    inner: Call<'m, 'a>,
    #[allow(clippy::type_complexity)]
    _marker: PhantomData<fn(&TParam) -> fn(TMode) -> TReturn>,
}

// If you add anything to this block, be sure to mirror it in Call
impl<'m, 'a, TParam: ArgumentEncoder, TReturn: for<'d> ArgumentDecoder<'d>, TMode>
    CandidCall<'m, 'a, TParam, TReturn, TMode>
{
    pub fn bounded_wait(canister_id: Principal, method: &'m str) -> Self {
        Self {
            inner: Call::bounded_wait(canister_id, method),
            _marker: PhantomData,
        }
    }
    pub fn unbounded_wait(canister_id: Principal, method: &'m str) -> Self {
        Self {
            inner: Call::unbounded_wait(canister_id, method),
            _marker: PhantomData,
        }
    }
    /// Sets the arguments for the call.
    #[must_use]
    pub fn with_args(mut self, arg: &TParam) -> Self {
        self.inner.encoded_args =
            Cow::Owned(encode_args_ref(arg).unwrap_or_else(panic_when_encode_fails));
        self
    }

    /// Sets the cycles payment for the call.
    ///
    /// # Note
    ///
    /// The behavior of this method when invoked multiple times is as follows:
    /// - Overrides any previously set cycle value
    /// - Last invocation determines the final cycles amount
    /// - Does not accumulate cycles across multiple invocations
    #[must_use]
    pub fn with_cycles(mut self, cycles: u128) -> Self {
        self.inner.cycles = cycles;
        self
    }

    /// Changes the timeout for bounded response waiting.
    ///
    /// If invoked multiple times, the last value takes effect.
    ///
    /// The timeout value is silently capped by the `MAX_CALL_TIMEOUT` constant which is currently set to 300 seconds.
    /// Therefore, setting a timeout greater than 300 seconds will actually result in a 300-second timeout.
    ///
    /// # Note
    ///
    /// A timeout of 0 second **DOES NOT** mean unbounded response waiting.
    /// The call would most likely time out (result in a [`SysUnknown`](RejectCode::SysUnknown) reject).
    /// Unless it's a call to the canister on the same subnet,
    /// and the execution manages to schedule both the request and the response in the same round.
    ///
    /// To unboundedly wait for response, use the [`CandidCall::unbounded_wait`] constructor instead.
    #[must_use]
    pub fn change_timeout(mut self, timeout_seconds: u32) -> Self {
        match self.inner.timeout_seconds {
            Some(_) => self.inner.timeout_seconds = Some(timeout_seconds),
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
    #[must_use]
    pub fn get_cost(&self) -> u128 {
        self.inner.get_cost()
    }
}

impl<'m, 'a, TElem: CandidType, TReturn: for<'d> ArgumentDecoder<'d>, TMode>
    CandidCall<'m, 'a, (TElem,), TReturn, TMode>
{
    /// Sets the argument for the call, if there is only one argument.
    #[must_use]
    pub fn with_arg(mut self, arg: &TElem) -> Self {
        self.inner.encoded_args =
            Cow::Owned(encode_args_ref(&(&arg,)).unwrap_or_else(panic_when_encode_fails));
        self
    }
}

impl<'m, 'a, TParam: ArgumentEncoder, TReturn: for<'d> ArgumentDecoder<'d>> IntoFuture
    for CandidCall<'m, 'a, TParam, TReturn, ModeMulti>
{
    type Output = Result<TReturn, CandidCallFailed>;
    type IntoFuture = CandidCallFuture<'m, 'a, TReturn, ModeMulti>;
    fn into_future(self) -> Self::IntoFuture {
        CandidCallFuture {
            inner: self.inner.into_future(),
            _marker: PhantomData,
        }
    }
}

impl<'m, 'a, TParam: ArgumentEncoder, TElem: CandidType + DeserializeOwned> IntoFuture
    for CandidCall<'m, 'a, TParam, (TElem,), ModeSingle>
{
    type Output = Result<TElem, CandidCallFailed>;
    type IntoFuture = CandidCallFuture<'m, 'a, (TElem,), ModeSingle>;
    fn into_future(self) -> Self::IntoFuture {
        CandidCallFuture {
            inner: self.inner.into_future(),
            _marker: PhantomData,
        }
    }
}

pub struct CandidCallFuture<'m, 'a, TReturn, TMode> {
    inner: CallFuture<'m, 'a>,
    _marker: PhantomData<fn(TMode) -> TReturn>,
}

impl<'m, 'a, TReturn: for<'d> ArgumentDecoder<'d>> Future
    for CandidCallFuture<'m, 'a, TReturn, ModeMulti>
{
    type Output = Result<TReturn, CandidCallFailed>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let inner = Pin::new(&mut self.get_mut().inner);
        inner.poll(cx).map(|res| match res {
            Err(failed) => Err(CandidCallFailed::CallFailed(failed)),
            Ok(resp) => match candid::decode_args(&resp) {
                Ok(decoded) => Ok(decoded),
                Err(err) => Err(CandidCallFailed::DecodeFailed {
                    err,
                    original: resp,
                }),
            },
        })
    }
}

impl<'m, 'a, TElem: CandidType + DeserializeOwned> Future
    for CandidCallFuture<'m, 'a, (TElem,), ModeSingle>
{
    type Output = Result<TElem, CandidCallFailed>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let inner = Pin::new(&mut self.get_mut().inner);
        inner.poll(cx).map(|res| match res {
            Err(failed) => Err(CandidCallFailed::CallFailed(failed)),
            Ok(resp) => match candid::decode_args::<(TElem,)>(&resp) {
                Ok(decoded) => Ok(decoded.0),
                Err(err) => Err(CandidCallFailed::DecodeFailed {
                    err,
                    original: resp,
                }),
            },
        })
    }
}

pub enum CandidCallFailed {
    CallFailed(CallFailed),
    DecodeFailed {
        err: candid::Error,
        original: Response,
    },
}

pub struct ModeSingle;
pub struct ModeMulti;
