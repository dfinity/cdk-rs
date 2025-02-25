use ic_cdk::management_canister::{
    http_request_with_closure_with_cycles, http_request_with_cycles, transform_context_from_query,
    HttpHeader, HttpMethod, HttpRequestArgs, HttpRequestResult, TransformArgs,
};
use ic_cdk::{query, update};

/// The formula to calculate the cost of a request.
fn cycles_cost(args: &HttpRequestArgs) -> u128 {
    const N: u128 = 13;
    let request_bytes_len = (args.url.len()
        + args
            .headers
            .iter()
            .map(|h| h.name.len() + h.value.len())
            .sum::<usize>()
        + args.body.as_ref().map(|b| b.len()).unwrap_or(0)
        + args
            .transform
            .as_ref()
            .map(|t| t.context.len() + t.function.0.method.len())
            .unwrap_or(0)) as u128;
    let response_bytes_len = args.max_response_bytes.unwrap_or(2_000_000) as u128;
    (3_000_000 + 60_000 * N) * N + 400 * N * request_bytes_len + 800 * N * response_bytes_len
}

/// All fields are Some except transform.
#[update]
async fn get_without_transform() {
    let args = HttpRequestArgs {
        url: "https://example.com".to_string(),
        method: HttpMethod::GET,
        headers: vec![HttpHeader {
            name: "request_header_name".to_string(),
            value: "request_header_value".to_string(),
        }],
        body: Some(vec![1]),
        max_response_bytes: Some(100_000),
        transform: None,
    };
    let cycles = cycles_cost(&args);
    let res = http_request_with_cycles(&args, cycles).await.unwrap();
    assert_eq!(res.status, 200u32);
    assert_eq!(
        res.headers,
        vec![HttpHeader {
            name: "response_header_name".to_string(),
            value: "response_header_value".to_string(),
        }]
    );
    assert_eq!(res.body, vec![42]);
}

/// Method is POST.
#[update]
async fn post() {
    let args = HttpRequestArgs {
        url: "https://example.com".to_string(),
        method: HttpMethod::POST,
        ..Default::default()
    };
    let cycles = cycles_cost(&args);
    http_request_with_cycles(&args, cycles).await.unwrap();
}

/// Method is HEAD.
#[update]
async fn head() {
    let args = HttpRequestArgs {
        url: "https://example.com".to_string(),
        method: HttpMethod::HEAD,
        ..Default::default()
    };
    let cycles = cycles_cost(&args);
    http_request_with_cycles(&args, cycles).await.unwrap();
}

/// The standard way to define a transform function.
///
/// It is a query endpoint that takes a TransformArgs and returns an HttpRequestResult.
#[query]
fn transform(args: TransformArgs) -> HttpRequestResult {
    let mut body = args.response.body;
    body.push(args.context[0]);
    HttpRequestResult {
        status: args.response.status,
        headers: args.response.headers,
        body,
    }
}

/// Set the transform field with the name of the transform query endpoint.
#[update]
async fn get_with_transform() {
    let args = HttpRequestArgs {
        url: "https://example.com".to_string(),
        method: HttpMethod::GET,
        transform: Some(transform_context_from_query(
            "transform".to_string(),
            vec![42],
        )),
        ..Default::default()
    };
    let cycles = cycles_cost(&args);
    let res = http_request_with_cycles(&args, cycles).await.unwrap();
    assert_eq!(res.status, 200u32);
    assert_eq!(
        res.headers,
        vec![HttpHeader {
            name: "response_header_name".to_string(),
            value: "response_header_value".to_string(),
        }]
    );
    // The first 42 is from the response body, the second 42 is from the transform context.
    assert_eq!(res.body, vec![42, 42]);
}

/// Set the transform field with a closure.
#[update]
async fn get_with_transform_closure() {
    let transform = |args: HttpRequestResult| {
        let mut body = args.body;
        body.push(42);
        HttpRequestResult {
            status: args.status,
            headers: args.headers,
            body,
        }
    };
    let args = HttpRequestArgs {
        url: "https://example.com".to_string(),
        method: HttpMethod::GET,
        transform: None,
        ..Default::default()
    };
    // The transform closure takes 40 bytes.
    let cycles = cycles_cost(&args) + 40 * 400 * 13;
    let res = http_request_with_closure_with_cycles(&args, transform, cycles)
        .await
        .unwrap();
    assert_eq!(res.status, 200u32);
    assert_eq!(
        res.headers,
        vec![HttpHeader {
            name: "response_header_name".to_string(),
            value: "response_header_value".to_string(),
        }]
    );
    // The first 42 is from the response body, the second 42 is from the transform closure.
    assert_eq!(res.body, vec![42, 42]);
}

fn main() {}
