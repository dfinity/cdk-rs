//! Mocks HTTP requests.

use super::super::super::call::RejectionCode;
use super::storage;
use super::{CanisterHttpRequestArgument, HttpResponse, TransformArgs};
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

    /// Creates a new mock with a response.
    pub fn new_ok(
        request: CanisterHttpRequestArgument,
        response: HttpResponse,
        delay: Duration,
    ) -> Self {
        Self::new(request, Ok(response), delay)
    }

    /// Creates a new mock with an error.
    pub fn new_err(
        request: CanisterHttpRequestArgument,
        error: MockError,
        delay: Duration,
    ) -> Self {
        Self::new(request, Err(error), delay)
    }
}

/// Mocks a HTTP request.
pub fn mock(request: CanisterHttpRequestArgument, response: HttpResponse) {
    mock_with_delay(request, response, Duration::from_secs(0));
}

/// Mocks a HTTP request with a delay.
pub fn mock_with_delay(
    request: CanisterHttpRequestArgument,
    response: HttpResponse,
    delay: Duration,
) {
    storage::mock_insert(Mock::new_ok(request, response, delay));
}

/// Mocks a HTTP request with an error.
pub fn mock_error(request: CanisterHttpRequestArgument, error: (RejectionCode, String)) {
    mock_error_with_delay(request, error, Duration::from_secs(0));
}

/// Mocks a HTTP request with an error and a delay.
pub fn mock_error_with_delay(
    request: CanisterHttpRequestArgument,
    error: (RejectionCode, String),
    delay: Duration,
) {
    storage::mock_insert(Mock::new_err(request, error, delay));
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

/// Handles incoming HTTP requests by retrieving a mock response based
/// on the request, possibly delaying the response, transforming the response if necessary,
/// and returning it. If there is no mock found, it returns an error.
pub(crate) async fn http_request(
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
    let transformed_response = call_transform_function(
        mock.request,
        TransformArgs {
            response: mock_response.clone(),
            context: vec![],
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
