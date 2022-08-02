use std::cell::RefCell;

use ic_cdk::api::management_canister::*;
use ic_cdk_macros::update;

thread_local! {
    static CANISTER_ID: RefCell<Option<CanisterId>> = RefCell::default();
}

#[update]
async fn call_create_canister() {
    match CANISTER_ID.with(|id| *id.borrow()) {
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
    if let Some(canister_id) = CANISTER_ID.with(|id| *id.borrow()) {
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
async fn call_install_code() {
    if let Some(canister_id) = CANISTER_ID.with(|id| *id.borrow()) {
        let arg = InstallCodeArgument {
            mode: CanisterInstallMode::Install,
            canister_id,
            // A minimal valid wasm module
            // wat2wasm "(module)"
            wasm_module: b"\x00asm\x01\x00\x00\x00".to_vec(),
            arg: vec![],
        };
        install_code(arg).await.unwrap();
    } else {
        ic_cdk::api::trap("Canister hasn't been created yet!");
    }
}

#[update]
async fn call_uninstall_code() {
    if let Some(canister_id) = CANISTER_ID.with(|id| *id.borrow()) {
        let arg = CanisterIdRecord { canister_id };
        uninstall_code(arg).await.unwrap();
    } else {
        ic_cdk::api::trap("Canister hasn't been created yet!");
    }
}

#[update]
async fn call_start_canister() {
    if let Some(canister_id) = CANISTER_ID.with(|id| *id.borrow()) {
        let arg = CanisterIdRecord { canister_id };
        start_canister(arg).await.unwrap();
    } else {
        ic_cdk::api::trap("Canister hasn't been created yet!");
    }
}

#[update]
async fn call_stop_canister() {
    if let Some(canister_id) = CANISTER_ID.with(|id| *id.borrow()) {
        let arg = CanisterIdRecord { canister_id };
        stop_canister(arg).await.unwrap();
    } else {
        ic_cdk::api::trap("Canister hasn't been created yet!");
    }
}

#[update]
async fn call_canister_status() -> CanisterStatusReply {
    if let Some(canister_id) = CANISTER_ID.with(|id| *id.borrow()) {
        let arg = CanisterIdRecord { canister_id };
        canister_status(arg).await.unwrap().0
    } else {
        ic_cdk::api::trap("Canister hasn't been created yet!");
    }
}

#[update]
async fn call_delete_canister() {
    if let Some(canister_id) = CANISTER_ID.with(|id| *id.borrow()) {
        let arg = CanisterIdRecord { canister_id };
        delete_canister(arg).await.unwrap();
        CANISTER_ID.with(|id| *id.borrow_mut() = None);
    } else {
        ic_cdk::api::trap("Canister hasn't been created yet!");
    }
}

#[update]
async fn call_deposit_cycles() {
    if let Some(canister_id) = CANISTER_ID.with(|id| *id.borrow()) {
        let arg = CanisterIdRecord { canister_id };
        deposit_cycles(arg, 1_000_000_000_000u128).await.unwrap();
    } else {
        ic_cdk::api::trap("Canister hasn't been created yet!");
    }
}

#[update]
async fn call_raw_rand() -> Vec<u8> {
    raw_rand().await.unwrap().0
}

mod provisional {
    use super::*;
    use ic_cdk::api::management_canister::provisional::*;

    #[update]
    async fn call_provisional_create_canister_with_cycles() {
        match CANISTER_ID.with(|id| *id.borrow()) {
            Some(canister_id) => {
                ic_cdk::api::print(format!("Canister already created. {}", canister_id));
            }
            None => {
                let canister_id = provisional_create_canister_with_cycles(
                    ProvisionalCreateCanisterWithCyclesArgument::default(),
                )
                .await
                .unwrap()
                .0
                .canister_id;
                CANISTER_ID.with(|id| *id.borrow_mut() = Some(canister_id));
            }
        }
    }

    #[update]
    async fn call_provisional_top_up_canister() {
        if let Some(canister_id) = CANISTER_ID.with(|id| *id.borrow()) {
            let arg = ProvisionalTopUpCanisterArgument {
                canister_id,
                amount: 1_000_000_000u64.into(),
            };
            provisional_top_up_canister(arg).await.unwrap();
        } else {
            ic_cdk::api::trap("Canister hasn't been created yet!");
        }
    }
}

mod threshold_ecdsa {
    use super::*;
    use ic_cdk::api::management_canister::threshold_ecdsa::*;

    #[update]
    async fn call_ecdsa_public_key() -> EcdsaPublicKeyReply {
        let key_id = EcdsaKeyId {
            curve: EcdsaCurve::Secp256k1,
            name: "dfx_test_key".to_string(),
        };
        let caller = ic_cdk::caller().as_slice().to_vec();
        let arg = EcdsaPublicKeyArgument {
            canister_id: None,
            derivation_path: vec![caller],
            key_id,
        };
        ecdsa_public_key(arg).await.unwrap().0
    }
}

fn main() {}
