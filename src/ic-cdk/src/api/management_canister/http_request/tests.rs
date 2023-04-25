use super::super::super::call::RejectionCode;
use super::*;
use std::time::{Duration, Instant};

const STATUS_CODE_OK: u64 = 200;
const STATUS_CODE_NOT_FOUND: u64 = 404;

#[tokio::test]
async fn test_http_request_no_transform() {
    // Arrange
    let body = "some text";
    let request = create_request().get("https://example.com").build();
    let mock_response = create_response().status(STATUS_CODE_OK).body(body).build();
    mock::mock(request.clone(), mock_response);

    // Act
    let (response,) = http_request(request.clone()).await.unwrap();

    // Assert
    assert_eq!(response.status, candid::Nat::from(STATUS_CODE_OK));
    assert_eq!(response.body, body.to_string().as_bytes().to_vec());
    assert_eq!(mock::times_called(request), 1);
}

#[tokio::test]
async fn test_http_request_called_several_times() {
    // Arrange
    let calls = 3;
    let body = "some text";
    let request = create_request().get("https://example.com").build();
    let mock_response = create_response().status(STATUS_CODE_OK).body(body).build();
    mock::mock(request.clone(), mock_response);

    // Act
    for _ in 0..calls {
        let (response,) = http_request(request.clone()).await.unwrap();
        assert_eq!(response.status, candid::Nat::from(STATUS_CODE_OK));
        assert_eq!(response.body, body.to_string().as_bytes().to_vec());
    }

    // Assert
    assert_eq!(mock::times_called(request), calls);
}

#[tokio::test]
async fn test_http_request_transform_status() {
    // Arrange
    fn transform(_arg: TransformArgs) -> HttpResponse {
        create_response().status(STATUS_CODE_NOT_FOUND).build()
    }
    let request = create_request()
        .get("https://example.com")
        .transform(transform, vec![])
        .build();
    let mock_response = create_response()
        .status(STATUS_CODE_OK)
        .body("some text")
        .build();
    mock::mock(request.clone(), mock_response);

    // Act
    let (response,) = http_request(request.clone()).await.unwrap();

    // Assert
    assert_ne!(response.status, candid::Nat::from(STATUS_CODE_OK));
    assert_eq!(response.status, candid::Nat::from(STATUS_CODE_NOT_FOUND));
    assert_eq!(mock::times_called(request), 1);
}

#[tokio::test]
async fn test_http_request_transform_body() {
    // Arrange
    const ORIGINAL_BODY: &str = "original body";
    const TRANSFORMED_BODY: &str = "transformed body";
    fn transform(_arg: TransformArgs) -> HttpResponse {
        create_response().body(TRANSFORMED_BODY).build()
    }
    let request = create_request()
        .get("https://dummyjson.com/todos/1")
        .transform(transform, vec![])
        .build();
    let mock_response = create_response()
        .status(STATUS_CODE_OK)
        .body(ORIGINAL_BODY)
        .build();
    mock::mock(request.clone(), mock_response);

    // Act
    let (response,) = http_request(request.clone()).await.unwrap();

    // Assert
    assert_ne!(response.body, ORIGINAL_BODY.as_bytes().to_vec());
    assert_eq!(response.body, TRANSFORMED_BODY.as_bytes().to_vec());
    assert_eq!(mock::times_called(request), 1);
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

    let request_1 = create_request()
        .get("https://dummyjson.com/todos/1")
        .transform(transform_status, vec![])
        .build();
    let mock_response_1 = create_response()
        .status(STATUS_CODE_NOT_FOUND)
        .body(ORIGINAL_BODY)
        .build();
    mock::mock(request_1.clone(), mock_response_1);

    let request_2 = create_request()
        .get("https://dummyjson.com/todos/2")
        .transform(transform_body, vec![])
        .build();
    let mock_response_2 = create_response()
        .status(STATUS_CODE_OK)
        .body(TRANSFORMED_BODY)
        .build();
    mock::mock(request_2.clone(), mock_response_2);

    // Act
    let futures = vec![
        http_request(request_1.clone()),
        http_request(request_2.clone()),
    ];
    let results = futures::future::join_all(futures).await;
    let responses: Vec<_> = results
        .into_iter()
        .filter(|result| result.is_ok())
        .map(|result| result.unwrap().0)
        .collect();

    // Assert
    assert_eq!(
        mock::registered_transform_function_names(),
        vec!["transform_body", "transform_status"]
    );
    assert_eq!(responses.len(), 2);
    assert_eq!(
        responses[0].status,
        candid::Nat::from(STATUS_CODE_NOT_FOUND)
    );
    assert_eq!(responses[0].body, ORIGINAL_BODY.as_bytes().to_vec());
    assert_eq!(responses[1].status, candid::Nat::from(STATUS_CODE_OK));
    assert_eq!(responses[1].body, TRANSFORMED_BODY.as_bytes().to_vec());
    assert_eq!(mock::times_called(request_1), 1);
    assert_eq!(mock::times_called(request_2), 1);
}

