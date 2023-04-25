//! Helper functions and builders for creating HTTP requests and responses.

use super::{
    CanisterHttpRequestArgument, HttpHeader, HttpMethod, HttpResponse, TransformArgs,
    TransformContext,
};

pub fn create_request() -> CanisterHttpRequestArgumentBuilder {
    CanisterHttpRequestArgumentBuilder::new()
}

pub struct CanisterHttpRequestArgumentBuilder(CanisterHttpRequestArgument);

impl CanisterHttpRequestArgumentBuilder {
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

    pub fn url(mut self, url: String) -> Self {
        self.0.url = url;
        self
    }

    pub fn get(mut self, url: &str) -> Self {
        self.0.method = HttpMethod::GET;
        self.0.url = url.to_string();
        self
    }

    pub fn post(mut self, url: &str) -> Self {
        self.0.method = HttpMethod::POST;
        self.0.url = url.to_string();
        self
    }

    pub fn head(mut self, url: &str) -> Self {
        self.0.method = HttpMethod::HEAD;
        self.0.url = url.to_string();
        self
    }

    pub fn max_response_bytes(mut self, max_response_bytes: u64) -> Self {
        self.0.max_response_bytes = Some(max_response_bytes);
        self
    }

    pub fn method(mut self, method: HttpMethod) -> Self {
        self.0.method = method;
        self
    }

    pub fn header(mut self, name: String, value: String) -> Self {
        self.0.headers.push(HttpHeader { name, value });
        self
    }

    pub fn body(mut self, body: Vec<u8>) -> Self {
        self.0.body = Some(body);
        self
    }

    pub fn transform<T>(mut self, func: T, context: Vec<u8>) -> Self
    where
        T: Fn(TransformArgs) -> HttpResponse + 'static,
    {
        self.0.transform = Some(TransformContext::new(func, context));
        self
    }

    pub fn build(self) -> CanisterHttpRequestArgument {
        self.0
    }
}

impl Default for CanisterHttpRequestArgumentBuilder {
    fn default() -> Self {
        Self::new()
    }
}

pub fn create_response() -> HttpResponseBuilder {
    HttpResponseBuilder::new()
}

pub struct HttpResponseBuilder(HttpResponse);

impl HttpResponseBuilder {
    pub fn new() -> Self {
        Self(HttpResponse {
            status: candid::Nat::from(200),
            headers: Vec::new(),
            body: Vec::new(),
        })
    }

    pub fn status(mut self, status: u64) -> Self {
        self.0.status = candid::Nat::from(status);
        self
    }

    pub fn header(mut self, header: HttpHeader) -> Self {
        self.0.headers.push(header);
        self
    }

    pub fn body(mut self, body: &str) -> Self {
        self.0.body = body.as_bytes().to_vec();
        self
    }

    pub fn build(self) -> HttpResponse {
        self.0
    }
}

impl Default for HttpResponseBuilder {
    fn default() -> Self {
        Self::new()
    }
}
