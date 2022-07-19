use std::cell::RefCell;

use ic_cdk_macros::update;
use ic_management::*;

thread_local! {
    static CANISTER_ID: RefCell<Option<CanisterId>> = RefCell::default();
}

#[update]
async fn call_create_canister() {
    match CANISTER_ID.with(|id| id.borrow().clone()) {
        Some(canister_id) => {
            ic_cdk::api::print(format!("Canister already created. {}", canister_id));
        }
        None => {
            let canister_id = create_canister(CreateCanisterArgument::default())
                .await
                .unwrap()
                .0
                .canister_id;
            CANISTER_ID.with(|id| *id.borrow_mut() = Some(canister_id));
        }
    }
}

#[update]
async fn call_update_settings() {
    if let Some(canister_id) = CANISTER_ID.with(|id| id.borrow().clone()) {
        let arg = UpdateSettingsArgument {
            canister_id,
            settings: CanisterSettings {
                memory_allocation: Some(10000u32.into()),
                freezing_threshold: Some(20000u32.into()),
                ..Default::default()
            },
        };
        update_settings(arg).await.unwrap();
    } else {
        ic_cdk::api::trap("Canister hasn't been created yet!");
    }
}

#[update]
async fn call_raw_rand() -> Vec<u8> {
    raw_rand().await.unwrap().0
}

fn main() {}
