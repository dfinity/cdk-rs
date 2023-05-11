//! Helper functions and builders for creating HTTP requests and responses.

use candid::Principal;
use ic_cdk::api::management_canister::http_request::{
    CanisterHttpRequestArgument, HttpHeader, HttpMethod, HttpResponse, TransformArgs,
    TransformContext, TransformFunc,
};

/// Creates a new HTTP request builder.
pub fn create_request() -> CanisterHttpRequestArgumentBuilder {
    CanisterHttpRequestArgumentBuilder::new()
}

/// A builder for a HTTP request.
#[derive(Debug)]
pub struct CanisterHttpRequestArgumentBuilder(CanisterHttpRequestArgument);

impl CanisterHttpRequestArgumentBuilder {
    /// Creates a new HTTP request builder.
    pub fn new() -> Self {
        Self(CanisterHttpRequestArgument {
            url: String::new(),
            max_response_bytes: None,
            method: HttpMethod::GET,
            headers: Vec::new(),
            body: None,
            transform: None,
        })
    }

    /// Sets the URL of the HTTP request.
    pub fn url(mut self, url: &str) -> Self {
        self.0.url = url.to_string();
        self
    }

    /// Sets the HTTP method to GET and the URL of the HTTP request.
    pub fn get(mut self, url: &str) -> Self {
        self.0.method = HttpMethod::GET;
        self.0.url = url.to_string();
        self
    }

    /// Sets the HTTP method to POST and the URL of the HTTP request.
    pub fn post(mut self, url: &str) -> Self {
        self.0.method = HttpMethod::POST;
        self.0.url = url.to_string();
        self
    }

    /// Sets the HTTP method to HEAD and the URL of the HTTP request.
    pub fn head(mut self, url: &str) -> Self {
        self.0.method = HttpMethod::HEAD;
        self.0.url = url.to_string();
        self
    }

    /// Sets the maximum response size in bytes.
    pub fn max_response_bytes(mut self, max_response_bytes: u64) -> Self {
        self.0.max_response_bytes = Some(max_response_bytes);
        self
    }

    /// Sets the HTTP method of the HTTP request.
    pub fn method(mut self, method: HttpMethod) -> Self {
        self.0.method = method;
        self
    }

    /// Adds a HTTP header to the HTTP request.
    pub fn header(mut self, name: String, value: String) -> Self {
        self.0.headers.push(HttpHeader { name, value });
        self
    }

    /// Sets the HTTP request body.
    pub fn body(mut self, body: Vec<u8>) -> Self {
        self.0.body = Some(body);
        self
    }

    /// Sets the transform function.
    pub fn transform<T>(mut self, candid_function_name: &str, func: T, context: Vec<u8>) -> Self
    where
        T: Fn(TransformArgs) -> HttpResponse + 'static,
    {
        self.0.transform = Some(create_transform_context(
            candid_function_name.to_string(),
            func,
            context,
        ));
        self
    }

    /// Builds the HTTP request.
    pub fn build(self) -> CanisterHttpRequestArgument {
        self.0
    }
}

impl Default for CanisterHttpRequestArgumentBuilder {
    fn default() -> Self {
        Self::new()
    }
}

fn create_transform_context<T>(
    candid_function_name: String,
    func: T,
    context: Vec<u8>,
) -> TransformContext
where
    T: Fn(TransformArgs) -> HttpResponse + 'static,
{
    #[cfg(target_arch = "wasm32")]
    {
        TransformContext::from_name(candid_function_name, context)
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        // crate::id() can not be called outside of canister, that's why for testing
        // it is replaced with Principal::management_canister().
        let principal = Principal::management_canister();
        super::storage::transform_function_insert(candid_function_name.clone(), Box::new(func));

        TransformContext {
            function: TransformFunc(candid::Func {
                principal,
                method: candid_function_name,
            }),
            context,
        }
    }
}

#[cfg(test)]
mod test {
    use ic_cdk::api::management_canister::http_request::{
        CanisterHttpRequestArgument, HttpResponse, TransformArgs,
    };

    /// Transform function which intentionally creates a new request passing
    /// itself as the target transform function.
    fn transform_function_with_overwrite(arg: TransformArgs) -> HttpResponse {
        create_request_with_transform();
        arg.response
    }

    /// Creates a request with a transform function which overwrites itself.
    fn create_request_with_transform() -> CanisterHttpRequestArgument {
        crate::create_request()
            .url("https://www.example.com")
            .transform(
                "transform_function_with_overwrite",
                transform_function_with_overwrite,
                vec![],
            )
            .build()
    }

    // IMPORTANT: If this test hangs check the implementation of inserting
    // transform function to the thread-local storage.
    //
    // This test simulates the case when transform function tries to
    // rewrite itself in a thread-local storage while it is being executed.
    // This may lead to a hang if the insertion to the thread-local storage
    // is not written properly.
    #[tokio::test]
    async fn test_transform_function_call_without_a_hang() {
        // Arrange
        let request = create_request_with_transform();
        let mock_response = crate::create_response().build();
        crate::mock::mock(request.clone(), Ok(mock_response));

        // Act
        let (response,) = crate::mock::http_request(request.clone()).await.unwrap();

        // Assert
        assert_eq!(response.status, 200);
        assert_eq!(crate::mock::times_called(request), 1);
    }
}
