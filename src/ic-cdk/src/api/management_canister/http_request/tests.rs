use super::*;

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
