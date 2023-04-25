//! Helper functions and builders for creating HTTP requests and responses.

use super::{
    CanisterHttpRequestArgument, HttpHeader, HttpMethod, HttpResponse, TransformArgs,
    TransformContext,
};

/// Creates a new HTTP request builder.
pub fn create_request() -> CanisterHttpRequestArgumentBuilder {
    CanisterHttpRequestArgumentBuilder::new()
}

/// Represents a builder for a HTTP request.
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
    pub fn transform<T>(mut self, func: T, context: Vec<u8>) -> Self
    where
        T: Fn(TransformArgs) -> HttpResponse + 'static,
    {
        self.0.transform = Some(TransformContext::new(func, context));
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

/// Creates a new HTTP response builder.
pub fn create_response() -> HttpResponseBuilder {
    HttpResponseBuilder::new()
}

/// Represents a builder for a HTTP response.
#[derive(Debug)]
pub struct HttpResponseBuilder(HttpResponse);

impl HttpResponseBuilder {
    /// Creates a new HTTP response builder.
    pub fn new() -> Self {
        Self(HttpResponse {
            status: candid::Nat::from(200),
            headers: Vec::new(),
            body: Vec::new(),
        })
    }

    /// Sets the HTTP status code.
    pub fn status(mut self, status: u64) -> Self {
        self.0.status = candid::Nat::from(status);
        self
    }

    /// Adds a HTTP header to the HTTP response.
    pub fn header(mut self, header: HttpHeader) -> Self {
        self.0.headers.push(header);
        self
    }

    /// Sets the HTTP response body.
    pub fn body(mut self, body: &str) -> Self {
        self.0.body = body.as_bytes().to_vec();
        self
    }

    /// Builds the HTTP response.
    pub fn build(self) -> HttpResponse {
        self.0
    }
}

impl Default for HttpResponseBuilder {
    fn default() -> Self {
        Self::new()
    }
}
