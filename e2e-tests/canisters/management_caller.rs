use ic_cdk::*;

mod main {
    use super::*;
    use ic_cdk::api::management_canister::main::*;
    #[update]
    async fn execute_main_methods() {
        let arg = CreateCanisterArgument {
            settings: Some(CanisterSettings {
                controllers: Some(vec![ic_cdk::id()]),
                compute_allocation: Some(0u8.into()),
                memory_allocation: Some(10000u16.into()),
                freezing_threshold: Some(10000u16.into()),
                reserved_cycles_limit: Some(10000u16.into()),
            }),
        };
        let canister_id = create_canister(arg, 100_000_000_000u128 / 13)
            .await
            .unwrap()
            .0
            .canister_id;

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
        let arg = CanisterIdRecord { canister_id };
        uninstall_code(arg).await.unwrap();
        start_canister(arg).await.unwrap();
        stop_canister(arg).await.unwrap();
        let response = canister_status(arg).await.unwrap().0;
        assert_eq!(response.status, CanisterStatusType::Stopped);
        assert_eq!(response.reserved_cycles.0, 0u128.into());
        deposit_cycles(arg, 1_000_000_000_000u128).await.unwrap();
        delete_canister(arg).await.unwrap();
        let response = raw_rand().await.unwrap().0;
        assert_eq!(response.len(), 32);
    }
}

mod provisional {
    use super::*;
    use ic_cdk::api::management_canister::provisional::*;

    #[update]
    async fn execute_provisional_methods() {
        let settings = CanisterSettings {
            controllers: Some(vec![ic_cdk::caller()]),
            compute_allocation: Some(50u8.into()),
            memory_allocation: Some(10000u16.into()),
            freezing_threshold: Some(10000u16.into()),
            reserved_cycles_limit: Some(10000u16.into()),
        };
        let arg = ProvisionalCreateCanisterWithCyclesArgument {
            amount: Some(1_000_000_000u64.into()),
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

fn main() {}
