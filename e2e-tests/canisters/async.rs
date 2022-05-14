use candid::Principal;
use ic_cdk_macros::{query, update};
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
    let mut lock = RESOURCE
        .write()
        .unwrap_or_else(|_| ic_cdk::api::trap("failed to obtain a write lock"));
    *lock += 1;
    let _: (u64,) = ic_cdk::call(ic_cdk::api::id(), "inc", (*lock,))
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

fn main() {}
