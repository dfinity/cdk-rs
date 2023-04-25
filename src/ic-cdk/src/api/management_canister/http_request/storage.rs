use super::mock::{hash, Mock};
use super::{CanisterHttpRequestArgument, HttpResponse, TransformArgs};
use std::collections::HashMap;
use std::sync::RwLock;

thread_local! {
    static MOCKS: RwLock<HashMap<String, Mock>> = RwLock::new(HashMap::new());

    static TRANSFORM_FUNCTIONS: RwLock<HashMap<String, Box<TransformFn>>> = RwLock::new(HashMap::new());
}

/// Inserts the provided mock into a thread-local hashmap.
pub(crate) fn mock_insert(mock: Mock) {
    MOCKS.with(|cell| {
        cell.write().unwrap().insert(hash(&mock.request), mock);
    });
}

/// Returns a cloned mock from the thread-local hashmap that corresponds to the provided request.
pub(crate) fn mock_get(request: &CanisterHttpRequestArgument) -> Option<Mock> {
    MOCKS.with(|cell| cell.read().unwrap().get(&hash(request)).cloned())
}

type TransformFn = dyn Fn(TransformArgs) -> HttpResponse + 'static;

/// Inserts the provided transform function into a thread-local hashmap.
/// If a transform function with the same name already exists, it is not inserted.
pub(crate) fn transform_function_insert(name: String, func: Box<TransformFn>) {
    TRANSFORM_FUNCTIONS.with(|cell| {
        // This is a workaround to prevent the transform function from being
        // overridden while it is being executed.
        if cell.read().unwrap().get(&name).is_none() {
            cell.write().unwrap().insert(name, func);
        }
    });
}

/// Executes the transform function that corresponds to the provided name.
pub(crate) fn transform_function_call(name: String, arg: TransformArgs) -> Option<HttpResponse> {
    TRANSFORM_FUNCTIONS.with(|cell| cell.read().unwrap().get(&name).map(|f| f(arg)))
}

/// Returns a sorted list of transform function names.
pub(crate) fn transform_function_names() -> Vec<String> {
    TRANSFORM_FUNCTIONS.with(|cell| {
        let mut names: Vec<String> = cell.read().unwrap().keys().cloned().collect();
        names.sort();
        names
    })
}
