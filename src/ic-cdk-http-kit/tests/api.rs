use ic_cdk::api::call::RejectionCode;
use ic_cdk::api::management_canister::http_request::{
    CanisterHttpRequestArgument, HttpResponse, TransformArgs,
};
use std::time::{Duration, Instant};

const STATUS_CODE_OK: u64 = 200;
const STATUS_CODE_NOT_FOUND: u64 = 404;

#[tokio::test]
async fn test_http_request_no_transform() {
    // Arrange
    let body = "some text";
    let request = ic_cdk_http_kit::create_request()
        .get("https://example.com")
        .build();
    let mock_response = ic_cdk_http_kit::create_response()
        .status(STATUS_CODE_OK)
        .body_str(body)
        .build();
    ic_cdk_http_kit::mock(request.clone(), Ok(mock_response));

    // Act
    let (response,) = ic_cdk_http_kit::http_request(request.clone())
        .await
        .unwrap();

    // Assert
    assert_eq!(response.status, STATUS_CODE_OK);
    assert_eq!(response.body, body.to_owned().into_bytes());
    assert_eq!(ic_cdk_http_kit::times_called(request), 1);
}

#[tokio::test]
async fn test_http_request_called_several_times() {
    // Arrange
    let calls = 3;
    let body = "some text";
    let request = ic_cdk_http_kit::create_request()
        .get("https://example.com")
        .build();
    let mock_response = ic_cdk_http_kit::create_response()
        .status(STATUS_CODE_OK)
        .body_str(body)
        .build();
    ic_cdk_http_kit::mock(request.clone(), Ok(mock_response));

    // Act
    for _ in 0..calls {
        let (response,) = ic_cdk_http_kit::http_request(request.clone())
            .await
            .unwrap();
        assert_eq!(response.status, STATUS_CODE_OK);
        assert_eq!(response.body, body.to_owned().into_bytes());
    }

    // Assert
    assert_eq!(ic_cdk_http_kit::times_called(request), calls);
}

#[tokio::test]
async fn test_http_request_transform_status() {
    // Arrange
    fn transform(_arg: TransformArgs) -> HttpResponse {
        ic_cdk_http_kit::create_response()
            .status(STATUS_CODE_NOT_FOUND)
            .build()
    }
    let request = ic_cdk_http_kit::create_request()
        .get("https://example.com")
        .transform(transform, vec![])
        .build();
    let mock_response = ic_cdk_http_kit::create_response()
        .status(STATUS_CODE_OK)
        .body_str("some text")
        .build();
    ic_cdk_http_kit::mock(request.clone(), Ok(mock_response));

    // Act
    let (response,) = ic_cdk_http_kit::http_request(request.clone())
        .await
        .unwrap();

    // Assert
    assert_eq!(response.status, STATUS_CODE_NOT_FOUND);
    assert_eq!(ic_cdk_http_kit::times_called(request), 1);
}

#[tokio::test]
async fn test_http_request_transform_body() {
    // Arrange
    const ORIGINAL_BODY: &str = "original body";
    const TRANSFORMED_BODY: &str = "transformed body";
    fn transform(_arg: TransformArgs) -> HttpResponse {
        ic_cdk_http_kit::create_response()
            .body_str(TRANSFORMED_BODY)
            .build()
    }
    let request = ic_cdk_http_kit::create_request()
        .get("https://dummyjson.com/todos/1")
        .transform(transform, vec![])
        .build();
    let mock_response = ic_cdk_http_kit::create_response()
        .status(STATUS_CODE_OK)
        .body_str(ORIGINAL_BODY)
        .build();
    ic_cdk_http_kit::mock(request.clone(), Ok(mock_response));

    // Act
    let (response,) = ic_cdk_http_kit::http_request(request.clone())
        .await
        .unwrap();

    // Assert
    assert_eq!(response.body, TRANSFORMED_BODY.as_bytes().to_vec());
    assert_eq!(ic_cdk_http_kit::times_called(request), 1);
}

#[tokio::test]
async fn test_http_request_transform_context() {
    // Arrange
    fn transform_context_to_body_text(arg: TransformArgs) -> HttpResponse {
        HttpResponse {
            body: arg.context,
            ..arg.response
        }
    }
    let request = ic_cdk_http_kit::create_request()
        .get("https://dummyjson.com/todos/1")
        .transform(
            transform_context_to_body_text,
            "some context".as_bytes().to_vec(),
        )
        .build();
    let mock_response = ic_cdk_http_kit::create_response()
        .status(STATUS_CODE_OK)
        .body_str("some context")
        .build();
    ic_cdk_http_kit::mock(request.clone(), Ok(mock_response));

    // Act
    let (response,) = ic_cdk_http_kit::http_request(request.clone())
        .await
        .unwrap();

    // Assert
    assert_eq!(response.body, "some context".as_bytes().to_vec());
    assert_eq!(ic_cdk_http_kit::times_called(request), 1);
}

