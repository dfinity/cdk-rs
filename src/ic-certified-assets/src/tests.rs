use crate::state_machine::State;
use crate::types::{
    BatchOperation, CommitBatchArguments, CreateAssetArguments, CreateChunkArg, HttpRequest,
    HttpResponse, SetAssetContentArguments,
};
use serde_bytes::ByteBuf;

#[test]
fn can_create_assets_using_batch_api() {
    let mut state = State::default();
    let time_now = 100_000_000_000;

    let batch_id = state.create_batch(time_now);

    const BODY: &[u8] = b"<!DOCTYPE html><html></html>";

    let chunk_id = state
        .create_chunk(
            CreateChunkArg {
                batch_id: batch_id.clone(),
                content: ByteBuf::from(BODY.to_vec()),
            },
            time_now,
        )
        .unwrap();

    state
        .commit_batch(
            CommitBatchArguments {
                batch_id: batch_id.clone(),
                operations: vec![
                    BatchOperation::CreateAsset(CreateAssetArguments {
                        key: "/contents.html".to_string(),
                        content_type: "text/html".to_string(),
                    }),
                    BatchOperation::SetAssetContent({
                        SetAssetContentArguments {
                            key: "/contents.html".to_string(),
                            content_encoding: "identity".to_string(),
                            chunk_ids: vec![chunk_id.clone()],
                            sha256: None,
                        }
                    }),
                ],
            },
            time_now,
        )
        .unwrap();

    let response = state.http_request(
        HttpRequest {
            body: ByteBuf::new(),
            headers: vec![("Accept-Encoding".to_string(), "gzip,identity".to_string())],
            method: "GET".to_string(),
            url: "/contents.html".to_string(),
        },
        &[],
    );

    assert_eq!(response.status_code, 200);
    assert_eq!(response.body.as_ref(), BODY);

    // Try to update a completed batch.
    let error_msg = state
        .create_chunk(
            CreateChunkArg {
                batch_id,
                content: ByteBuf::new(),
            },
            time_now,
        )
        .unwrap_err();

    let expected = "batch not found";
    assert!(
        error_msg.contains(expected),
        "expected '{}' error, got: {}",
        expected,
        error_msg
    );
}

#[test]
fn redirects_cleanly() {
    fn fake(host: &str) -> HttpRequest {
        HttpRequest {
            body: ByteBuf::new(),
            headers: vec![("Host".to_string(), host.to_string())],
            method: "GET".to_string(),
            url: "/asset.blob".to_string(),
        }
    }
    fn assert_308(resp: &HttpResponse, expected: &str) {
        assert_eq!(resp.status_code, 308);
        assert!(resp
            .headers
            .iter()
            .any(|(key, value)| key == "Location" && value == expected));
    }

    let state = State::default();
    let fake_cert = [0xca, 0xfe];

    assert_308(
        &state.http_request(fake("aaaaa-aa.raw.ic0.app"), &fake_cert),
        "https://aaaaa-aa.ic0.app/asset.blob",
    );
    assert_308(
        &state.http_request(fake("my.http.files.raw.ic0.app"), &fake_cert),
        "https://my.http.files.ic0.app/asset.blob",
    );
    assert_308(
        &state.http_request(fake("raw.ic0.app.raw.ic0.app"), &fake_cert),
        "https://raw.ic0.app.ic0.app/asset.blob",
    );
    assert_308(
        &state.http_request(fake("raw.ic0.app"), &fake_cert), // for ?canisterId=
        "https://ic0.app/asset.blob",
    );
    let no_redirect = state
        .http_request(fake("raw.ic0.app.ic0.app"), &fake_cert)
        .status_code;
    assert!(!matches!(no_redirect, 308));

    let no_redirect2 = state
        .http_request(fake("straw.ic0.app"), &fake_cert)
        .status_code;
    assert!(!matches!(no_redirect2, 308));
}
