use ic_cdk::*;

/// Some management canister "main" methods are tested with other e2e canisters:
/// - canister_info.rs
/// - chunk.rs
mod main {
    use super::*;
    use candid::Principal;
    use ic_cdk::api::management_canister::main::*;
    #[update]
    async fn execute_main_methods() {
        let arg = CreateCanisterArgument {
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
            .0
            .canister_id;

        let canister_id_record = CanisterIdRecord { canister_id };
        let response = canister_status(canister_id_record).await.unwrap().0;
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

        let arg = UpdateSettingsArgument {
            canister_id,
            settings: CanisterSettings::default(),
        };
        update_settings(arg).await.unwrap();

        let arg = InstallCodeArgument {
            mode: CanisterInstallMode::Install,
            canister_id,
            // A minimal valid wasm module
            // wat2wasm "(module)"
            wasm_module: b"\x00asm\x01\x00\x00\x00".to_vec(),
            arg: vec![],
        };
        install_code(arg).await.unwrap();

        uninstall_code(canister_id_record).await.unwrap();
        start_canister(canister_id_record).await.unwrap();
        stop_canister(canister_id_record).await.unwrap();
        deposit_cycles(canister_id_record, 1_000_000_000_000u128)
            .await
            .unwrap();
        delete_canister(canister_id_record).await.unwrap();
        let response = raw_rand().await.unwrap().0;
        assert_eq!(response.len(), 32);
    }

    #[update]
    async fn execute_subnet_info(subnet_id: Principal) {
        let arg = SubnetInfoArgs { subnet_id };
        let response = subnet_info(arg).await.unwrap().0;
        assert!(!response.replica_version.is_empty());
    }
}

mod provisional {
    use super::*;
    use api::management_canister::main::LogVisibility;
    use ic_cdk::api::management_canister::provisional::*;

    #[update]
    async fn execute_provisional_methods() {
        let settings = CanisterSettings {
            controllers: Some(vec![ic_cdk::caller()]),
            compute_allocation: Some(50u8.into()),
            memory_allocation: Some(10000u16.into()),
            freezing_threshold: Some(10000u16.into()),
            reserved_cycles_limit: Some(10000u16.into()),
            log_visibility: Some(LogVisibility::Public),
            wasm_memory_limit: Some(10000u16.into()),
        };
        let arg = ProvisionalCreateCanisterWithCyclesArgument {
            amount: Some(10_000_000_000_000u64.into()),
            settings: Some(settings),
        };
        let canister_id = provisional_create_canister_with_cycles(arg)
            .await
            .unwrap()
            .0
            .canister_id;

        let arg = ProvisionalTopUpCanisterArgument {
            canister_id,
            amount: 1_000_000_000u64.into(),
        };
        provisional_top_up_canister(arg).await.unwrap();
    }
}

mod snapshot {
    use super::*;
    use ic_cdk::api::management_canister::main::*;

    #[update]
    async fn execute_snapshot_methods() {
        let arg = CreateCanisterArgument::default();
        let canister_id = create_canister(arg, 2_000_000_000_000u128)
            .await
            .unwrap()
            .0
            .canister_id;

        // Cannot take a snapshot of a canister that is empty.
        // So we install a minimal wasm module.
        let arg = InstallCodeArgument {
            mode: CanisterInstallMode::Install,
            canister_id,
            // A minimal valid wasm module
            // wat2wasm "(module)"
            wasm_module: b"\x00asm\x01\x00\x00\x00".to_vec(),
            arg: vec![],
        };
        install_code(arg).await.unwrap();

        let arg = TakeCanisterSnapshotArgs {
            canister_id,
            replace_snapshot: None,
        };
        let snapshot = take_canister_snapshot(arg).await.unwrap().0;

        let arg = LoadCanisterSnapshotArgs {
            canister_id,
            snapshot_id: snapshot.id.clone(),
            sender_canister_version: None,
        };
        assert!(load_canister_snapshot(arg).await.is_ok());

        let canister_id_record = CanisterIdRecord { canister_id };
        let snapshots = list_canister_snapshots(canister_id_record).await.unwrap().0;
        assert_eq!(snapshots.len(), 1);
        assert_eq!(snapshots[0].id, snapshot.id);

        let arg = DeleteCanisterSnapshotArgs {
            canister_id,
            snapshot_id: snapshot.id.clone(),
        };
        assert!(delete_canister_snapshot(arg).await.is_ok());

        let arg = CanisterInfoRequest {
            canister_id,
            num_requested_changes: Some(1),
        };
        let canister_info_response = canister_info(arg).await.unwrap().0;
        assert_eq!(canister_info_response.total_num_changes, 3);
        assert_eq!(canister_info_response.recent_changes.len(), 1);
        if let CanisterChange {
            details: CanisterChangeDetails::LoadSnapshot(load_snapshot_record),
            ..
        } = &canister_info_response.recent_changes[0]
        {
            assert_eq!(load_snapshot_record.snapshot_id, snapshot.id);
        } else {
            panic!("Expected the most recent change to be LoadSnapshot");
        }
    }
}

fn main() {}
