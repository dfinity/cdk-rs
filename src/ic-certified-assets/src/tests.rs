use crate::state_machine::{StableState, State, BATCH_EXPIRY_NANOS};
use crate::types::{
    BatchId, BatchOperation, CommitBatchArguments, CreateAssetArguments, CreateChunkArg,
    HttpRequest, HttpResponse, SetAssetContentArguments, StreamingStrategy,
};
use crate::url_decode::{url_decode, UrlDecodeError};
use candid::Principal;
use serde_bytes::ByteBuf;
use sha2::Digest;

fn some_principal() -> Principal {
    Principal::from_text("ryjl3-tyaaa-aaaaa-aaaba-cai").unwrap()
}

fn unused_callback() -> candid::Func {
    candid::Func {
        method: "unused".to_string(),
        principal: some_principal(),
    }
}

type Encodings<'a> = Vec<(&'a str, Vec<&'a [u8]>)>;

fn create_assets(
    state: &mut State,
    time_now: u64,
    assets: Vec<(&str, &str, Encodings<'_>)>,
) -> BatchId {
    let batch_id = state.create_batch(time_now);

    let mut operations = vec![];

    for (asset, content_type, encodings) in assets {
        operations.push(BatchOperation::CreateAsset(CreateAssetArguments {
            key: asset.to_string(),
            content_type: content_type.to_string(),
        }));
        for (enc, chunks) in encodings {
            let mut chunk_ids = vec![];
            for chunk in chunks {
                chunk_ids.push(
                    state
                        .create_chunk(
                            CreateChunkArg {
                                batch_id: batch_id.clone(),
                                content: ByteBuf::from(chunk.to_vec()),
                            },
                            time_now,
                        )
                        .unwrap(),
                );
            }

            operations.push(BatchOperation::SetAssetContent({
                SetAssetContentArguments {
                    key: asset.to_string(),
                    content_encoding: enc.to_string(),
                    chunk_ids,
                    sha256: None,
                }
            }));
        }
    }

    state
        .commit_batch(
            CommitBatchArguments {
                batch_id: batch_id.clone(),
                operations,
            },
            time_now,
        )
        .unwrap();

    batch_id
}

#[test]
fn can_create_assets_using_batch_api() {
    let mut state = State::default();
    let time_now = 100_000_000_000;

    const BODY: &[u8] = b"<!DOCTYPE html><html></html>";

    let batch_id = create_assets(
        &mut state,
        time_now,
        vec![(
            "/contents.html",
            "text/html",
            vec![("identity", vec![BODY])],
        )],
    );

    let response = state.http_request(
        HttpRequest {
            body: ByteBuf::new(),
            headers: vec![("Accept-Encoding".to_string(), "gzip,identity".to_string())],
            method: "GET".to_string(),
            url: "/contents.html".to_string(),
        },
        &[],
        unused_callback(),
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
fn batches_are_dropped_after_timeout() {
    let mut state = State::default();
    let time_now = 100_000_000_000;

    let batch_1 = state.create_batch(time_now);

    const BODY: &[u8] = b"<!DOCTYPE html><html></html>";

    let _chunk_1 = state
        .create_chunk(
            CreateChunkArg {
                batch_id: batch_1.clone(),
                content: ByteBuf::from(BODY.to_vec()),
            },
            time_now,
        )
        .unwrap();

    let time_now = time_now + BATCH_EXPIRY_NANOS + 1;
    let _batch_2 = state.create_batch(time_now);

    match state.create_chunk(
        CreateChunkArg {
            batch_id: batch_1,
            content: ByteBuf::from(BODY.to_vec()),
        },
        time_now,
    ) {
        Err(err) if err.contains("batch not found") => (),
        other => panic!("expected 'batch not found' error, got: {:?}", other),
    }
}

#[test]
fn returns_index_file_for_missing_assets() {
    let mut state = State::default();
    let time_now = 100_000_000_000;

    const INDEX_BODY: &[u8] = b"<!DOCTYPE html><html>Index</html>";
    const OTHER_BODY: &[u8] = b"<!DOCTYPE html><html>Other</html>";

    create_assets(
        &mut state,
        time_now,
        vec![
            (
                "/index.html",
                "text/html",
                vec![("identity", vec![INDEX_BODY])],
            ),
            (
                "/other.html",
                "text/html",
                vec![("identity", vec![OTHER_BODY])],
            ),
        ],
    );

    let response = state.http_request(
        HttpRequest {
            body: ByteBuf::new(),
            headers: vec![("Accept-Encoding".to_string(), "gzip,identity".to_string())],
            method: "GET".to_string(),
            url: "/missing.html".to_string(),
        },
        &[],
        unused_callback(),
    );

    assert_eq!(response.status_code, 200);
    assert_eq!(response.body.as_ref(), INDEX_BODY);
}

#[test]
fn preserves_state_on_stable_roundtrip() {
    let mut state = State::default();
    let time_now = 100_000_000_000;

    const INDEX_BODY: &[u8] = b"<!DOCTYPE html><html>Index</html>";

    create_assets(
        &mut state,
        time_now,
        vec![(
            "/index.html",
            "text/html",
            vec![("identity", vec![INDEX_BODY])],
        )],
    );

    let stable_state: StableState = state.into();
    let state: State = stable_state.into();

    let response = state.http_request(
        HttpRequest {
            body: ByteBuf::new(),
            headers: vec![("Accept-Encoding".to_string(), "gzip,identity".to_string())],
            method: "GET".to_string(),
            url: "/index.html".to_string(),
        },
        &[],
        unused_callback(),
    );
    assert_eq!(response.status_code, 200);
    assert_eq!(response.body.as_ref(), INDEX_BODY);
}

#[test]
fn uses_streaming_for_multichunk_assets() {
    let mut state = State::default();
    let time_now = 100_000_000_000;

    const INDEX_BODY_CHUNK_1: &[u8] = b"<!DOCTYPE html>";
    const INDEX_BODY_CHUNK_2: &[u8] = b"<html>Index</html>";

    create_assets(
        &mut state,
        time_now,
        vec![(
            "/index.html",
            "text/html",
            vec![("identity", vec![INDEX_BODY_CHUNK_1, INDEX_BODY_CHUNK_2])],
        )],
    );

    let streaming_callback = candid::Func {
        method: "stream".to_string(),
        principal: some_principal(),
    };
    let response = state.http_request(
        HttpRequest {
            body: ByteBuf::new(),
            headers: vec![("Accept-Encoding".to_string(), "gzip,identity".to_string())],
            method: "GET".to_string(),
            url: "/index.html".to_string(),
        },
        &[],
        streaming_callback.clone(),
    );

    assert_eq!(response.status_code, 200);
    assert_eq!(response.body.as_ref(), INDEX_BODY_CHUNK_1);

    let StreamingStrategy::Callback { callback, token } = response
        .streaming_strategy
        .expect("missing streaming strategy");
    assert_eq!(callback, streaming_callback);

    let streaming_response = state.http_request_streaming_callback(token).unwrap();
    assert_eq!(streaming_response.body.as_ref(), INDEX_BODY_CHUNK_2);
    assert!(
        streaming_response.token.is_none(),
        "Unexpected streaming response: {:?}",
        streaming_response
    );
}

#[test]
fn supports_etag_caching() {
    let mut state = State::default();
    let time_now = 100_000_000_000;

    const BODY: &[u8] = b"<!DOCTYPE html><html></html>";
    let hash: [u8; 32] = sha2::Sha256::digest(BODY).into();
    let etag = hex::encode(hash);

    create_assets(
        &mut state,
        time_now,
        vec![(
            "/contents.html",
            "text/html",
            vec![("identity", vec![BODY])],
        )],
    );

    let response = state.http_request(
        HttpRequest {
            body: ByteBuf::new(),
            headers: vec![("Accept-Encoding".to_string(), "gzip,identity".to_string())],
            method: "GET".to_string(),
            url: "/contents.html".to_string(),
        },
        &[],
        unused_callback(),
    );

    assert_eq!(response.status_code, 200);
    assert_eq!(response.body.as_ref(), BODY);
    assert!(
        response
            .headers
            .contains(&("ETag".to_string(), etag.clone())),
        "No matching ETag header in response: {:#?}, expected ETag {}",
        response,
        etag
    );
    assert!(
        response
            .headers
            .iter()
            .any(|(name, _)| name.eq_ignore_ascii_case("IC-Certificate")),
        "No IC-Certificate header in response: {:#?}",
        response
    );

    let response = state.http_request(
        HttpRequest {
            body: ByteBuf::new(),
            headers: vec![
                ("Accept-Encoding".to_string(), "gzip,identity".to_string()),
                ("If-None-Match".to_string(), etag),
            ],
            method: "GET".to_string(),
            url: "/contents.html".to_string(),
        },
        &[],
        unused_callback(),
    );

    assert_eq!(response.status_code, 304);
    assert_eq!(response.body.as_ref(), &[] as &[u8]);
}

#[test]
fn returns_400_on_invalid_etag() {
    let mut state = State::default();
    let time_now = 100_000_000_000;

    const BODY: &[u8] = b"<!DOCTYPE html><html></html>";

    create_assets(
        &mut state,
        time_now,
        vec![(
            "/contents.html",
            "text/html",
            vec![("identity", vec![BODY])],
        )],
    );

    let response = state.http_request(
        HttpRequest {
            body: ByteBuf::new(),
            headers: vec![
                ("Accept-Encoding".to_string(), "gzip,identity".to_string()),
                ("If-None-Match".to_string(), "cafe".to_string()),
            ],
            method: "GET".to_string(),
            url: "/contents.html".to_string(),
        },
        &[],
        unused_callback(),
    );

    assert_eq!(response.status_code, 400);
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
        &state.http_request(fake("aaaaa-aa.raw.ic0.app"), &fake_cert, unused_callback()),
        "https://aaaaa-aa.ic0.app/asset.blob",
    );
    assert_308(
        &state.http_request(
            fake("my.http.files.raw.ic0.app"),
            &fake_cert,
            unused_callback(),
        ),
        "https://my.http.files.ic0.app/asset.blob",
    );
    assert_308(
        &state.http_request(
            fake("raw.ic0.app.raw.ic0.app"),
            &fake_cert,
            unused_callback(),
        ),
        "https://raw.ic0.app.ic0.app/asset.blob",
    );
    assert_308(
        &state.http_request(fake("raw.ic0.app"), &fake_cert, unused_callback()), // for ?canisterId=
        "https://ic0.app/asset.blob",
    );
    let no_redirect = state
        .http_request(fake("raw.ic0.app.ic0.app"), &fake_cert, unused_callback())
        .status_code;
    assert!(!matches!(no_redirect, 308));

    let no_redirect2 = state
        .http_request(fake("straw.ic0.app"), &fake_cert, unused_callback())
        .status_code;
    assert!(!matches!(no_redirect2, 308));
}

#[test]
fn check_url_decode() {
    assert_eq!(
        url_decode("/%"),
        Err(UrlDecodeError::InvalidPercentEncoding)
    );
    assert_eq!(url_decode("/%%"), Ok("/%".to_string()));
    assert_eq!(url_decode("/%20a"), Ok("/ a".to_string()));
    assert_eq!(
        url_decode("/%%+a%20+%@"),
        Err(UrlDecodeError::InvalidPercentEncoding)
    );
    assert_eq!(
        url_decode("/has%percent.txt"),
        Err(UrlDecodeError::InvalidPercentEncoding)
    );
    assert_eq!(url_decode("/%e6"), Ok("/æ".to_string()));
}