#[tokio::test]
async fn test_http_request_transform_both_status_and_body() {
    // Arrange
    const ORIGINAL_BODY: &str = "original body";
    const TRANSFORMED_BODY: &str = "transformed body";

    fn transform_status(arg: TransformArgs) -> HttpResponse {
        let mut response = arg.response;
        response.status = candid::Nat::from(STATUS_CODE_NOT_FOUND);
        response
    }

    fn transform_body(arg: TransformArgs) -> HttpResponse {
        let mut response = arg.response;
        response.body = TRANSFORMED_BODY.as_bytes().to_vec();
        response
    }

    let request_1 = ic_cdk_http_kit::create_request()
        .get("https://dummyjson.com/todos/1")
        .transform(transform_status, vec![])
        .build();
    let mock_response_1 = ic_cdk_http_kit::create_response()
        .status(STATUS_CODE_NOT_FOUND)
        .body_str(ORIGINAL_BODY)
        .build();
    ic_cdk_http_kit::mock(request_1.clone(), Ok(mock_response_1));

    let request_2 = ic_cdk_http_kit::create_request()
        .get("https://dummyjson.com/todos/2")
        .transform(transform_body, vec![])
        .build();
    let mock_response_2 = ic_cdk_http_kit::create_response()
        .status(STATUS_CODE_OK)
        .body_str(TRANSFORMED_BODY)
        .build();
    ic_cdk_http_kit::mock(request_2.clone(), Ok(mock_response_2));

    // Act
    let futures = vec![
        ic_cdk_http_kit::http_request(request_1.clone()),
        ic_cdk_http_kit::http_request(request_2.clone()),
    ];
    let results = futures::future::join_all(futures).await;
    let responses: Vec<_> = results
        .into_iter()
        .filter(|result| result.is_ok())
        .map(|result| result.unwrap().0)
        .collect();

    // Assert
    assert_eq!(
        ic_cdk_http_kit::registered_transform_function_names(),
        vec!["transform_body", "transform_status"]
    );
    assert_eq!(responses.len(), 2);
    assert_eq!(responses[0].status, STATUS_CODE_NOT_FOUND);
    assert_eq!(responses[0].body, ORIGINAL_BODY.as_bytes().to_vec());
    assert_eq!(responses[1].status, STATUS_CODE_OK);
    assert_eq!(responses[1].body, TRANSFORMED_BODY.as_bytes().to_vec());
    assert_eq!(ic_cdk_http_kit::times_called(request_1), 1);
    assert_eq!(ic_cdk_http_kit::times_called(request_2), 1);
}

#[tokio::test]
async fn test_http_request_max_response_bytes_ok() {
    // Arrange
    let max_response_bytes = 3;
    let body_small_enough = "123";
    let request = ic_cdk_http_kit::create_request()
        .get("https://example.com")
        .max_response_bytes(max_response_bytes)
        .build();
    let mock_response = ic_cdk_http_kit::create_response()
        .status(STATUS_CODE_OK)
        .body_str(body_small_enough)
        .build();
    ic_cdk_http_kit::mock(request.clone(), Ok(mock_response));

    // Act
    let result = ic_cdk_http_kit::http_request(request.clone()).await;

    // Assert
    assert!(result.is_ok());
    assert_eq!(ic_cdk_http_kit::times_called(request), 1);
}

#[tokio::test]
async fn test_http_request_max_response_bytes_error() {
    // Arrange
    let max_response_bytes = 3;
    let body_too_big = "1234";
    let request = ic_cdk_http_kit::create_request()
        .get("https://example.com")
        .max_response_bytes(max_response_bytes)
        .build();
    let mock_response = ic_cdk_http_kit::create_response()
        .status(STATUS_CODE_OK)
        .body_str(body_too_big)
        .build();
    ic_cdk_http_kit::mock(request.clone(), Ok(mock_response));

    // Act
    let result = ic_cdk_http_kit::http_request(request.clone()).await;

    // Assert
    assert!(result.is_err());
    assert_eq!(ic_cdk_http_kit::times_called(request), 1);
}

