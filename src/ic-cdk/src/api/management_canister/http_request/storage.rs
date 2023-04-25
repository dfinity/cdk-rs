use super::mock::{hash, Mock};
use super::{CanisterHttpRequestArgument, HttpResponse, TransformArgs};
use std::collections::HashMap;
use std::sync::RwLock;

thread_local! {
    static MOCKS: RwLock<HashMap<String, Mock>> = RwLock::new(HashMap::new());

    static TRANSFORM_FUNCTIONS: RwLock<HashMap<String, Box<TransformFn>>> = RwLock::new(HashMap::new());
}

pub(crate) fn mock_insert(mock: Mock) {
    MOCKS.with(|cell| {
        cell.write().unwrap().insert(hash(&mock.request), mock);
    });
}

pub(crate) fn mock_get(request: &CanisterHttpRequestArgument) -> Option<Mock> {
    MOCKS.with(|cell| cell.read().unwrap().get(&hash(request)).cloned())
}

pub type TransformFn = dyn Fn(TransformArgs) -> HttpResponse + 'static;

pub(crate) fn transform_function_insert(name: String, func: Box<TransformFn>) {
    TRANSFORM_FUNCTIONS.with(|cell| {
        // This is a workaround to prevent the transform function from being
        // overridden while it is being executed.
        if cell.read().unwrap().get(&name).is_none() {
            cell.write().unwrap().insert(name, func);
        }
    });
}

pub(crate) fn transform_function_call(name: String, arg: TransformArgs) -> Option<HttpResponse> {
    TRANSFORM_FUNCTIONS.with(|cell| cell.read().unwrap().get(&name).map(|f| f(arg)))
}

pub(crate) fn transform_function_names() -> Vec<String> {
    TRANSFORM_FUNCTIONS.with(|cell| {
        let mut names: Vec<String> = cell.read().unwrap().keys().cloned().collect();
        names.sort();
        names
    })
}
