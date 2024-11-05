use candid::Principal;
use ic_cdk::api::management_canister::main::{
    canister_info, create_canister, install_code, uninstall_code, update_settings,
    CanisterIdRecord, CanisterInfoRequest, CanisterInfoResponse,
    CanisterInstallMode::{Install, Reinstall, Upgrade},
    CanisterSettings, CreateCanisterArgument, InstallCodeArgument, UpdateSettingsArgument,
};

#[ic_cdk::update]
async fn info(canister_id: Principal) -> CanisterInfoResponse {
    let request = CanisterInfoRequest {
        canister_id,
        num_requested_changes: Some(20),
    };
    canister_info(request).await.unwrap().0
}

#[ic_cdk::update]
async fn canister_lifecycle() -> Principal {
    let canister_id = create_canister(CreateCanisterArgument { settings: None }, 1_000_000_000_000)
        .await
        .unwrap()
        .0;
    install_code(InstallCodeArgument {
        mode: Install,
        arg: vec![],
        wasm_module: vec![0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00],
        canister_id: canister_id.canister_id,
    })
    .await
    .unwrap();
    uninstall_code(CanisterIdRecord {
        canister_id: canister_id.canister_id,
    })
    .await
    .unwrap();
    install_code(InstallCodeArgument {
        mode: Install,
        arg: vec![],
        wasm_module: vec![0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00],
        canister_id: canister_id.canister_id,
    })
    .await
    .unwrap();
    install_code(InstallCodeArgument {
        mode: Reinstall,
        arg: vec![],
        wasm_module: vec![0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00],
        canister_id: canister_id.canister_id,
    })
    .await
    .unwrap();
    install_code(InstallCodeArgument {
        mode: Upgrade(None),
        arg: vec![],
        wasm_module: vec![0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00],
        canister_id: canister_id.canister_id,
    })
    .await
    .unwrap();
    update_settings(UpdateSettingsArgument {
        settings: CanisterSettings {
            controllers: Some(vec![
                ic_cdk::id(),
                canister_id.canister_id,
                Principal::anonymous(),
            ]),
            compute_allocation: None,
            memory_allocation: None,
            freezing_threshold: None,
            reserved_cycles_limit: None,
            log_visibility: None,
            wasm_memory_limit: None,
        },
        canister_id: canister_id.canister_id,
    })
    .await
    .unwrap();
    canister_id.canister_id
}

fn main() {}
