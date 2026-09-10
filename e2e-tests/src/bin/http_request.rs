use candid::Reserved;
use ic_cdk::{query, update};
use ic_cdk_management_canister::{
    FlexibleHttpGlobalError, FlexibleHttpRequest, FlexibleHttpRequestResult, HttpHeader,
    HttpMethod, HttpRequest, HttpRequestResult, ReplicationCounts, TransformArgs,
    transform_context_from_query,
};

fn expected_response_headers() -> Vec<HttpHeader> {
    vec![HttpHeader {
        name: "response_header_name".to_string(),
        value: "response_header_value".to_string(),
    }]
}

/// All fields are set except transform.
#[update]
async fn get_without_transform() {
    let res = HttpRequest::new("https://example.com")
        .with_method(HttpMethod::GET)
        .with_header("request_header_name", "request_header_value")
        .with_body(vec![1])
        .with_max_response_bytes(100_000)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status, 200u32);
    assert_eq!(res.headers, expected_response_headers());
    assert_eq!(res.body, vec![42]);
}

/// Method is POST.
#[update]
async fn post() {
    HttpRequest::new("https://example.com")
        .with_method(HttpMethod::POST)
        .send()
        .await
        .unwrap();
}

/// Method is HEAD.
#[update]
async fn head() {
    HttpRequest::new("https://example.com")
        .with_method(HttpMethod::HEAD)
        .send()
        .await
        .unwrap();
}

/// The standard way to define a transform function.
///
/// It is a query method that takes a `TransformArgs` and returns an `HttpRequestResult`.
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

/// Set the transform with the name of the transform query method.
#[update]
async fn get_with_transform() {
    let res = HttpRequest::new("https://example.com")
        .with_transform(transform_context_from_query(
            "transform".to_string(),
            vec![42],
        ))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status, 200u32);
    assert_eq!(res.headers, expected_response_headers());
    // The first 42 is from the response body, the second 42 is from the transform context.
    assert_eq!(res.body, vec![42, 42]);
}

/// Set the transform with a closure.
#[update]
async fn get_with_transform_closure() {
    let res = HttpRequest::new("https://example.com")
        .with_transform_closure(|args: HttpRequestResult| {
            let mut body = args.body;
            body.push(42);
            HttpRequestResult {
                status: args.status,
                headers: args.headers,
                body,
            }
        })
        .send()
        .await
        .unwrap();
    assert_eq!(res.status, 200u32);
    assert_eq!(res.headers, expected_response_headers());
    // The first 42 is from the response body, the second 42 is from the transform closure.
    assert_eq!(res.body, vec![42, 42]);
}

/// Non replicated HTTP request.
#[update]
async fn non_replicated() {
    HttpRequest::new("https://example.com")
        .with_method(HttpMethod::GET)
        .non_replicated()
        .send()
        .await
        .unwrap();
}

/// Narrowing the expected resource usage must lower the cycles reservation.
#[update]
async fn expected_usage_lowers_cost() {
    let worst_case = HttpRequest::new("https://example.com")
        .with_max_response_bytes(4_000)
        .get_cost();
    let expected = HttpRequest::new("https://example.com")
        .with_max_response_bytes(4_000)
        .with_expected_roundtrip_time_ms(300)
        .with_expected_transform_instructions(1_000_000)
        .get_cost();
    assert!(
        expected < worst_case,
        "expected {expected} should be below worst case {worst_case}"
    );
}

/// Flexible outcall with the default replication counts.
///
/// Returns how many responses were delivered, so that the test can check it against the
/// committee it answered.
#[update]
async fn flexible_default() -> u32 {
    let res = FlexibleHttpRequest::new("https://example.com")
        .with_method(HttpMethod::GET)
        .with_header("request_header_name", "request_header_value")
        .with_max_response_bytes(100_000)
        .send()
        .await
        .unwrap();
    let FlexibleHttpRequestResult::Ok(responses) = res else {
        panic!("expected responses, got {res:?}");
    };
    assert!(!responses.is_empty());
    for response in &responses {
        assert_eq!(response.status, 200u32);
        assert_eq!(response.headers, expected_response_headers());
        assert_eq!(response.body, vec![42]);
    }
    responses.len() as u32
}

/// Flexible outcall from a committee of one, whose single response must be delivered.
#[update]
async fn flexible_single() {
    let res = FlexibleHttpRequest::new("https://example.com")
        .with_replication(ReplicationCounts {
            min_responses: 1,
            max_responses: 1,
            total_requests: 1,
        })
        .send()
        .await
        .unwrap();
    let FlexibleHttpRequestResult::Ok(responses) = res else {
        panic!("expected responses, got {res:?}");
    };
    assert_eq!(responses.len(), 1);
    assert_eq!(responses[0].body, vec![42]);
}

/// Each node runs the transform on its own response.
#[update]
async fn flexible_with_transform_closure() {
    let res = FlexibleHttpRequest::new("https://example.com")
        .with_replication(ReplicationCounts {
            min_responses: 1,
            max_responses: 1,
            total_requests: 1,
        })
        .with_transform_closure(|args: HttpRequestResult| {
            let mut body = args.body;
            body.push(42);
            HttpRequestResult {
                status: args.status,
                headers: args.headers,
                body,
            }
        })
        .send()
        .await
        .unwrap();
    let FlexibleHttpRequestResult::Ok(responses) = res else {
        panic!("expected responses, got {res:?}");
    };
    assert_eq!(responses.len(), 1);
    // The first 42 is from the response body, the second 42 is from the transform closure.
    assert_eq!(responses[0].body, vec![42, 42]);
}

/// A committee that only rejects reports `too_many_rejects`, and does so as a reply.
#[update]
async fn flexible_too_many_rejects() {
    let res = FlexibleHttpRequest::new("https://example.com")
        .with_replication(ReplicationCounts {
            min_responses: 1,
            max_responses: 1,
            total_requests: 1,
        })
        .send()
        .await
        .expect("the outcall itself must not be rejected");
    let FlexibleHttpRequestResult::Err(err) = res else {
        panic!("expected an error result, got {res:?}");
    };
    assert_eq!(
        err.global_error,
        Some(FlexibleHttpGlobalError::TooManyRejects(Reserved))
    );
}

/// Narrowing the expected resource usage must lower the cycles reservation here too.
#[update]
async fn flexible_expected_usage_lowers_cost() {
    let worst_case = FlexibleHttpRequest::new("https://example.com")
        .with_max_response_bytes(4_000)
        .get_cost();
    let expected = FlexibleHttpRequest::new("https://example.com")
        .with_max_response_bytes(4_000)
        .with_expected_roundtrip_time_ms(300)
        .with_expected_transform_instructions(1_000_000)
        .get_cost();
    assert!(
        expected < worst_case,
        "expected {expected} should be below worst case {worst_case}"
    );
}

fn main() {}
