use candid::{CandidType, Nat, Principal};
use ic_cdk::api::call::{call, CallResult};
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
    pub method: HttpMethod,
    pub headers: Vec<HttpHeader>,
    pub body: Option<Vec<u8>>,
    // TODO: Here is a discrepancy between System API and the implementation.
    pub transform_method_name: Option<String>,
}

#[derive(CandidType, Clone, Debug, PartialEq, Eq, Hash, Deserialize)]
pub struct CanisterHttpResponse {
    pub status: Nat,
    pub headers: Vec<HttpHeader>,
    pub body: Vec<u8>,
}

pub async fn http_request(arg: CanisterHttpRequestArgument) -> CallResult<(CanisterHttpResponse,)> {
    call(Principal::management_canister(), "http_request", (arg,)).await
}
