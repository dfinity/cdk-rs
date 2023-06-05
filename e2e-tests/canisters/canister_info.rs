use ic_cdk::api::management_canister::main::{
    create_canister, install_code, uninstall_code, update_settings, CanisterIdRecord,
    CanisterInfoRequest, CanisterInfoResponse, CanisterInstallMode::*, CanisterSettings,
    CreateCanisterArgument, InstallCodeArgument, UpdateSettingsArgument,
};
use ic_cdk::export::Principal;

#[ic_cdk::update]
async fn info(canister_id: Principal) -> CanisterInfoResponse {
    let request = CanisterInfoRequest {
        canister_id,
        num_requested_changes: Some(20),
    };
    ic_cdk::api::call::call::<(CanisterInfoRequest,), (CanisterInfoResponse,)>(
        Principal::management_canister(),
        "canister_info",
        (request,),
    )
    .await
    .unwrap()
    .0
}

#[ic_cdk_macros::update]
async fn canister_lifecycle() -> Principal {
    let canister_id = create_canister(CreateCanisterArgument { settings: None })
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
        mode: Upgrade,
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
        },
        canister_id: canister_id.canister_id,
    })
    .await
    .unwrap();
    canister_id.canister_id
}

fn main() {}
