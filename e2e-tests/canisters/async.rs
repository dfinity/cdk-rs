use ic_cdk::{export::Principal, query, update};
use lazy_static::lazy_static;
use std::sync::RwLock;

lazy_static! {
    static ref RESOURCE: RwLock<u64> = RwLock::new(0);
    static ref NOTIFICATIONS_RECEIVED: RwLock<u64> = RwLock::new(0);
}

#[query]
fn inc(n: u64) -> u64 {
    n + 1
}

#[query]
fn invocation_count() -> u64 {
    let lock = RESOURCE
        .read()
        .unwrap_or_else(|_| ic_cdk::api::trap("failed to obtain a read lock"));
    *lock
}

#[update]
async fn panic_after_async() {
    let value = {
        let mut lock = RESOURCE
            .write()
            .unwrap_or_else(|_| ic_cdk::api::trap("failed to obtain a write lock"));
        *lock += 1;
        *lock
        // Drop the lock before the await point.
    };
    let _: (u64,) = ic_cdk::call(ic_cdk::api::id(), "inc", (value,))
        .await
        .expect("failed to call self");
    ic_cdk::api::trap("Goodbye, cruel world.")
}

#[query]
fn notifications_received() -> u64 {
    *NOTIFICATIONS_RECEIVED.read().unwrap()
}

#[update]
fn on_notify() {
    *NOTIFICATIONS_RECEIVED.write().unwrap() += 1;
}

#[update]
fn notify(whom: Principal, method: String) {
    ic_cdk::notify(whom, method.as_str(), ()).unwrap_or_else(|reject| {
        ic_cdk::api::trap(&format!(
            "failed to notify (callee={}, method={}): {:?}",
            whom, method, reject
        ))
    });
}

#[query]
fn greet(name: String) -> String {
    format!("Hello, {}", name)
}

#[query(composite = true)]
async fn greet_self(greeter: Principal) -> String {
    let (greeting,) = ic_cdk::api::call::call(greeter, "greet", ("myself",))
        .await
        .unwrap();
    greeting
}

#[update]
async fn invalid_reply_payload_does_not_trap() -> String {
    // We're decoding an integer instead of a string, decoding must fail.
    let result: Result<(u64,), _> =
        ic_cdk::call(ic_cdk::api::id(), "greet", ("World".to_string(),)).await;

    match result {
        Ok((_n,)) => ic_cdk::api::trap("expected the decoding to fail"),
        Err((err_code, _)) => format!(
            "handled decoding error gracefully with code {}",
            err_code as i32
        ),
    }
}

fn main() {}
