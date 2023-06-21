use ic_cdk::*;

mod main {
    use super::*;
    use ic_cdk::api::management_canister::main::*;
    #[update]
    async fn execute_main_methods() {
        let arg = CreateCanisterArgument {
            settings: Some(CanisterSettings {
                controllers: Some(vec![ic_cdk::id()]),
                compute_allocation: Some(0.into()),
                memory_allocation: Some(10000.into()),
                freezing_threshold: Some(10000.into()),
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
            compute_allocation: Some(50.into()),
            memory_allocation: Some(10000.into()),
            freezing_threshold: Some(10000.into()),
        };
        let arg = ProvisionalCreateCanisterWithCyclesArgument {
            amount: Some(1_000_000_000.into()),
            settings: Some(settings),
        };
        let canister_id = provisional_create_canister_with_cycles(arg)
            .await
            .unwrap()
            .0
            .canister_id;

        let arg = ProvisionalTopUpCanisterArgument {
            canister_id,
            amount: 1_000_000_000.into(),
        };
        provisional_top_up_canister(arg).await.unwrap();
    }
}

mod http_request {
    use super::*;
    use ic_cdk::api::management_canister::http_request::*;

    fn http_request_required_cycles(arg: &CanisterHttpRequestArgument) -> u128 {
        let max_response_bytes = match arg.max_response_bytes {
            Some(ref n) => *n as u128,
            None => 2 * 1024 * 1024u128, // default 2MiB
        };
        let arg_raw = ic_cdk::export::candid::utils::encode_args((arg,))
            .expect("Failed to encode arguments.");
        400_000_000u128 / 13
            + 100_000u128 / 13
                * (arg_raw.len() as u128 + "http_request".len() as u128 + max_response_bytes)
    }

    #[update]
    async fn http_request_example() {
        let url = "https://example.com".to_string();
        let arg = CanisterHttpRequestArgument {
            url,
            max_response_bytes: Some(3000),
            method: HttpMethod::GET,
            headers: vec![],
            body: None,
            transform: None,
        };
        let header = HttpHeader {
            name: "custom-header".to_string(),
            value: "test".to_string(),
        };
        let cycles = http_request_required_cycles(&arg);
        let response = http_request_with_closure(arg.clone(), cycles, {
            let header = header.clone();
            move |mut response| {
                response.headers = vec![header];
                response
            }
        })
        .await
        .unwrap()
        .0;
        assert_eq!(response.status, 200);
        assert_eq!(response.headers.get(0), Some(&header));
    }
}

mod ecdsa {
    use super::*;
    use ic_cdk::api::management_canister::ecdsa::*;
    use sha2::Digest;

    #[update]
    async fn execute_ecdsa_methods() {
        let key_id = EcdsaKeyId {
            curve: EcdsaCurve::Secp256k1,
            name: "dfx_test_key".to_string(),
        };
        let derivation_path = vec![];
        let arg = EcdsaPublicKeyArgument {
            canister_id: None,
            derivation_path: derivation_path.clone(),
            key_id: key_id.clone(),
        };
        let EcdsaPublicKeyResponse {
            public_key,
            chain_code,
        } = ecdsa_public_key(arg).await.unwrap().0;
        assert_eq!(public_key.len(), 33);
        assert_eq!(chain_code.len(), 32);

        let message = "hello world";
        let message_hash = sha2::Sha256::digest(message).to_vec();
        let arg = SignWithEcdsaArgument {
            message_hash,
            derivation_path,
            key_id,
        };
        let SignWithEcdsaResponse { signature } = sign_with_ecdsa(arg, 10_000_000_000u128 / 13)
            .await
            .unwrap()
            .0;
        assert_eq!(signature.len(), 64);
    }
}

mod bitcoin {
    use super::*;
    use ic_cdk::api::{call::RejectionCode, management_canister::bitcoin::*};

    #[update]
    async fn execute_bitcoin_methods() {
        let address = "bcrt1qu58aj62urda83c00eylc6w34yl2s6e5rkzqet7".to_string();

        let network = BitcoinNetwork::Regtest;
        let arg = GetBalanceRequest {
            address: address.clone(),
            network,
            min_confirmations: Some(1),
        };
        let _balance = bitcoin_get_balance(arg).await.unwrap().0;

        let arg = GetUtxosRequest {
            address: address.clone(),
            network,
            filter: Some(UtxoFilter::MinConfirmations(1)),
        };
        let mut response = bitcoin_get_utxos(arg).await.unwrap().0;

        while let Some(page) = response.next_page {
            ic_cdk::println!("bitcoin_get_utxos next page");
            let arg = GetUtxosRequest {
                address: address.clone(),
                network,
                filter: Some(UtxoFilter::Page(page)),
            };
            response = bitcoin_get_utxos(arg).await.unwrap().0;
        }

        let arg = GetCurrentFeePercentilesRequest { network };
        let _percentiles = bitcoin_get_current_fee_percentiles(arg).await.unwrap().0;

        let arg = SendTransactionRequest {
            transaction: vec![],
            network,
        };
        let response = bitcoin_send_transaction(arg).await;
        assert!(response.is_err());
        if let Err((rejection_code, rejection_reason)) = response {
            assert_eq!(rejection_code, RejectionCode::CanisterReject);
            assert_eq!(&rejection_reason, "bitcoin_send_transaction failed: Can't deserialize transaction because it's malformed.");
        };
    }
}
