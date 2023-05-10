//! A simple toolkit for constructing and testing HTTP Outcalls on the Internet Computer.
//!
//! It streamlines unit testing of HTTP Outcalls and provides user-friendly utilities.
//! The crate simulates the `http_request` function from `ic_cdk` by retrieving mock responses, checking the maximum allowed size, and applying a transformation function if specified, optionally with a delay to simulate latency.
//!
//! Note: To properly simulate the transformation function inside `ic_cdk_http_kit::http_request`, the request builder must be used.
//!
//! ## Features
//!
//! - Simple interface for creating HTTP requests and responses
//! - Support for HTTP response transformation functions
//! - Control over response size with a maximum byte limit
//! - Mock response with optional delay to simulate latency
//! - Assert the number of times a request was called
//!
//! ## Examples
//!
//! ### Creating a Request
//!
//! ```rust
//! # use ic_cdk::api::management_canister::http_request::{TransformArgs, HttpResponse};
//! fn transform_function(arg: TransformArgs) -> HttpResponse {
//!     // Modify arg.response here
//!     arg.response
//! }
//!
//! let request = ic_cdk_http_kit::create_request()
//!     .get("https://dummyjson.com/todos/1")
//!     .max_response_bytes(1_024)
//!     .transform(transform_function, vec![])
//!     .build();
//! ```
//!
//! ### Creating a Response
//!
//! ```rust
//! let mock_response = ic_cdk_http_kit::create_response()
//!     .status(200)
//!     .body_str("some text")
//!     .build();
//! ```
//!
//! ### Mocking
//!
//! ```rust
//! # use std::time::Duration;
//! # use ic_cdk::api::call::RejectionCode;
//! # let request = ic_cdk_http_kit::create_request().build();
//! # let mock_response = ic_cdk_http_kit::create_response().build();
//! ic_cdk_http_kit::mock(request.clone(), Ok(mock_response.clone()));
//! ic_cdk_http_kit::mock_with_delay(request.clone(), Ok(mock_response.clone()), Duration::from_secs(2));
//!
//! let mock_error = (RejectionCode::SysFatal, "system fatal error".to_string());
//! ic_cdk_http_kit::mock(request.clone(), Err(mock_error.clone()));
//! ic_cdk_http_kit::mock_with_delay(request.clone(), Err(mock_error.clone()), Duration::from_secs(2));
//! ```
//!
//! ### Making an HTTP Outcall
//!
//! ```ignore
//! # // Ignored since this is an async function.
//! let (response,) = ic_cdk_http_kit::http_request(request).await.unwrap();
//! ```
//!
//! ### Asserts
//!
//! ```no_run
//! # // `no_run` since it would require to call an async function to count the number of calls.
//! # use ic_cdk::api::management_canister::http_request::HttpResponse;
//! # let request = ic_cdk_http_kit::create_request().build();
//! # let response = ic_cdk_http_kit::create_response().build();
//! assert_eq!(response.status, 200);
//! assert_eq!(response.body, "transformed body".to_owned().into_bytes());
//! assert_eq!(ic_cdk_http_kit::times_called(request), 1);
//! ```
//!
//! ### More Examples
//!
//! Please refer to the provided usage examples in the [tests](./tests) or [examples](./examples) directories.
//!
//! ## References
//!
//! - [Integrations](https://internetcomputer.org/docs/current/developer-docs/integrations/)
//! - [HTTPS Outcalls](https://internetcomputer.org/docs/current/developer-docs/integrations/http_requests/)
//! - HTTP Outcalls, [IC method http_request](https://internetcomputer.org/docs/current/references/ic-interface-spec#ic-http_request)
//! - Serving HTTP responses, [The HTTP Gateway protocol](https://internetcomputer.org/docs/current/references/ic-interface-spec#http-gateway)
//! - [Transformation Function](https://internetcomputer.org/docs/current/developer-docs/integrations/http_requests/http_requests-how-it-works#transformation-function)
//!

mod mock;
mod request;
mod response;
mod storage;

pub use mock::*;
pub use request::*;
pub use response::*;
