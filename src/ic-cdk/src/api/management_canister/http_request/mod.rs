//! Canister HTTP request.
pub mod helpers;
pub mod lib;
pub mod mock;
mod storage;

pub use helpers::*;
pub use lib::*;

#[cfg(test)]
mod test {
    use super::*;

    /// A test transform function.
    fn transform_function_1(arg: TransformArgs) -> HttpResponse {
        arg.response
    }

    /// A test transform function.
    fn transform_function_2(arg: TransformArgs) -> HttpResponse {
        arg.response
    }

    /// Inserts the provided transform function into a thread-local hashmap.
    fn insert<T>(f: T)
    where
        T: Fn(TransformArgs) -> HttpResponse + 'static,
    {
        let name = get_function_name(&f).to_string();
        storage::transform_function_insert(name, Box::new(f));
    }

    /// This test makes sure that transform function names are preserved
    /// when passing to the function.
    #[test]
    fn test_transform_function_names() {
        // Arrange.
        insert(transform_function_1);
        insert(transform_function_2);

        // Act.
        let names = mock::registered_transform_function_names();

        // Assert.
        assert_eq!(names, vec!["transform_function_1", "transform_function_2"]);
    }

    /// Transform function which intentionally creates a new request passing
    /// itself as the target transform function.
    fn transform_function_with_overwrite(arg: TransformArgs) -> HttpResponse {
        create_request_with_transform();
        arg.response
    }

    /// Creates a request with a transform function which overwrites itself.
    fn create_request_with_transform() -> CanisterHttpRequestArgument {
        create_request()
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
        let mock_response = create_response().build();
        mock::mock(request.clone(), mock_response);

        // Act
        let (response,) = mock::http_request(request.clone()).await.unwrap();

        // Assert
        assert_eq!(response.status, 200);
        assert_eq!(mock::times_called(request), 1);
        assert_eq!(
            mock::registered_transform_function_names(),
            vec!["transform_function_with_overwrite"]
        );
    }
}
