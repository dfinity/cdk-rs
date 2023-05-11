use crate::mock::{hash, Mock};
use ic_cdk::api::management_canister::http_request::{
    CanisterHttpRequestArgument, HttpResponse, TransformArgs,
};
use std::cell::RefCell;
use std::collections::HashMap;

thread_local! {
    static MOCKS: RefCell<HashMap<String, Mock>> = RefCell::new(HashMap::new());

    static TRANSFORM_FUNCTIONS: RefCell<HashMap<String, Box<TransformFn>>> = RefCell::new(HashMap::new());
}

/// Inserts the provided mock into a thread-local hashmap.
pub(crate) fn mock_insert(mock: Mock) {
    MOCKS.with(|cell| {
        cell.borrow_mut().insert(hash(&mock.request), mock);
    });
}

/// Returns a cloned mock from the thread-local hashmap that corresponds to the provided request.
pub(crate) fn mock_get(request: &CanisterHttpRequestArgument) -> Option<Mock> {
    MOCKS.with(|cell| cell.borrow().get(&hash(request)).cloned())
}

type TransformFn = dyn Fn(TransformArgs) -> HttpResponse + 'static;

/// Inserts the provided transform function into a thread-local hashmap.
/// If a transform function with the same name already exists, it is not inserted.
pub(crate) fn transform_function_insert(name: String, func: Box<TransformFn>) {
    TRANSFORM_FUNCTIONS.with(|cell| {
        // This is a workaround to prevent the transform function from being
        // overridden while it is being executed.
        if cell.borrow().get(&name).is_none() {
            cell.borrow_mut().insert(name, func);
        }
    });
}

/// Executes the transform function that corresponds to the provided name.
pub(crate) fn transform_function_call(name: String, arg: TransformArgs) -> Option<HttpResponse> {
    TRANSFORM_FUNCTIONS.with(|cell| cell.borrow().get(&name).map(|f| f(arg)))
}
