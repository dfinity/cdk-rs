use ic_cdk_macros::update;
use ic_management::raw_rand;

#[update]
async fn call_raw_rand() -> Vec<u8> {
    raw_rand().await.unwrap().0
}

fn main() {}