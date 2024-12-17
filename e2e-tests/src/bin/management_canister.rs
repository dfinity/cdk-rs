use ic_cdk::api::canister_self;
use ic_cdk::management_canister::*;
use ic_cdk::update;

#[update]
async fn basic() {
    // create_canister
    let self_id = canister_self();
    let arg = CreateCanisterArgs {
        settings: Some(CanisterSettings {
            controllers: Some(vec![self_id]),
            compute_allocation: Some(0u8.into()),
            memory_allocation: Some(0u8.into()),
            freezing_threshold: Some(0u8.into()),
            reserved_cycles_limit: Some(0u8.into()),
            log_visibility: Some(LogVisibility::Public),
            wasm_memory_limit: Some(0u8.into()),
        }),
    };
    // 500 B is the minimum cycles required to create a canister.
    // Here we set 1 T cycles for other operations below.
    let canister_id = create_canister(arg, 1_000_000_000_000u128)
        .await
        .unwrap()
        .canister_id;

    // canister_status
    let arg = CanisterStatusArgs { canister_id };
    let result = canister_status(arg).await.unwrap();
    assert_eq!(result.status, CanisterStatusType::Running);
    assert_eq!(result.reserved_cycles.0, 0u128.into());
    let definite_canister_setting = result.settings;
    assert_eq!(definite_canister_setting.controllers, vec![self_id]);
    assert_eq!(definite_canister_setting.compute_allocation, 0u8);
    assert_eq!(definite_canister_setting.memory_allocation, 0u8);
    assert_eq!(definite_canister_setting.freezing_threshold, 0u8);
    assert_eq!(definite_canister_setting.reserved_cycles_limit, 0u8);
    assert_eq!(
        definite_canister_setting.log_visibility,
        LogVisibility::Public
    );
    assert_eq!(definite_canister_setting.wasm_memory_limit, 0u8);

    // update_settings
    let arg = UpdateSettingsArgs {
        canister_id,
        settings: CanisterSettings {
            freezing_threshold: Some(0u16.into()),
            log_visibility: Some(LogVisibility::AllowedViewers(vec![self_id])),
            ..Default::default()
        },
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
async fn provisional() {
    // provisional_create_canister_with_cycles
    let settings = CanisterSettings {
        log_visibility: Some(LogVisibility::Controllers),
        ..Default::default()
    };
    let arg = ProvisionalCreateCanisterWithCyclesArgs {
        amount: Some(10_000_000_000_000u64.into()),
        settings: Some(settings),
    };
    let canister_id = provisional_create_canister_with_cycles(arg)
        .await
        .unwrap()
        .canister_id;

    // provisional_top_up_canister
    let arg = ProvisionalTopUpCanisterArgs {
        canister_id,
        amount: 1_000_000_000u64.into(),
    };
    provisional_top_up_canister(arg).await.unwrap();
}

#[update]
async fn snapshots() {
    let arg = CreateCanisterArgs::default();
    let canister_id = create_canister(arg, 2_000_000_000_000u128)
        .await
        .unwrap()
        .canister_id;

    // Cannot take a snapshot of a canister that is empty.
    // So we install a minimal wasm module.
    let arg = InstallCodeArgs {
        mode: CanisterInstallMode::Install,
        canister_id,
        // A minimal valid wasm module
        // wat2wasm "(module)"
        wasm_module: b"\x00asm\x01\x00\x00\x00".to_vec(),
        arg: vec![],
    };
    install_code(arg).await.unwrap();

    // take_canister_snapshot
    let arg = TakeCanisterSnapshotArgs {
        canister_id,
        replace_snapshot: None,
    };
    let snapshot = take_canister_snapshot(arg).await.unwrap();

    // load_canister_snapshot
    let arg = LoadCanisterSnapshotArgs {
        canister_id,
        snapshot_id: snapshot.id.clone(),
    };
    assert!(load_canister_snapshot(arg).await.is_ok());

    // list_canister_snapshots
    let args = ListCanisterSnapshotsArgs { canister_id };
    let snapshots = list_canister_snapshots(args).await.unwrap();
    assert_eq!(snapshots.len(), 1);
    assert_eq!(snapshots[0].id, snapshot.id);

    // delete_canister_snapshot
    let arg = DeleteCanisterSnapshotArgs {
        canister_id,
        snapshot_id: snapshot.id.clone(),
    };
    assert!(delete_canister_snapshot(arg).await.is_ok());

    // check the above snapshot operations are recorded in the canister's history.
    let arg = CanisterInfoArgs {
        canister_id,
        num_requested_changes: Some(1),
    };
    let canister_info_result = canister_info(arg).await.unwrap();
    assert_eq!(canister_info_result.total_num_changes, 3);
    assert_eq!(canister_info_result.recent_changes.len(), 1);
    if let CanisterChange {
        details: CanisterChangeDetails::LoadSnapshot(load_snapshot_record),
        ..
    } = &canister_info_result.recent_changes[0]
    {
        assert_eq!(load_snapshot_record.snapshot_id, snapshot.id);
    } else {
        panic!("Expected the most recent change to be LoadSnapshot");
    }
}
fn main() {}
