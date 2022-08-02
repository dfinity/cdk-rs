use crate::api::call::{call_with_payment128, CallResult};
use candid::{CandidType, Principal};
use serde::{Deserialize, Serialize};

#[derive(CandidType, Clone, Deserialize, Debug, Eq, Hash, PartialEq, Serialize)]
pub struct HttpHeader {
    pub name: String,
    pub value: String,
}

#[derive(Clone, Debug, PartialEq, CandidType, Eq, Hash, Serialize, Deserialize)]
pub enum HttpMethod {
    GET,
    POST,
    HEAD,
}

/*
record {
  url : text;
  max_response_bytes: opt nat64;
  method : variant { get; head; post };
  headers: vec http_header;
  body : opt blob;
  transform : opt variant {
    function: func (http_response) -> (http_response) query
  };
}
*/
#[derive(CandidType, Deserialize, Debug)]
pub struct CanisterHttpRequestArgument {
    pub url: String,
    pub max_response_bytes: Option<u64>,
    // TODO: Different name in the Spec.
    pub http_method: HttpMethod,
    pub headers: Vec<HttpHeader>,
    pub body: Option<Vec<u8>>,
    // TODO: Here is a discrepancy between System API and the implementation.
    pub transform_method_name: Option<String>,
}

#[derive(CandidType, Clone, Debug, PartialEq, Eq, Hash, Deserialize)]
pub struct CanisterHttpResponse {
    // TODO: Different type in the Spec.
    pub status: u64,
    pub headers: Vec<HttpHeader>,
    pub body: Vec<u8>,
}

pub async fn http_request(arg: CanisterHttpRequestArgument) -> CallResult<(CanisterHttpResponse,)> {
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
    400_000_000u128 + 100_000u128 * (arg_raw.len() as u128 + 12u128 + max_response_bytes)
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
        assert_eq!(http_request_required_cycles(&arg), 713100000u128);
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
        assert_eq!(http_request_required_cycles(&arg), 210127500000u128);
    }
}