#[tokio::test]
async fn test_http_request_sequentially() {
    // Arrange
    let request_a = ic_cdk_http_kit::create_request().get("a").build();
    let request_b = ic_cdk_http_kit::create_request().get("b").build();
    let request_c = ic_cdk_http_kit::create_request().get("c").build();
    let mock_response = ic_cdk_http_kit::create_response()
        .status(STATUS_CODE_OK)
        .build();
    ic_cdk_http_kit::mock_with_delay(
        request_a.clone(),
        Ok(mock_response.clone()),
        Duration::from_millis(100),
    );
    ic_cdk_http_kit::mock_with_delay(
        request_b.clone(),
        Ok(mock_response.clone()),
        Duration::from_millis(200),
    );
    ic_cdk_http_kit::mock_with_delay(
        request_c.clone(),
        Ok(mock_response),
        Duration::from_millis(300),
    );

    // Act
    let start = Instant::now();
    let _ = ic_cdk_http_kit::http_request(request_a.clone()).await;
    let _ = ic_cdk_http_kit::http_request(request_b.clone()).await;
    let _ = ic_cdk_http_kit::http_request(request_c.clone()).await;
    println!("All finished after {} s", start.elapsed().as_secs_f32());

    // Assert
    assert!(start.elapsed() > Duration::from_millis(500));
    assert_eq!(ic_cdk_http_kit::times_called(request_a), 1);
    assert_eq!(ic_cdk_http_kit::times_called(request_b), 1);
    assert_eq!(ic_cdk_http_kit::times_called(request_c), 1);
}

#[tokio::test]
async fn test_http_request_concurrently() {
    // Arrange
    let request_a = ic_cdk_http_kit::create_request().get("a").build();
    let request_b = ic_cdk_http_kit::create_request().get("b").build();
    let request_c = ic_cdk_http_kit::create_request().get("c").build();
    let mock_response = ic_cdk_http_kit::create_response()
        .status(STATUS_CODE_OK)
        .build();
    ic_cdk_http_kit::mock_with_delay(
        request_a.clone(),
        Ok(mock_response.clone()),
        Duration::from_millis(100),
    );
    ic_cdk_http_kit::mock_with_delay(
        request_b.clone(),
        Ok(mock_response.clone()),
        Duration::from_millis(200),
    );
    ic_cdk_http_kit::mock_with_delay(
        request_c.clone(),
        Ok(mock_response),
        Duration::from_millis(300),
    );

    // Act
    let start = Instant::now();
    let futures = vec![
        ic_cdk_http_kit::http_request(request_a.clone()),
        ic_cdk_http_kit::http_request(request_b.clone()),
        ic_cdk_http_kit::http_request(request_c.clone()),
    ];
    futures::future::join_all(futures).await;
    println!("All finished after {} s", start.elapsed().as_secs_f32());

    // Assert
    assert!(start.elapsed() < Duration::from_millis(500));
    assert_eq!(ic_cdk_http_kit::times_called(request_a), 1);
    assert_eq!(ic_cdk_http_kit::times_called(request_b), 1);
    assert_eq!(ic_cdk_http_kit::times_called(request_c), 1);
}

#[tokio::test]
async fn test_http_request_error() {
    // Arrange
    let request = ic_cdk_http_kit::create_request()
        .get("https://example.com")
        .build();
    let mock_error = (RejectionCode::SysFatal, "system fatal error".to_string());
    ic_cdk_http_kit::mock(request.clone(), Err(mock_error));

    // Act
    let result = ic_cdk_http_kit::http_request(request.clone()).await;

    // Assert
    assert_eq!(
        result,
        Err((RejectionCode::SysFatal, "system fatal error".to_string()))
    );
    assert_eq!(ic_cdk_http_kit::times_called(request), 1);
}

#[tokio::test]
async fn test_http_request_error_with_delay() {
    // Arrange
    let request = ic_cdk_http_kit::create_request()
        .get("https://example.com")
        .build();
    let mock_error = (RejectionCode::SysFatal, "system fatal error".to_string());
    ic_cdk_http_kit::mock_with_delay(request.clone(), Err(mock_error), Duration::from_millis(200));

    // Act
    let start = Instant::now();
    let result = ic_cdk_http_kit::http_request(request.clone()).await;

    // Assert
    assert!(start.elapsed() > Duration::from_millis(100));
    assert_eq!(
        result,
        Err((RejectionCode::SysFatal, "system fatal error".to_string()))
    );
    assert_eq!(ic_cdk_http_kit::times_called(request), 1);
}

/// Transform function which intentionally creates a new request passing
/// itself as the target transform function.
fn transform_function_with_overwrite(arg: TransformArgs) -> HttpResponse {
    create_request_with_transform();
    arg.response
}

/// Creates a request with a transform function which overwrites itself.
fn create_request_with_transform() -> CanisterHttpRequestArgument {
    ic_cdk_http_kit::create_request()
        .url("https://www.example.com")
        .transform(transform_function_with_overwrite, vec![])
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
    let mock_response = ic_cdk_http_kit::create_response().build();
    ic_cdk_http_kit::mock(request.clone(), Ok(mock_response));

    // Act
    let (response,) = ic_cdk_http_kit::http_request(request.clone())
        .await
        .unwrap();

    // Assert
    assert_eq!(response.status, 200);
    assert_eq!(ic_cdk_http_kit::times_called(request), 1);
    assert_eq!(
        ic_cdk_http_kit::registered_transform_function_names(),
        vec!["transform_function_with_overwrite"]
    );
}
