use ic_cdk::api::management_canister::http_request::{
    CanisterHttpRequestArgument, HttpResponse, TransformArgs,
};

/// Transform the response body by extracting the author from the JSON response.
#[ic_cdk_macros::query]
fn transform_quote(raw: TransformArgs) -> HttpResponse {
    let mut response = HttpResponse {
        status: raw.response.status.clone(),
        ..Default::default()
    };
    if response.status == 200 {
        let original = parse_json(raw.response.body);
        let transformed = original["author"].as_str().unwrap_or_default();
        response.body = transformed.to_string().into_bytes();
    } else {
        print(&format!("Transform error: err = {:?}", raw));
    }
    response
}

/// Create a quote request with transformation function.
fn build_quote_request(url: &str) -> CanisterHttpRequestArgument {
    ic_cdk_http_kit::create_request()
        .get(url)
        .header(
            "User-Agent".to_string(),
            "ic-http-outcall-kit-example".to_string(),
        )
        .transform(transform_quote, vec![])
        .build()
}

/// Fetch data by making an HTTP request.
async fn fetch(request: CanisterHttpRequestArgument) -> String {
    match ic_cdk_http_kit::http_request(request).await {
        Ok((response,)) => {
            if response.status == 200 {
                format!("Response: {:?}", String::from_utf8(response.body).unwrap())
            } else {
                format!("Unexpected status: {:?}", response.status)
            }
        }
        Err((code, msg)) => {
            format!("Error: {:?} {:?}", code, msg)
        }
    }
}

/// Fetch a quote from the dummyjson.com API.
#[ic_cdk_macros::update]
async fn fetch_quote() -> String {
    let request = build_quote_request("https://dummyjson.com/quotes/1");
    fetch(request).await
}

/// Parse the raw response body as JSON.
fn parse_json(body: Vec<u8>) -> serde_json::Value {
    let json_str = String::from_utf8(body).expect("Raw response is not UTF-8 encoded.");
    serde_json::from_str(&json_str).expect("Failed to parse JSON from string")
}

/// Print a message to the console.
fn print(msg: &str) {
    #[cfg(target_arch = "wasm32")]
    ic_cdk::api::print(msg);

    #[cfg(not(target_arch = "wasm32"))]
    println!("{}", msg);
}

fn main() {}

#[cfg(test)]
mod test {
    use super::*;
    use ic_cdk::api::call::RejectionCode;

    // Test http_request returns an author after modifying the response body.
    #[tokio::test]
    async fn test_http_request_transform_body_quote() {
        // Arrange
        let request = build_quote_request("https://dummyjson.com/quotes/1");
        let mock_response = ic_cdk_http_kit::create_response()
            .status(200)
            .body(r#"{"quote": "Be yourself; everyone else is taken.", "author": "Oscar Wilde"}"#)
            .build();
        ic_cdk_http_kit::mock(request.clone(), Ok(mock_response));

        // Act
        let result = fetch(request.clone()).await;

        // Assert
        assert_eq!(result, r#"Response: "Oscar Wilde""#.to_string());
        assert_eq!(ic_cdk_http_kit::times_called(request), 1);
    }

    // Test http_request returns a system fatal error.
    #[tokio::test]
    async fn test_http_request_transform_body_quote_error() {
        // Arrange
        let request = build_quote_request("https://dummyjson.com/quotes/1");
        let mock_error = (RejectionCode::SysFatal, "fatal".to_string());
        ic_cdk_http_kit::mock(request.clone(), Err(mock_error));

        // Act
        let result = fetch(request.clone()).await;

        // Assert
        assert_eq!(result, r#"Error: SysFatal "fatal""#.to_string());
        assert_eq!(ic_cdk_http_kit::times_called(request), 1);
    }

    // Test http_request returns a response with status 404.
    #[tokio::test]
    async fn test_http_request_transform_body_quote_404() {
        // Arrange
        let request = build_quote_request("https://dummyjson.com/quotes/1");
        let mock_response = ic_cdk_http_kit::create_response().status(404).build();
        ic_cdk_http_kit::mock(request.clone(), Ok(mock_response));

        // Act
        let result = fetch(request.clone()).await;

        // Assert
        assert_eq!(result, "Unexpected status: Nat(404)".to_string());
        assert_eq!(ic_cdk_http_kit::times_called(request), 1);
    }
}
