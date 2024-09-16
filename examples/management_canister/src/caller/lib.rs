use ic_cdk::*;

mod http_request {
    use super::*;
    use ic_cdk::api::management_canister::http_request::*;

    fn http_request_required_cycles(arg: &CanisterHttpRequestArgument) -> u128 {
        let max_response_bytes = match arg.max_response_bytes {
            Some(ref n) => *n as u128,
            None => 2 * 1024 * 1024u128, // default 2MiB
        };
        let arg_raw = candid::utils::encode_args((arg,)).expect("Failed to encode arguments.");
        // The fee is for a 13-node subnet to demonstrate a typical usage.
        (3_000_000u128
            + 60_000u128 * 13
            + (arg_raw.len() as u128 + "http_request".len() as u128) * 400
            + max_response_bytes * 800)
            * 13
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
        assert_eq!(response.status, 200u8);
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
        let SignWithEcdsaResponse { signature } = sign_with_ecdsa(arg).await.unwrap().0;
        assert_eq!(signature.len(), 64);
    }
}

mod schnorr {
    use super::*;
    use ic_cdk::api::management_canister::schnorr::*;

    #[update]
    async fn execute_schnorr_methods() {
        let key_id = SchnorrKeyId {
            algorithm: SchnorrAlgorithm::Bip340secp256k1,
            name: "dfx_test_key".to_string(),
        };
        let derivation_path = vec![];
        let arg = SchnorrPublicKeyArgument {
            canister_id: None,
            derivation_path: derivation_path.clone(),
            key_id: key_id.clone(),
        };
        let SchnorrPublicKeyResponse {
            public_key,
            chain_code,
        } = schnorr_public_key(arg).await.unwrap().0;
        assert_eq!(public_key.len(), 33);
        assert_eq!(chain_code.len(), 32);

        let message = "hello world".into();
        let arg = SignWithSchnorrArgument {
            message,
            derivation_path,
            key_id,
        };
        let SignWithSchnorrResponse { signature } = sign_with_schnorr(arg).await.unwrap().0;
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
            assert_eq!(
                &rejection_reason,
                "send_transaction failed: MalformedTransaction"
            );
        };
    }
}

ic_cdk::export_candid!();
