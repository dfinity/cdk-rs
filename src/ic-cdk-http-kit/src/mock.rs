//! Mocks HTTP requests.

use crate::storage;
use ic_cdk::api::call::{CallResult, RejectionCode};
use ic_cdk::api::management_canister::http_request::{
    CanisterHttpRequestArgument, HttpResponse, TransformArgs,
};
use std::time::Duration;

type MockError = (RejectionCode, String);

#[derive(Clone)]
pub(crate) struct Mock {
    pub(crate) request: CanisterHttpRequestArgument,
    result: Option<Result<HttpResponse, MockError>>,
    delay: Duration,
    times_called: u64,
}

impl Mock {
    /// Creates a new mock.
    pub fn new(
        request: CanisterHttpRequestArgument,
        result: Result<HttpResponse, MockError>,
        delay: Duration,
    ) -> Self {
        Self {
            request,
            result: Some(result),
            delay,
            times_called: 0,
        }
    }
}

/// Mocks a HTTP request.
pub fn mock(request: CanisterHttpRequestArgument, result: Result<HttpResponse, MockError>) {
    mock_with_delay(request, result, Duration::from_secs(0));
}

/// Mocks a HTTP request with a delay.
pub fn mock_with_delay(
    request: CanisterHttpRequestArgument,
    result: Result<HttpResponse, MockError>,
    delay: Duration,
) {
    storage::mock_insert(Mock::new(request, result, delay));
}

/// Returns the number of times a HTTP request was called.
/// Returns 0 if no mock has been found for the request.
pub fn times_called(request: CanisterHttpRequestArgument) -> u64 {
    storage::mock_get(&request)
        .map(|mock| mock.times_called)
        .unwrap_or(0)
}

/// Returns a sorted list of registered transform function names.
pub fn registered_transform_function_names() -> Vec<String> {
    storage::transform_function_names()
}

/// Make an HTTP request to a given URL and return the HTTP response, possibly after a transformation.
///
/// This is a helper function that compiles differently depending on the target architecture.
/// For wasm32 (assuming a canister in prod), it calls the IC method `http_request`.
/// For other architectures, it calls a mock function.
pub async fn http_request(arg: CanisterHttpRequestArgument) -> CallResult<(HttpResponse,)> {
    #[cfg(target_arch = "wasm32")]
    {
        ic_cdk::api::management_canister::http_request::http_request(arg).await
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        mock_http_request(arg).await
    }
}

/// Make an HTTP request to a given URL and return the HTTP response, possibly after a transformation.
///
/// This is a helper function that compiles differently depending on the target architecture.
/// For wasm32 (assuming a canister in prod), it calls the IC method `http_request_with_cycles`.
/// For other architectures, it calls a mock function.
pub async fn http_request_with_cycles(
    arg: CanisterHttpRequestArgument,
    cycles: u128,
) -> CallResult<(HttpResponse,)> {
    #[cfg(target_arch = "wasm32")]
    {
        ic_cdk::api::management_canister::http_request::http_request_with_cycles(arg, cycles).await
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        // Mocking cycles is not implemented at the moment.
        let _unused = cycles;
        mock_http_request(arg).await
    }
}

/// Handles incoming HTTP requests by retrieving a mock response based
/// on the request, possibly delaying the response, transforming the response if necessary,
/// and returning it. If there is no mock found, it returns an error.
async fn mock_http_request(
    request: CanisterHttpRequestArgument,
) -> Result<(HttpResponse,), (RejectionCode, String)> {
    let mut mock = storage::mock_get(&request)
        .ok_or((RejectionCode::CanisterReject, "No mock found".to_string()))?;
    mock.times_called += 1;
    storage::mock_insert(mock.clone());

    // Delay the response if necessary.
    if mock.delay > Duration::from_secs(0) {
        // Use a non-blocking sleep for tests, while wasm32 does not support tokio.
        #[cfg(not(target_arch = "wasm32"))]
        tokio::time::sleep(mock.delay).await;
    }

    let mock_response = match mock.result {
        None => panic!("Mock response is missing"),
        // Return the error if one is specified.
        Some(Err(error)) => return Err(error),
        Some(Ok(response)) => response,
    };

    // Check if the response body exceeds the maximum allowed size.
    if let Some(max_response_bytes) = mock.request.max_response_bytes {
        if mock_response.body.len() as u64 > max_response_bytes {
            return Err((
                RejectionCode::SysFatal,
                format!(
                    "Value of 'Content-length' header exceeds http body size limit, {} > {}.",
                    mock_response.body.len(),
                    max_response_bytes
                ),
            ));
        }
    }

    // Apply the transform function if one is specified.
    let context = mock.request.clone().transform.map_or(vec![], |f| f.context);
    let transformed_response = call_transform_function(
        mock.request,
        TransformArgs {
            response: mock_response.clone(),
            context,
        },
    )
    .unwrap_or(mock_response);

    Ok((transformed_response,))
}

/// Calls the transform function if one is specified in the request.
fn call_transform_function(
    request: CanisterHttpRequestArgument,
    arg: TransformArgs,
) -> Option<HttpResponse> {
    request
        .transform
        .and_then(|t| storage::transform_function_call(t.function.0.method, arg))
}

/// Create a hash from a `CanisterHttpRequestArgument`, which includes its URL,
/// method, headers, body, and optionally, its transform function name.
/// This is because `CanisterHttpRequestArgument` does not have `Hash` implemented.
pub(crate) fn hash(request: &CanisterHttpRequestArgument) -> String {
    let mut hash = String::new();

    hash.push_str(&request.url);
    hash.push_str(&format!("{:?}", request.max_response_bytes));
    hash.push_str(&format!("{:?}", request.method));
    for header in request.headers.iter() {
        hash.push_str(&header.name);
        hash.push_str(&header.value);
    }
    let body = String::from_utf8(request.body.as_ref().unwrap_or(&vec![]).clone())
        .expect("Raw response is not UTF-8 encoded.");
    hash.push_str(&body);
    let function_name = request
        .transform
        .as_ref()
        .map(|transform| transform.function.0.method.clone());
    if let Some(name) = function_name {
        hash.push_str(&name);
    }

    hash
}
