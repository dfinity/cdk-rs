use candid::Principal;
use ic_cdk::call::Call;
use ic_cdk::{query, update};
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
#[allow(clippy::await_holding_lock)]
async fn panic_after_async() {
    let mut lock = RESOURCE
        .write()
        .unwrap_or_else(|_| ic_cdk::api::trap("failed to obtain a write lock"));
    *lock += 1;
    let value = *lock;
    // Do not drop the lock before the await point.

    Call::bounded_wait(ic_cdk::api::canister_self(), "inc")
        .with_arg(value)
        .await
        .unwrap();
    ic_cdk::api::trap("Goodbye, cruel world.")
}

#[update]
#[allow(clippy::await_holding_lock)]
async fn panic_twice() {
    let _lock = RESOURCE.write().unwrap();
    let fut1 = async_then_panic();
    let fut2 = async_then_panic();
    futures::future::join_all([fut1, fut2]).await;
}

async fn async_then_panic() {
    Call::bounded_wait(ic_cdk::api::canister_self(), "on_notify")
        .await
        .unwrap();
    panic!();
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
    Call::bounded_wait(whom, method.as_str())
        .oneway()
        .unwrap_or_else(|reject| {
            ic_cdk::api::trap(format!(
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
    Call::bounded_wait(greeter, "greet")
        .with_arg("myself")
        .await
        .unwrap()
        .candid()
        .unwrap()
}

#[update]
async fn invalid_reply_payload_does_not_trap() -> String {
    // We're decoding an integer instead of a string, decoding must fail.
    let result = Call::bounded_wait(ic_cdk::api::canister_self(), "greet")
        .with_arg("World")
        .await
        .unwrap()
        .candid::<u64>();

    match result {
        Ok(_) => ic_cdk::api::trap("expected the decoding to fail"),
        Err(e) => format!("handled decoding error gracefully: {e}"),
    }
}

#[update]
async fn await_channel_completion() -> String {
    let (tx, rx) = async_channel::bounded(1);
    ic_cdk::futures::spawn(async move {
        let greeting: String = Call::bounded_wait(ic_cdk::api::canister_self(), "greet")
            .with_arg("myself")
            .await
            .unwrap()
            .candid()
            .unwrap();
        tx.send(greeting).await.unwrap();
    });
    let greeting = rx.recv().await;
    greeting.unwrap()
}

#[update]
async fn schedule_on_panic() {
    struct Guard;
    impl Drop for Guard {
        fn drop(&mut self) {
            for _ in 0..3 {
                ic_cdk::futures::spawn(async {
                    on_notify();
                })
            }
        }
    }
    let _guard = Guard;
    Call::bounded_wait(ic_cdk::api::canister_self(), "on_notify")
        .await
        .unwrap();
    ic_cdk::trap("testing");
}

fn main() {}
