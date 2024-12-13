use candid::Encode;
use ic_cdk::{call::DecoderConfig, prelude::*};

/// This endpoint is to be called by the following Call struct invocation.
#[update]
async fn echo(arg: u32) -> u32 {
    arg
}

#[update]
async fn call_echo() {
    let num = 1u32;
    let bytes: Vec<u8> = Encode!(&num).unwrap();

    // 1. Various ways to call*
    // 1.1 call()
    let res: (u32,) = Call::new(id(), "echo")
        .with_args((num,))
        .call()
        .await
        .unwrap();
    assert_eq!(res.0, num);
    // 1.2 call_raw()
    let res = Call::new(id(), "echo")
        .with_args((num,))
        .call_raw()
        .await
        .unwrap();
    assert_eq!(res, bytes);
    // 1.3 call_with_decoder_config()
    let config = DecoderConfig::default();
    let res: (u32,) = Call::new(id(), "echo")
        .with_args((num,))
        .call_with_decoder_config(&config)
        .await
        .unwrap();
    assert_eq!(res.0, num);
    // 1.4 call_raw_with_decoder_config()
    Call::new(id(), "echo")
        .with_args((num,))
        .call_and_forget()
        .unwrap();

    // 2. Various ways to config the call
    // 2.1 with_raw_args()
    let res: (u32,) = Call::new(id(), "echo")
        .with_raw_args(&bytes)
        .call()
        .await
        .unwrap();
    assert_eq!(res.0, num);
    // 2.2 with_guaranteed_response()
    let res: (u32,) = Call::new(id(), "echo")
        .with_args((num,))
        .with_guaranteed_response()
        .call()
        .await
        .unwrap();
    assert_eq!(res.0, num);
    // 2.3 change_timeout()
    let res: (u32,) = Call::new(id(), "echo")
        .with_args((num,))
        .change_timeout(5)
        .call()
        .await
        .unwrap();
    assert_eq!(res.0, num);
    // 2.4 with_cycles()
    let res: (u32,) = Call::new(id(), "echo")
        .with_args((num,))
        .with_cycles(100_000)
        .call()
        .await
        .unwrap();
    assert_eq!(res.0, num);
}

#[update]
async fn foo() -> Vec<u8> {
    vec![1, 2, 3]
}

#[update]
async fn call_foo() {
    let res: (Vec<u8>,) = Call::new(id(), "foo").call().await.unwrap();
    assert_eq!(res.0, vec![1, 2, 3]);
}

fn main() {}
