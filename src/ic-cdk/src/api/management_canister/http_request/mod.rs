//! Canister HTTP request.

use crate::api::call::{call_with_payment128, CallResult};
use candid::Principal;
#[cfg(feature = "transform-closure")]
use slotmap::{DefaultKey, Key, SlotMap};
#[cfg(feature = "transform-closure")]
use std::cell::RefCell;

mod types;
pub use types::*;

#[cfg(feature = "transform-closure")]
thread_local! {
    #[allow(clippy::type_complexity)]
    static TRANSFORMS: RefCell<SlotMap<DefaultKey, Box<dyn FnOnce(HttpResponse) -> HttpResponse>>> = RefCell::default();
}

#[cfg(feature = "transform-closure")]
#[export_name = "canister_query <ic-cdk internal> http_transform"]
extern "C" fn http_transform() {
    use crate::api::{
        call::{arg_data, reply},
        caller,
    };
    use slotmap::KeyData;
    if caller() != Principal::management_canister() {
        crate::trap("This function is internal to ic-cdk and should not be called externally.");
    }
    crate::setup();
    let (args,): (TransformArgs,) = arg_data();
    let int = u64::from_be_bytes(args.context[..].try_into().unwrap());
    let key = DefaultKey::from(KeyData::from_ffi(int));
    let func = TRANSFORMS.with(|transforms| transforms.borrow_mut().remove(key));
    let Some(func) = func else {
        crate::trap(&format!("Missing transform function for request {int}"));
    };
    let transformed = func(args.response);
    reply((transformed,))
}

/// Make an HTTP request to a given URL and return the HTTP response, possibly after a transformation.
///
/// See [IC method `http_request`](https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-http_request).
///
/// This call requires cycles payment. The required cycles is a function of the request size and max_response_bytes.
/// This method handles the cycles cost calculation under the hood which assuming the canister is on a 13-node Application Subnet.
/// If the canister is on a 34-node Application Subnets, you may have to compute the cost by yourself and call [`http_request_with_cycles`] instead.
///
/// Check [this page](https://internetcomputer.org/docs/current/developer-docs/production/computation-and-storage-costs) for more details.
pub async fn http_request(arg: CanisterHttpRequestArgument) -> CallResult<(HttpResponse,)> {
    let cycles = http_request_required_cycles(&arg);
    call_with_payment128(
        Principal::management_canister(),
        "http_request",
        (arg,),
        cycles,
    )
    .await
}

/// Make an HTTP request to a given URL and return the HTTP response, after a transformation.
///
/// Do not set the `transform` field of `arg`. To use a Candid function, call [`http_request`] instead.
///
/// See [IC method `http_request`](https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-http_request).
///
/// This call requires cycles payment. The required cycles is a function of the request size and max_response_bytes.
/// This method handles the cycles cost calculation under the hood which assuming the canister is on a 13-node Application Subnet.
/// If the canister is on a 34-node Application Subnets, you may have to compute the cost by yourself and call [`http_request_with_cycles_with`] instead.
///
/// Check [this page](https://internetcomputer.org/docs/current/developer-docs/production/computation-and-storage-costs) for more details.
#[cfg(feature = "transform-closure")]
#[cfg_attr(docsrs, doc(cfg(feature = "transform-closure")))]
pub async fn http_request_with(
    arg: CanisterHttpRequestArgument,
    transform_func: impl FnOnce(HttpResponse) -> HttpResponse + 'static,
) -> CallResult<(HttpResponse,)> {
    let cycles = http_request_required_cycles(&arg);
    http_request_with_cycles_with(arg, cycles, transform_func).await
}

/// Make an HTTP request to a given URL and return the HTTP response, possibly after a transformation.
///
/// See [IC method `http_request`](https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-http_request).
///
/// This call requires cycles payment. The required cycles is a function of the request size and max_response_bytes.
/// Check [this page](https://internetcomputer.org/docs/current/developer-docs/production/computation-and-storage-costs) for more details.
///
/// If the canister is on a 13-node Application Subnet, you can call [`http_request`] instead which handles cycles cost calculation under the hood.
pub async fn http_request_with_cycles(
    arg: CanisterHttpRequestArgument,
    cycles: u128,
) -> CallResult<(HttpResponse,)> {
    call_with_payment128(
        Principal::management_canister(),
        "http_request",
        (arg,),
        cycles,
    )
    .await
}

/// Make an HTTP request to a given URL and return the HTTP response, after a transformation.
///
/// Do not set the `transform` field of `arg`. To use a Candid function, call [`http_request_with_cycles`] instead.
///
/// See [IC method `http_request`](https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-http_request).
///
/// This call requires cycles payment. The required cycles is a function of the request size and max_response_bytes.
/// Check [this page](https://internetcomputer.org/docs/current/developer-docs/production/computation-and-storage-costs) for more details.
///
/// If the canister is on a 13-node Application Subnet, you can call [`http_request_with`] instead which handles cycles cost calculation under the hood.
#[cfg(feature = "transform-closure")]
#[cfg_attr(docsrs, doc(cfg(feature = "transform-closure")))]
pub async fn http_request_with_cycles_with(
    arg: CanisterHttpRequestArgument,
    cycles: u128,
    transform_func: impl FnOnce(HttpResponse) -> HttpResponse + 'static,
) -> CallResult<(HttpResponse,)> {
    assert!(
        arg.transform.is_none(),
        "`CanisterHttpRequestArgument`'s `transform` field must be `None` when using a closure"
    );
    let transform_func = Box::new(transform_func) as _;
    let key = TRANSFORMS.with(|transforms| transforms.borrow_mut().insert(transform_func));
    struct DropGuard(DefaultKey);
    impl Drop for DropGuard {
        fn drop(&mut self) {
            TRANSFORMS.with(|transforms| transforms.borrow_mut().remove(self.0));
        }
    }
    let key = DropGuard(key);
    let context = key.0.data().as_ffi().to_be_bytes().to_vec();
    let arg = CanisterHttpRequestArgument {
        transform: Some(TransformContext {
            function: TransformFunc(candid::Func {
                method: "<ic-cdk internal> http_transform".into(),
                principal: crate::id(),
            }),
            context,
        }),
        ..arg
    };
    http_request_with_cycles(arg, cycles).await
}

fn http_request_required_cycles(arg: &CanisterHttpRequestArgument) -> u128 {
    let max_response_bytes = match arg.max_response_bytes {
        Some(ref n) => *n as u128,
        None => 2 * 1024 * 1024u128, // default 2MiB
    };
    let arg_raw = candid::utils::encode_args((arg,)).expect("Failed to encode arguments.");
    // The coefficients can be found in [this page](https://internetcomputer.org/docs/current/developer-docs/production/computation-and-storage-costs).
    // 12 is "http_request".len().
    400_000_000u128 + 100_000u128 * (arg_raw.len() as u128 + 12 + max_response_bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn required_cycles_some_max() {
        let url = "https://example.com".to_string();
        let arg = CanisterHttpRequestArgument {
            url,
            max_response_bytes: Some(3000),
            method: HttpMethod::GET,
            headers: vec![],
            body: None,
            transform: None,
        };
        assert_eq!(http_request_required_cycles(&arg), 718500000u128);
    }

    #[test]
    fn required_cycles_none_max() {
        let url = "https://example.com".to_string();
        let arg = CanisterHttpRequestArgument {
            url,
            max_response_bytes: None,
            method: HttpMethod::GET,
            headers: vec![],
            body: None,
            transform: None,
        };
        assert_eq!(http_request_required_cycles(&arg), 210132900000u128);
    }
}
