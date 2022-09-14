//! Canister HTTP request.

use crate::api::call::{call_with_payment128, CallResult};
use candid::{
    parser::types::FuncMode,
    types::{Function, Serializer, Type},
    CandidType, Principal,
};
use core::hash::Hash;
use serde::{Deserialize, Serialize};

/// "transform" function of type:  `func (http_response) -> (http_response) query`
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct TransformFunc(pub candid::Func);

impl CandidType for TransformFunc {
    fn _ty() -> Type {
        Type::Func(Function {
            modes: vec![FuncMode::Query],
            args: vec![HttpResponse::ty()],
            rets: vec![HttpResponse::ty()],
        })
    }

    fn idl_serialize<S: Serializer>(&self, serializer: S) -> Result<(), S::Error> {
        serializer.serialize_function(self.0.principal.as_slice(), &self.0.method)
    }
}

/// "transform" reference function type:
/// `opt variant { function: func (http_response) -> (http_response) query }`
#[derive(Clone, Debug, CandidType, Deserialize, PartialEq)]
pub enum TransformType {
    /// reference function with signature: `func (http_response) -> (http_response) query`
    #[serde(rename = "function")]
    Function(TransformFunc),
}

/// HTTP header.
#[derive(
    CandidType, Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Default,
)]
pub struct HttpHeader {
    /// Name
    pub name: String,
    /// Value
    pub value: String,
}

/// HTTP method.
///
/// Currently support following methods.
#[derive(
    CandidType, Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy,
)]
pub enum HttpMethod {
    /// GET
    GET,
    /// POST
    POST,
    /// HEAD
    HEAD,
}

/// Argument type of [http_request].
#[derive(CandidType, Deserialize, Debug, PartialEq, Clone)]
pub struct CanisterHttpRequestArgument {
    /// The requested URL.
    pub url: String,
    /// The maximal size of the response in bytes. If None, 2MiB will be the limit.
    pub max_response_bytes: Option<u64>,
    // TODO: Different name in the Spec.
    /// The method of HTTP request.
    pub http_method: HttpMethod,
    /// List of HTTP request headers and their corresponding values.
    pub headers: Vec<HttpHeader>,
    /// Optionally provide request body.
    pub body: Option<Vec<u8>>,
    /// Name of the transform function which is `func (http_response) -> (http_response) query`.
    pub transform_method_name: Option<TransformType>,
}

/// The returned HTTP response.
#[derive(
    CandidType, Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Default,
)]
pub struct HttpResponse {
    /// The response status (e.g., 200, 404).
    pub status: u64,
    /// List of HTTP response headers and their corresponding values.
    pub headers: Vec<HttpHeader>,
    /// The response’s body.
    pub body: Vec<u8>,
}

/// Make an HTTP request to a given URL and return the HTTP response, possibly after a transformation.
///
/// This call requires cycles payment. The required cycles is a function of the request size and max_response_bytes.
/// See source code for the exact function.
///
/// See [IC method `http_request`](https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-http_request).
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

fn http_request_required_cycles(arg: &CanisterHttpRequestArgument) -> u128 {
    let max_response_bytes = match arg.max_response_bytes {
        Some(ref n) => *n as u128,
        None => 2 * 1024 * 1024u128, // default 2MiB
    };
    let arg_raw = candid::utils::encode_args((arg,)).expect("Failed to encode arguments.");
    // TODO: this formula should be documented somewhere
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
            http_method: HttpMethod::GET,
            headers: vec![],
            body: None,
            transform_method_name: None,
        };
        assert_eq!(http_request_required_cycles(&arg), 716500000u128);
    }

    #[test]
    fn required_cycles_none_max() {
        let url = "https://example.com".to_string();
        let arg = CanisterHttpRequestArgument {
            url,
            max_response_bytes: None,
            http_method: HttpMethod::GET,
            headers: vec![],
            body: None,
            transform_method_name: None,
        };
        assert_eq!(http_request_required_cycles(&arg), 210130900000u128);
    }
}
