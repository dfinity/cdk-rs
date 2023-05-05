use ic_cdk::api::management_canister::http_request::{HttpHeader, HttpResponse};

const STATUS_CODE_OK: u64 = 200;

/// Creates a new HTTP response builder.
pub fn create_response() -> HttpResponseBuilder {
    HttpResponseBuilder::new()
}

/// A builder for a HTTP response.
#[derive(Debug)]
pub struct HttpResponseBuilder(HttpResponse);

impl HttpResponseBuilder {
    /// Creates a new HTTP response builder.
    pub fn new() -> Self {
        Self(HttpResponse {
            status: candid::Nat::from(STATUS_CODE_OK),
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
    pub fn body(mut self, body: Vec<u8>) -> Self {
        self.0.body = body;
        self
    }

    /// Sets the HTTP response body text.
    pub fn body_str(mut self, body: &str) -> Self {
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
