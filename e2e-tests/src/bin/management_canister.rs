use ic_cdk::management_canister::*;
use ic_cdk::update;

#[update]
async fn test_basic() {
    // create_canister
    let arg = CreateCanisterArgs {
        settings: Some(CanisterSettings {
            controllers: Some(vec![ic_cdk::id()]),
            // There is no canister in the subnet, so we can set it to 100.
            compute_allocation: Some(1u8.into()),
            // Though the upper limit is 256TiB, the actual subnet may have less memory resource (e.g. local replica).
            // Here we set it to 10KiB for testing.
            memory_allocation: Some(10000u16.into()),
            freezing_threshold: Some(u64::MAX.into()),
            reserved_cycles_limit: Some(u128::MAX.into()),
            log_visibility: Some(LogVisibility::Public),
            wasm_memory_limit: Some((2u64.pow(48) - 1).into()),
        }),
    };
    let canister_id = create_canister(arg, 200_000_000_000_000_000_000_000_000u128)
        .await
        .unwrap()
        .canister_id;

    // canister_status
    let arg = CanisterStatusArgs { canister_id };
    let response = canister_status(arg).await.unwrap();
    assert_eq!(response.status, CanisterStatusType::Running);
    assert_eq!(response.reserved_cycles.0, 0u128.into());
    let definite_canister_setting = response.settings;
    assert_eq!(definite_canister_setting.controllers, vec![ic_cdk::id()]);
    assert_eq!(definite_canister_setting.compute_allocation, 1u8);
    assert_eq!(definite_canister_setting.memory_allocation, 10000u16);
    assert_eq!(definite_canister_setting.freezing_threshold, u64::MAX);
    assert_eq!(definite_canister_setting.reserved_cycles_limit, u128::MAX);
    assert_eq!(
        definite_canister_setting.log_visibility,
        LogVisibility::Public
    );
    assert_eq!(
        definite_canister_setting.wasm_memory_limit,
        2u64.pow(48) - 1
    );

    // update_settings
    let arg = UpdateSettingsArgs {
        canister_id,
        settings: CanisterSettings::default(),
    };
    update_settings(arg).await.unwrap();

    // install_code
    let arg = InstallCodeArgs {
        mode: CanisterInstallMode::Install,
        canister_id,
        // A minimal valid wasm module
        // wat2wasm "(module)"
        wasm_module: b"\x00asm\x01\x00\x00\x00".to_vec(),
        arg: vec![],
    };
    install_code(arg).await.unwrap();

    // uninstall_code
    let arg = UninstallCodeArgs { canister_id };
    uninstall_code(arg).await.unwrap();

    // start_canister
    let arg = StartCanisterArgs { canister_id };
    start_canister(arg).await.unwrap();

    // stop_canister
    let arg = StopCanisterArgs { canister_id };
    stop_canister(arg).await.unwrap();

    // deposit_cycles
    let arg = DepositCyclesArgs { canister_id };
    deposit_cycles(arg, 1_000_000_000_000u128).await.unwrap();

    // delete_canister
    let arg = DeleteCanisterArgs { canister_id };
    delete_canister(arg).await.unwrap();

    // raw_rand
    let bytes = raw_rand().await.unwrap();
    assert_eq!(bytes.len(), 32);
}

#[update]
fn test_canister_info() {
    
}

fn main() {}