#[tokio::test]
async fn test_http_request_max_response_bytes_ok() {
    // Arrange
    let max_response_bytes = 3;
    let body_small_enough = "123";
    let request = create_request()
        .get("https://example.com")
        .max_response_bytes(max_response_bytes)
        .build();
    let mock_response = create_response()
        .status(STATUS_CODE_OK)
        .body(body_small_enough)
        .build();
    mock::mock(request.clone(), mock_response);

    // Act
    let result = http_request(request.clone()).await;

    // Assert
    assert!(result.is_ok());
    assert_eq!(mock::times_called(request), 1);
}

#[tokio::test]
async fn test_http_request_max_response_bytes_error() {
    // Arrange
    let max_response_bytes = 3;
    let body_too_big = "1234";
    let request = create_request()
        .get("https://example.com")
        .max_response_bytes(max_response_bytes)
        .build();
    let mock_response = create_response()
        .status(STATUS_CODE_OK)
        .body(body_too_big)
        .build();
    mock::mock(request.clone(), mock_response);

    // Act
    let result = http_request(request.clone()).await;

    // Assert
    assert!(result.is_err());
    assert_eq!(mock::times_called(request), 1);
}

#[tokio::test]
async fn test_http_request_sequentially() {
    // Arrange
    let request_a = create_request().get("a").build();
    let request_b = create_request().get("b").build();
    let request_c = create_request().get("c").build();
    let mock_response = create_response().status(STATUS_CODE_OK).build();
    mock::mock_with_delay(
        request_a.clone(),
        mock_response.clone(),
        Duration::from_millis(100),
    );
    mock::mock_with_delay(
        request_b.clone(),
        mock_response.clone(),
        Duration::from_millis(200),
    );
    mock::mock_with_delay(request_c.clone(), mock_response, Duration::from_millis(300));

    // Act
    let start = Instant::now();
    let _ = http_request(request_a.clone()).await;
    let _ = http_request(request_b.clone()).await;
    let _ = http_request(request_c.clone()).await;
    println!("All finished after {} s", start.elapsed().as_secs_f32());

    // Assert
    assert!(start.elapsed() > Duration::from_millis(500));
    assert_eq!(mock::times_called(request_a), 1);
    assert_eq!(mock::times_called(request_b), 1);
    assert_eq!(mock::times_called(request_c), 1);
}

#[tokio::test]
async fn test_http_request_concurrently() {
    // Arrange
    let request_a = create_request().get("a").build();
    let request_b = create_request().get("b").build();
    let request_c = create_request().get("c").build();
    let mock_response = create_response().status(STATUS_CODE_OK).build();
    mock::mock_with_delay(
        request_a.clone(),
        mock_response.clone(),
        Duration::from_millis(100),
    );
    mock::mock_with_delay(
        request_b.clone(),
        mock_response.clone(),
        Duration::from_millis(200),
    );
    mock::mock_with_delay(request_c.clone(), mock_response, Duration::from_millis(300));

    // Act
    let start = Instant::now();
    let futures = vec![
        http_request(request_a.clone()),
        http_request(request_b.clone()),
        http_request(request_c.clone()),
    ];
    futures::future::join_all(futures).await;
    println!("All finished after {} s", start.elapsed().as_secs_f32());

    // Assert
    assert!(start.elapsed() < Duration::from_millis(500));
    assert_eq!(mock::times_called(request_a), 1);
    assert_eq!(mock::times_called(request_b), 1);
    assert_eq!(mock::times_called(request_c), 1);
}

#[tokio::test]
async fn test_http_request_error() {
    // Arrange
    let request = create_request().get("https://example.com").build();
    let mock_error = (RejectionCode::SysFatal, "system fatal error".to_string());
    mock::mock_error(request.clone(), mock_error);

    // Act
    let result = http_request(request.clone()).await;

    // Assert
    assert_eq!(
        result,
        Err((RejectionCode::SysFatal, "system fatal error".to_string()))
    );
    assert_eq!(mock::times_called(request), 1);
}

#[tokio::test]
async fn test_http_request_error_with_delay() {
    // Arrange
    let request = create_request().get("https://example.com").build();
    let mock_error = (RejectionCode::SysFatal, "system fatal error".to_string());
    mock::mock_error_with_delay(request.clone(), mock_error, Duration::from_millis(200));

    // Act
    let start = Instant::now();
    let result = http_request(request.clone()).await;

    // Assert
    assert!(start.elapsed() > Duration::from_millis(100));
    assert_eq!(
        result,
        Err((RejectionCode::SysFatal, "system fatal error".to_string()))
    );
    assert_eq!(mock::times_called(request), 1);
}
