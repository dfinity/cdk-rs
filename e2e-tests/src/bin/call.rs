use candid::Encode;
use ic_cdk::api::canister_self;
use ic_cdk::call::Call;
use ic_cdk::update;

/// A simple endpoint that takes empty arguments.
#[update]
async fn foo() -> u32 {
    0
}

/// `Call::new(...)` can be configured and called.
#[update]
async fn call_foo() {
    let n = 0u32;
    let bytes = Encode!(&n).unwrap();

    let res: u32 = Call::new(canister_self(), "foo").call().await.unwrap();
    assert_eq!(res, n);
    let res: (u32,) = Call::new(canister_self(), "foo")
        .call_tuple()
        .await
        .unwrap();
    assert_eq!(res.0, n);
    let res = Call::new(canister_self(), "foo").call_raw().await.unwrap();
    assert_eq!(res, bytes);
    Call::new(canister_self(), "foo").call_oneway().unwrap();

    let res: (u32,) = Call::new(canister_self(), "foo")
        .with_guaranteed_response()
        .call_tuple()
        .await
        .unwrap();
    assert_eq!(res.0, n);
    let res: (u32,) = Call::new(canister_self(), "foo")
        .change_timeout(5)
        .call_tuple()
        .await
        .unwrap();
    assert_eq!(res.0, n);
    let res: (u32,) = Call::new(canister_self(), "foo")
        .with_cycles(1000)
        .call_tuple()
        .await
        .unwrap();
    assert_eq!(res.0, n);
}

/// A simple endpoint that takes a single `u32` argument and returns it.
#[update]
async fn echo(arg: u32) -> u32 {
    arg
}

/// `Call::new(...).with_arg(...)` can be configured and called.
#[update]
async fn call_echo_with_arg() {
    let n = 1u32;
    let bytes = Encode!(&n).unwrap();
    // call*
    let res: u32 = Call::new(canister_self(), "echo")
        .with_arg(n)
        .call()
        .await
        .unwrap();
    assert_eq!(res, n);
    let res: (u32,) = Call::new(canister_self(), "echo")
        .with_arg(n)
        .call_tuple()
        .await
        .unwrap();
    assert_eq!(res.0, n);
    let res = Call::new(canister_self(), "echo")
        .with_arg(n)
        .call_raw()
        .await
        .unwrap();
    assert_eq!(res, bytes);
    Call::new(canister_self(), "echo")
        .with_arg(n)
        .call_oneway()
        .unwrap();
    // with*
    let res: (u32,) = Call::new(canister_self(), "echo")
        .with_arg(n)
        .with_guaranteed_response()
        .call_tuple()
        .await
        .unwrap();
    assert_eq!(res.0, n);
    let res: (u32,) = Call::new(canister_self(), "echo")
        .with_arg(n)
        .change_timeout(5)
        .call_tuple()
        .await
        .unwrap();
    assert_eq!(res.0, n);
    let res: (u32,) = Call::new(canister_self(), "echo")
        .with_arg(n)
        .with_cycles(1000)
        .call_tuple()
        .await
        .unwrap();
    assert_eq!(res.0, n);
}

/// `Call::new(...).with_args(...)` can be configured and called.
#[update]
async fn call_echo_with_args() {
    let n = 1u32;
    let bytes = Encode!(&n).unwrap();
    // call*
    let res: u32 = Call::new(canister_self(), "echo")
        .with_args(&(n,))
        .call()
        .await
        .unwrap();
    assert_eq!(res, n);
    let res: (u32,) = Call::new(canister_self(), "echo")
        .with_args(&(n,))
        .call_tuple()
        .await
        .unwrap();
    assert_eq!(res.0, n);
    let res = Call::new(canister_self(), "echo")
        .with_args(&(n,))
        .call_raw()
        .await
        .unwrap();
    assert_eq!(res, bytes);
    Call::new(canister_self(), "echo")
        .with_args(&(n,))
        .call_oneway()
        .unwrap();
    // with*
    let res: (u32,) = Call::new(canister_self(), "echo")
        .with_args(&(n,))
        .with_guaranteed_response()
        .call_tuple()
        .await
        .unwrap();
    assert_eq!(res.0, n);
    let res: (u32,) = Call::new(canister_self(), "echo")
        .with_args(&(n,))
        .change_timeout(5)
        .call_tuple()
        .await
        .unwrap();
    assert_eq!(res.0, n);
    let res: (u32,) = Call::new(canister_self(), "echo")
        .with_args(&(n,))
        .with_cycles(1000)
        .call_tuple()
        .await
        .unwrap();
    assert_eq!(res.0, n);
}

/// Call::new(...).with_raw_args(...) can be configured and called.
#[update]
async fn call_echo_with_raw_args() {
    let n = 1u32;
    let bytes: Vec<u8> = Encode!(&n).unwrap();
    // call*
    let res: u32 = Call::new(canister_self(), "echo")
        .with_raw_args(&bytes)
        .call()
        .await
        .unwrap();
    assert_eq!(res, n);
    let res: (u32,) = Call::new(canister_self(), "echo")
        .with_raw_args(&bytes)
        .call_tuple()
        .await
        .unwrap();
    assert_eq!(res.0, n);
    let res = Call::new(canister_self(), "echo")
        .with_raw_args(&bytes)
        .call_raw()
        .await
        .unwrap();
    assert_eq!(res, bytes);
    Call::new(canister_self(), "echo")
        .with_raw_args(&bytes)
        .call_oneway()
        .unwrap();
    // with*
    let res: (u32,) = Call::new(canister_self(), "echo")
        .with_raw_args(&bytes)
        .with_guaranteed_response()
        .call_tuple()
        .await
        .unwrap();
    assert_eq!(res.0, n);
    let res: (u32,) = Call::new(canister_self(), "echo")
        .with_raw_args(&bytes)
        .change_timeout(5)
        .call_tuple()
        .await
        .unwrap();
    assert_eq!(res.0, n);
    let res: (u32,) = Call::new(canister_self(), "echo")
        .with_raw_args(&bytes)
        .with_cycles(1000)
        .call_tuple()
        .await
        .unwrap();
    assert_eq!(res.0, n);
}

/// Retries the call until it succeeds.
///
/// Returns the number of retries.
async fn retry(call_to_retry: Call<'_, '_>) -> u32 {
    let mut retry = 0;
    loop {
        match call_to_retry.call_raw().await {
            Ok(_) => break,
            Err(_) => {
                retry += 1;
                continue;
            }
        }
    }
    retry
}

#[update]
async fn retry_calls() {
    let n: u32 = 1u32;
    let call = Call::new(canister_self(), "foo");
    assert_eq!(retry(call).await, 0);
    let call_with_arg = Call::new(canister_self(), "echo").with_arg(n);
    assert_eq!(retry(call_with_arg).await, 0);
    let args = (n,);
    let call_with_args = Call::new(canister_self(), "echo").with_args(&args);
    assert_eq!(retry(call_with_args).await, 0);
    let raw_args = Encode!(&n).unwrap();
    let call_with_raw_args = Call::new(canister_self(), "echo").with_raw_args(&raw_args);
    assert_eq!(retry(call_with_raw_args).await, 0);
}
fn main() {}
