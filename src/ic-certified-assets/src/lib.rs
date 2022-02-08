mod rc_bytes;

use crate::rc_bytes::RcBytes;
use ic_cdk::api::{caller, data_certificate, set_certified_data, time, trap};
use ic_cdk::export::candid::{CandidType, Deserialize, Func, Int, Nat, Principal};
use ic_cdk_macros::{query, update};
use ic_certified_map::{AsHashTree, Hash, HashTree, RbTree};
use num_traits::ToPrimitive;
use serde::Serialize;
use serde_bytes::ByteBuf;
use sha2::Digest;
use std::cell::RefCell;
use std::collections::HashMap;
use std::convert::TryInto;
use std::fmt;

/// The amount of time a batch is kept alive. Modifying the batch
/// delays the expiry further.
const BATCH_EXPIRY_NANOS: u64 = 300_000_000_000;

/// The order in which we pick encodings for certification.
const ENCODING_CERTIFICATION_ORDER: &[&str] = &["identity", "gzip", "compress", "deflate", "br"];

/// The file to serve if the requested file wasn't found.
const INDEX_FILE: &str = "/index.html";

const MAX_CHUNK_SIZE: usize = 3_000_000;

thread_local! {
    static STATE: State = State::default();
    static ASSET_HASHES: RefCell<AssetHashes> = RefCell::new(RbTree::new());
}

type AssetHashes = RbTree<Key, Hash>;

#[derive(Default)]
struct State {
    assets: RefCell<HashMap<Key, Asset>>,

    chunks: RefCell<HashMap<ChunkId, Chunk>>,
    next_chunk_id: RefCell<ChunkId>,

    batches: RefCell<HashMap<BatchId, Batch>>,
    next_batch_id: RefCell<BatchId>,

    authorized: RefCell<Vec<Principal>>,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct StableState {
    authorized: Vec<Principal>,
    stable_assets: HashMap<String, Asset>,
}

#[derive(Default, Clone, Debug, CandidType, Deserialize)]
struct AssetEncoding {
    modified: Timestamp,
    content_chunks: Vec<ContentChunk>,
    total_length: usize,
    certified: bool,
    sha256: [u8; 32],
}

#[derive(Clone, Debug, CandidType, Deserialize)]
struct ContentChunk {
    content: RcBytes,
    start_byte: u64,
    end_byte: u64
}

#[derive(Default, Clone, Debug, CandidType, Deserialize)]
struct Asset {
    content_type: String,
    encodings: HashMap<String, AssetEncoding>,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
struct EncodedAsset {
    content: RcBytes,
    content_type: String,
    content_encoding: String,
    total_length: Nat,
    sha256: Option<ByteBuf>,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
struct AssetDetails {
    key: String,
    content_type: String,
    encodings: Vec<AssetEncodingDetails>,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
struct AssetEncodingDetails {
    content_encoding: String,
    sha256: Option<ByteBuf>,
    length: Nat,
    modified: Timestamp,
}

struct Chunk {
    batch_id: BatchId,
    content: RcBytes,
}

struct Batch {
    expires_at: Timestamp,
}

type Timestamp = Int;
type BatchId = Nat;
type ChunkId = Nat;
type Key = String;

// IDL Types

#[derive(Clone, Debug, CandidType, Deserialize)]
struct CreateAssetArguments {
    key: Key,
    content_type: String,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
struct SetAssetContentArguments {
    key: Key,
    content_encoding: String,
    chunk_ids: Vec<ChunkId>,
    sha256: Option<ByteBuf>,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
struct UnsetAssetContentArguments {
    key: Key,
    content_encoding: String,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
struct DeleteAssetArguments {
    key: Key,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
struct ClearArguments {}

#[derive(Clone, Debug, CandidType, Deserialize)]
enum BatchOperation {
    CreateAsset(CreateAssetArguments),
    SetAssetContent(SetAssetContentArguments),
    UnsetAssetContent(UnsetAssetContentArguments),
    DeleteAsset(DeleteAssetArguments),
    Clear(ClearArguments),
}

#[derive(Clone, Debug, CandidType, Deserialize)]
struct CommitBatchArguments {
    batch_id: BatchId,
    operations: Vec<BatchOperation>,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
struct StoreArg {
    key: Key,
    content_type: String,
    content_encoding: String,
    content: ByteBuf,
    sha256: Option<ByteBuf>,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
struct GetArg {
    key: Key,
    accept_encodings: Vec<String>,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
struct GetChunkArg {
    key: Key,
    content_encoding: String,
    index: Nat,
    sha256: Option<ByteBuf>,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
struct GetChunkResponse {
    content: RcBytes,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
struct CreateBatchResponse {
    batch_id: BatchId,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
struct CreateChunkArg {
    batch_id: BatchId,
    content: ByteBuf,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
struct CreateChunkResponse {
    chunk_id: ChunkId,
}
// HTTP interface

type HeaderField = (String, String);

#[derive(Clone, Debug, CandidType, Deserialize)]
struct HttpRequest {
    method: String,
    url: String,
    headers: Vec<(String, String)>,
    body: ByteBuf,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
struct HttpResponse {
    status_code: u16,
    headers: Vec<HeaderField>,
    body: RcBytes,
    streaming_strategy: Option<StreamingStrategy>,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
struct StreamingCallbackToken {
    key: String,
    content_encoding: String,
    index: Nat,
    // We don't care about the sha, we just want to be backward compatible.
    sha256: Option<ByteBuf>,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
enum StreamingStrategy {
    Callback {
        callback: Func,
        token: StreamingCallbackToken,
    },
}

#[derive(Clone, Debug, CandidType, Deserialize)]
struct StreamingCallbackHttpResponse {
    body: RcBytes,
    token: Option<StreamingCallbackToken>,
}

#[update]
fn authorize(other: Principal) {
    let caller = caller();
    STATE.with(|s| {
        let caller_autorized = s.authorized.borrow().iter().any(|p| *p == caller);
        if caller_autorized {
            s.authorized.borrow_mut().push(other);
        }
    })
}

#[query]
fn retrieve(key: Key) -> RcBytes {
    STATE.with(|s| {
        let assets = s.assets.borrow();
        let asset = assets.get(&key).unwrap_or_else(|| trap("asset not found"));
        let id_enc = asset
            .encodings
            .get("identity")
            .unwrap_or_else(|| trap("no identity encoding"));
        if id_enc.content_chunks.len() > 1 {
            trap("Asset too large. Use get() and get_chunk() instead.");
        }
        id_enc.content_chunks[0].content.clone()
    })
}

#[update(guard = "is_authorized")]
fn store(arg: StoreArg) {
    STATE.with(move |s| {
        let mut assets = s.assets.borrow_mut();
        let asset = assets.entry(arg.key.clone()).or_default();
        asset.content_type = arg.content_type;

        let hash = hash_bytes(&arg.content);
        if let Some(provided_hash) = arg.sha256 {
            if hash != provided_hash.as_ref() {
                trap("sha256 mismatch");
            }
        }

        let encoding = asset.encodings.entry(arg.content_encoding).or_default();
        let content_length = arg.content.len() as u64;

        encoding.total_length = arg.content.len();
        encoding.content_chunks = vec![ContentChunk {
            content: RcBytes::from(arg.content),
            start_byte: 0,
            end_byte: content_length - 1
        }];
        encoding.modified = Int::from(time() as u64);
        encoding.sha256 = hash;

        on_asset_change(&arg.key, asset);
    });
}

#[update(guard = "is_authorized")]
fn create_batch() -> CreateBatchResponse {
    STATE.with(|s| {
        let batch_id = s.next_batch_id.borrow().clone();
        *s.next_batch_id.borrow_mut() += 1;

        let now = time() as u64;

        let mut batches = s.batches.borrow_mut();
        batches.insert(
            batch_id.clone(),
            Batch {
                expires_at: Int::from(now + BATCH_EXPIRY_NANOS),
            },
        );
        s.chunks.borrow_mut().retain(|_, c| {
            batches
                .get(&c.batch_id)
                .map(|b| b.expires_at > now)
                .unwrap_or(false)
        });
        batches.retain(|_, b| b.expires_at > now);

        CreateBatchResponse { batch_id }
    })
}

#[update(guard = "is_authorized")]
fn create_chunk(arg: CreateChunkArg) -> CreateChunkResponse {
    STATE.with(|s| {
        let mut batches = s.batches.borrow_mut();
        let now = time() as u64;
        let mut batch = batches
            .get_mut(&arg.batch_id)
            .unwrap_or_else(|| trap("batch not found"));
        batch.expires_at = Int::from(now + BATCH_EXPIRY_NANOS);

        let chunk_id = s.next_chunk_id.borrow().clone();
        *s.next_chunk_id.borrow_mut() += 1;

        s.chunks.borrow_mut().insert(
            chunk_id.clone(),
            Chunk {
                batch_id: arg.batch_id,
                content: RcBytes::from(arg.content),
            },
        );

        CreateChunkResponse { chunk_id }
    })
}

#[update(guard = "is_authorized")]
fn create_asset(arg: CreateAssetArguments) {
    do_create_asset(arg);
}

#[update(guard = "is_authorized")]
fn set_asset_content(arg: SetAssetContentArguments) {
    do_set_asset_content(arg);
}

#[update(guard = "is_authorized")]
fn unset_asset_content(arg: UnsetAssetContentArguments) {
    do_unset_asset_content(arg);
}

#[update(guard = "is_authorized")]
fn delete_content(arg: DeleteAssetArguments) {
    do_delete_asset(arg);
}

#[update(guard = "is_authorized")]
fn clear() {
    do_clear();
}

#[update(guard = "is_authorized")]
fn commit_batch(arg: CommitBatchArguments) {
    let batch_id = arg.batch_id;
    for op in arg.operations {
        match op {
            BatchOperation::CreateAsset(arg) => do_create_asset(arg),
            BatchOperation::SetAssetContent(arg) => do_set_asset_content(arg),
            BatchOperation::UnsetAssetContent(arg) => do_unset_asset_content(arg),
            BatchOperation::DeleteAsset(arg) => do_delete_asset(arg),
            BatchOperation::Clear(_) => do_clear(),
        }
    }
    STATE.with(|s| {
        s.batches.borrow_mut().remove(&batch_id);
    })
}

#[query]
fn get(arg: GetArg) -> EncodedAsset {
    STATE.with(|s| {
        let assets = s.assets.borrow();
        let asset = assets.get(&arg.key).unwrap_or_else(|| {
            trap("asset not found");
        });

        for enc in arg.accept_encodings.iter() {
            if let Some(asset_enc) = asset.encodings.get(enc) {
                return EncodedAsset {
                    content: asset_enc.content_chunks[0].content.clone(),
                    content_type: asset.content_type.clone(),
                    content_encoding: enc.clone(),
                    total_length: Nat::from(asset_enc.total_length as u64),
                    sha256: Some(ByteBuf::from(asset_enc.sha256)),
                };
            }
        }
        trap("no such encoding");
    })
}

#[query]
fn get_chunk(arg: GetChunkArg) -> GetChunkResponse {
    STATE.with(|s| {
        let assets = s.assets.borrow();
        let asset = assets
            .get(&arg.key)
            .unwrap_or_else(|| trap("asset not found"));

        let enc = asset
            .encodings
            .get(&arg.content_encoding)
            .unwrap_or_else(|| trap("no such encoding"));

        if let Some(expected_hash) = arg.sha256 {
            if expected_hash != enc.sha256 {
                trap("sha256 mismatch")
            }
        }
        if arg.index >= enc.content_chunks.len() {
            trap("chunk index out of bounds");
        }
        let index: usize = arg.index.0.to_usize().unwrap();

        GetChunkResponse {
            content: enc.content_chunks[index].content.clone(),
        }
    })
}

#[query]
fn list() -> Vec<AssetDetails> {
    STATE.with(|s| {
        s.assets
            .borrow()
            .iter()
            .map(|(key, asset)| {
                let mut encodings: Vec<_> = asset
                    .encodings
                    .iter()
                    .map(|(enc_name, enc)| AssetEncodingDetails {
                        content_encoding: enc_name.clone(),
                        sha256: Some(ByteBuf::from(enc.sha256)),
                        length: Nat::from(enc.total_length),
                        modified: enc.modified.clone(),
                    })
                    .collect();
                encodings.sort_by(|l, r| l.content_encoding.cmp(&r.content_encoding));

                AssetDetails {
                    key: key.clone(),
                    content_type: asset.content_type.clone(),
                    encodings,
                }
            })
            .collect::<Vec<_>>()
    })
}

fn create_default_strategy(
    asset: &Asset,
    enc_name: &str,
    enc: &AssetEncoding,
    key: &str,
    chunk_index: usize,
) -> Option<StreamingStrategy> {
    create_default_token(asset, enc_name, enc, key, chunk_index).map(|token| StreamingStrategy::Callback {
        callback: ic_cdk::export::candid::Func {
            method: "http_request_streaming_callback".to_string(),
            principal: ic_cdk::id(),
        },
        token,
    })
}

fn create_default_token(
    _asset: &Asset,
    enc_name: &str,
    enc: &AssetEncoding,
    key: &str,
    chunk_index: usize,
) -> Option<StreamingCallbackToken> {
    if chunk_index + 1 >= enc.content_chunks.len() {
        None
    } else {
        Some(StreamingCallbackToken {
            key: key.to_string(),
            content_encoding: enc_name.to_string(),
            index: Nat::from(chunk_index + 1),
            sha256: Some(ByteBuf::from(enc.sha256)),
        })
    }
}

// TODO I am hacking this for now by encoding the necessary information into the key field of StreamingCallbackToken
// TODO I think the better way to do this would be to update the boundary node code to allow a custom StreamingCallbackToken
// TODO See this issue: https://github.com/dfinity/certified-assets/issues/11
fn create_partial_content_strategy(
    enc_name: &str,
    enc: &AssetEncoding,
    key: &str,
    chunk_index: usize,
    start_byte: u64,
    end_byte: u64
) -> StreamingStrategy {
    StreamingStrategy::Callback {
        callback: ic_cdk::export::candid::Func {
            method: "http_request_streaming_callback".to_string(),
            principal: ic_cdk::id()
        },
        token: create_partial_content_token(
            enc_name,
            enc,
            key,
            chunk_index,
            start_byte,
            end_byte
        )
    }
}

fn create_partial_content_token(
    enc_name: &str,
    enc: &AssetEncoding,
    key: &str,
    chunk_index: usize,
    start_byte: u64,
    end_byte: u64
) -> StreamingCallbackToken {
    StreamingCallbackToken {
        key: format!(
            "RANGE::::{key}::::{start_byte}::::{end_byte}::::{partition_size}",
            key = key,
            start_byte = start_byte,
            end_byte = end_byte,
            partition_size = MAX_CHUNK_SIZE
        ),
        content_encoding: enc_name.to_string(),
        index: Nat::from(chunk_index + 1),
        sha256: Some(ByteBuf::from(enc.sha256))
    }
}

fn build_200(
    asset: &Asset,
    enc_name: &str,
    enc: &AssetEncoding,
    key: &str,
    chunk_index: usize,
    certificate_header: Option<HeaderField>,
) -> HttpResponse {
    let mut headers = vec![
        ("Accept-Ranges".to_string(), "bytes".to_string()),
        ("Content-Type".to_string(), asset.content_type.to_string())
    ];

    if enc_name != "identity" {
        headers.push(("Content-Encoding".to_string(), enc_name.to_string()));
    }

    if let Some(head) = certificate_header {
        headers.push(head);
    }

    let default_streaming_strategy = create_default_strategy(asset, enc_name, enc, key, chunk_index);

    HttpResponse {
        status_code: 200,
        headers,
        body: enc.content_chunks[chunk_index].content.clone(),
        streaming_strategy: default_streaming_strategy,
    }
}

fn build_206(
    asset: &Asset,
    enc_name: &str,
    enc: &AssetEncoding,
    key: &str,
    chunk_index: usize,
    range_request_info: &RangeRequestInfo
) -> HttpResponse {
    if range_request_info.ranges.len() == 1 {
        return build_206_single_part(
            asset,
            enc_name,
            enc,
            key,
            chunk_index,
            range_request_info
        );
    }
    
    // TODO Multipart ranges have not yet been implemented
    // if range_request_info.ranges.len() > 1 {
        // return build_206_multipart(
            // asset,
            // enc_name,
            // enc,
            // range_request_info
        // );
    // }

    build_416(&enc.total_length.to_string())
}

fn build_206_single_part(
    asset: &Asset,
    enc_name: &str,
    enc: &AssetEncoding,
    key: &str,
    chunk_index: usize,
    range_request_info: &RangeRequestInfo
) -> HttpResponse {
    let range_option = &range_request_info.ranges.get(0);

    if let Some(range) = range_option {
        let total_bytes: u64 = enc.total_length.try_into().unwrap();

        let start_and_end_byte_result = get_start_and_end_byte_for_range(
            total_bytes,
            range
        );
    
        match start_and_end_byte_result {
            Ok((start_byte, end_byte)) => {
                let body_result = get_range_request_body(
                    enc,
                    start_byte,
                    end_byte
                );
        
                handle_206_single_part_body_result(
                    body_result,
                    total_bytes,
                    start_byte,
                    end_byte,
                    asset,
                    enc_name,
                    enc,
                    key,
                    chunk_index
                )
            },
            Err(http_416_response) => http_416_response
        }
    }
    else {
        build_416(&enc.total_length.to_string())
    }
}

fn get_start_and_end_byte_for_range(
    total_bytes: u64,
    range: &Range
) -> Result<(u64, u64), HttpResponse> {
    match (range.start_byte, range.end_byte) {
        (Some(start_byte), Some(end_byte)) => {
            if
                start_byte >= total_bytes ||
                end_byte < start_byte
            {
                return Err(build_416(&total_bytes.to_string()));
            }

            if end_byte >= total_bytes {
                return Ok((start_byte, total_bytes - 1));
            }

            Ok((start_byte, end_byte))
        },
        (Some(start_byte), None) => {
            if start_byte >= total_bytes {
                return Err(build_416(&total_bytes.to_string()));
            }

            Ok((start_byte, total_bytes - 1))
        },
        (None, Some(end_byte)) => {
            if end_byte >= total_bytes {
                Ok((0, total_bytes - 1))
            }
            else {
                Ok((total_bytes - end_byte - 1, total_bytes - 1)) // TODO I hope I am not off by one for the start_byte value
            }
        },
        (None, None) => Err(build_416(&total_bytes.to_string()))
    }
}

fn handle_206_single_part_body_result(
    body_result: Result<RcBytes, HttpResponse>,
    total_bytes: u64,
    start_byte: u64,
    end_byte: u64,
    asset: &Asset,
    enc_name: &str,
    enc: &AssetEncoding,
    key: &str,
    chunk_index: usize
) -> HttpResponse {
    match body_result {
        Ok(body) => {
            let mut headers = vec![
                ("Accept-Ranges".to_string(), "bytes".to_string()),
                ("Content-Length".to_string(), body.len().to_string()),
                ("Content-Range".to_string(), format!(
                    "bytes {start_byte}-{end_byte}/{total_bytes}",
                    start_byte = start_byte,
                    end_byte = end_byte,
                    total_bytes = total_bytes
                )),
                ("Content-Type".to_string(), asset.content_type.to_string())
            ];
    
            if enc_name != "identity" {
                headers.push(("Content-Encoding".to_string(), enc_name.to_string()));
            }
    
            if body.len() > MAX_CHUNK_SIZE {
                let body = RcBytes(std::rc::Rc::new(ByteBuf::from(body[..MAX_CHUNK_SIZE].to_vec())));
    
                let streaming_strategy = Some(create_partial_content_strategy(
                    enc_name,
                    enc,
                    key,
                    chunk_index,
                    start_byte,
                    end_byte
                ));
            
                HttpResponse {
                    status_code: 206,
                    headers,
                    body,
                    streaming_strategy
                }
            }
            else {
                HttpResponse {
                    status_code: 206,
                    headers,
                    body,
                    streaming_strategy: None
                }
            }
        },
        Err(http_416_response) => http_416_response
    }
}

// TODO Multipart ranges have not yet been implemented
// fn build_206_multipart(
//     asset: &Asset,
//     enc_name: &str,
//     enc: &AssetEncoding,
//     range_request_info: RangeRequestInfo // TODO should this be a reference?
// ) -> HttpResponse {
//     ic_cdk::println!("build_206_multipart");

//     let mut headers = vec![
//         ("Accept-Ranges".to_string(), "bytes".to_string()), // TODO when should accept-ranges be returned?
//         ("Content-Length".to_string(), enc.content_chunks[0].content.len().to_string()),
//         ("Content-Type".to_string(), "multipart/byteranges; boundary=3d6b6a416f9b5".to_string())
//     ];
    
//     if enc_name != "identity" {
//         headers.push(("Content-Encoding".to_string(), enc_name.to_string()));
//     }

//     HttpResponse {
//         status_code: 206,
//         headers,
//         body: enc.content_chunks[0].content.clone(),
//         streaming_strategy: None
//     }
// }

fn build_404(certificate_header: HeaderField) -> HttpResponse {
    HttpResponse {
        status_code: 404,
        headers: vec![certificate_header],
        body: RcBytes::from(ByteBuf::from("not found")),
        streaming_strategy: None,
    }
}

// TODO Do I need the certificate_header here?
fn build_416(length: &str) -> HttpResponse {
    HttpResponse {
        status_code: 416,
        headers: vec![("Content-Range".to_string(), format!("*/{length}", length = length))],
        body: RcBytes(std::rc::Rc::new(ByteBuf::from(vec![]))),
        streaming_strategy: None,
    }
}

fn get_range_request_body(
    enc: &AssetEncoding,
    start_byte: u64,
    end_byte: u64
) -> Result<RcBytes, HttpResponse> {
    // TODO see if we can do this performantly and immutably
    let final_slice_result = enc
        .content_chunks
        .iter()
        .try_fold(vec![], |mut result, content_chunk| {
            let start_byte_before_content_chunk = start_byte < content_chunk.start_byte;
            let start_byte_after_content_chunk = start_byte > content_chunk.end_byte;
            let start_byte_within_content_chunk = 
                start_byte >= content_chunk.start_byte &&
                start_byte <= content_chunk.end_byte;

            let end_byte_before_content_chunk = end_byte < content_chunk.start_byte;
            let end_byte_after_content_chunk = end_byte > content_chunk.end_byte;
            let end_byte_within_content_chunk =
                end_byte >= content_chunk.start_byte &&
                end_byte <= content_chunk.end_byte;

            let virtual_start_byte = start_byte - content_chunk.start_byte;
            let virtual_end_byte = end_byte - content_chunk.start_byte + 1;

            if
                start_byte_before_content_chunk &&
                end_byte_before_content_chunk
            {
                return Ok(result);
            }

            if
                start_byte_before_content_chunk &&
                end_byte_after_content_chunk
            {
                result.append(&mut content_chunk.content.to_vec());
            }

            if
                start_byte_before_content_chunk &&
                end_byte_within_content_chunk
            {
                result.append(&mut content_chunk.content[..virtual_end_byte as usize].to_vec());
            }

            if
                start_byte_after_content_chunk &&
                end_byte_before_content_chunk
            {
                return Err(build_416(&enc.total_length.to_string()));
            }

            if
                start_byte_after_content_chunk &&
                end_byte_after_content_chunk
            {
                return Ok(result);
            }

            if
                start_byte_after_content_chunk &&
                end_byte_within_content_chunk
            {
                return Err(build_416(&enc.total_length.to_string()));
            }

            if
                start_byte_within_content_chunk &&
                end_byte_before_content_chunk
            {
                return Err(build_416(&enc.total_length.to_string()));
            }

            if
                start_byte_within_content_chunk &&
                end_byte_after_content_chunk
            {
                result.append(&mut content_chunk.content[virtual_start_byte as usize..].to_vec());
            }

            if
                start_byte_within_content_chunk &&
                end_byte_within_content_chunk
            {
                result.append(&mut content_chunk.content[virtual_start_byte as usize..virtual_end_byte as usize].to_vec());
            }

            Ok(result)
        });

    match final_slice_result {
        Ok(final_slice) => Ok(RcBytes(std::rc::Rc::new(ByteBuf::from(final_slice)))),
        Err(http_response) => Err(http_response)
    }
}

// TODO not sure what to do with certification for 206 responses
fn build_http_response(
    path: &str,
    encodings: Vec<String>,
    range_request_info_option: Option<RangeRequestInfo>,
    index: usize
) -> HttpResponse {
    STATE.with(|s| {
        let assets = s.assets.borrow();

        let index_redirect_certificate = ASSET_HASHES.with(|t| {
            let tree = t.borrow();
            if tree.get(path.as_bytes()).is_none() && tree.get(INDEX_FILE.as_bytes()).is_some() {
                let absence_proof = tree.witness(path.as_bytes());
                let index_proof = tree.witness(INDEX_FILE.as_bytes());
                let combined_proof = merge_hash_trees(absence_proof, index_proof);
                Some(witness_to_header(combined_proof))
            } else {
                None
            }
        });

        if let Some(certificate_header) = index_redirect_certificate {
            if let Some(asset) = assets.get(INDEX_FILE) {
                for enc_name in encodings.iter() {
                    if let Some(enc) = asset.encodings.get(enc_name) {
                        if enc.certified {
                            if let Some(range_request_info) = range_request_info_option {
                                return build_206(
                                    asset,
                                    enc_name,
                                    enc,
                                    INDEX_FILE,
                                    index,
                                    &range_request_info
                                );
                            }
                            else {
                                return build_200(
                                    asset,
                                    enc_name,
                                    enc,
                                    INDEX_FILE,
                                    index,
                                    Some(certificate_header),
                                );
                            }
                        }
                    }
                }
            }
        }

        let certificate_header =
            ASSET_HASHES.with(|t| witness_to_header(t.borrow().witness(path.as_bytes())));

        if let Some(asset) = assets.get(path) {
            for enc_name in encodings.iter() {
                if let Some(enc) = asset.encodings.get(enc_name) {
                    if enc.certified {
                        if let Some(range_request_info) = range_request_info_option {
                            return build_206(
                                asset,
                                enc_name,
                                enc,
                                path,
                                index,
                                &range_request_info
                            );
                        }
                        else {
                            return build_200(
                                asset,
                                enc_name,
                                enc,
                                path,
                                index,
                                Some(certificate_header),
                            );
                        }
                    } else {
                        // Find if identity is certified, if it's not.
                        if let Some(id_enc) = asset.encodings.get("identity") {
                            if id_enc.certified {
                                if let Some(range_request_info) = range_request_info_option {
                                    return build_206(
                                        asset,
                                        enc_name,
                                        enc,
                                        path,
                                        index,
                                        &range_request_info
                                    );
                                }
                                else {
                                    return build_200(
                                        asset,
                                        enc_name,
                                        enc,
                                        path,
                                        index,
                                        Some(certificate_header),
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }

        build_404(certificate_header)
    })
}

/// An iterator-like structure that decode a URL.
struct UrlDecode<'a> {
    bytes: std::slice::Iter<'a, u8>,
}

fn convert_percent(iter: &mut std::slice::Iter<u8>) -> Option<u8> {
    let mut cloned_iter = iter.clone();
    let result = match cloned_iter.next()? {
        b'%' => b'%',
        h => {
            let h = char::from(*h).to_digit(16)?;
            let l = char::from(*cloned_iter.next()?).to_digit(16)?;
            h as u8 * 0x10 + l as u8
        }
    };
    // Update this if we make it this far, otherwise "reset" the iterator.
    *iter = cloned_iter;
    Some(result)
}

#[derive(Debug, PartialEq)]
pub enum UrlDecodeError {
    InvalidPercentEncoding,
}

impl fmt::Display for UrlDecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidPercentEncoding => write!(f, "invalid percent encoding"),
        }
    }
}

impl<'a> Iterator for UrlDecode<'a> {
    type Item = Result<char, UrlDecodeError>;

    fn next(&mut self) -> Option<Self::Item> {
        let b = self.bytes.next()?;
        match b {
            b'%' => Some(
                convert_percent(&mut self.bytes)
                    .map(char::from)
                    .ok_or(UrlDecodeError::InvalidPercentEncoding),
            ),
            b'+' => Some(Ok(' ')),
            x => Some(Ok(char::from(*x))),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let bytes = self.bytes.len();
        (bytes / 3, Some(bytes))
    }
}

fn url_decode(url: &str) -> Result<String, UrlDecodeError> {
    UrlDecode {
        bytes: url.as_bytes().iter(),
    }
    .collect()
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

#[query]
fn http_request(req: HttpRequest) -> HttpResponse {
    let mut encodings = vec![];
    for (name, value) in req.headers.iter() {
        if name.eq_ignore_ascii_case("Accept-Encoding") {
            for v in value.split(',') {
                encodings.push(v.trim().to_string());
            }
        }
    }
    encodings.push("identity".to_string());

    let range_request_info_result = get_range_request_info(&req);

    match range_request_info_result {
        Ok(range_request_info) => {
            let path = match req.url.find('?') {
                Some(i) => &req.url[..i],
                None => &req.url[..],
            };
            
            match url_decode(path) {
                Ok(path) => build_http_response(&path, encodings, range_request_info, 0),
                Err(err) => HttpResponse {
                    status_code: 400,
                    headers: vec![],
                    body: RcBytes::from(ByteBuf::from(format!(
                        "failed to decode path '{}': {}",
                        path, err
                    ))),
                    streaming_strategy: None,
                },
            }
        },
        Err(http_416_response) => http_416_response
    }
}

#[query]
fn http_request_streaming_callback(
    StreamingCallbackToken {
        key,
        content_encoding,
        index,
        sha256,
    }: StreamingCallbackToken,
) -> StreamingCallbackHttpResponse {
    STATE.with(|s| {
        let key_contents = key.split("::::").collect::<Vec<&str>>();

        if key_contents[0] != "RANGE" {
            handle_default_streaming_callback(
                s,
                &key,
                &content_encoding,
                sha256,
                index
            )
        }
        else {
            handle_range_streaming_callback(
                &key_contents,
                s,
                &content_encoding,
                sha256,
                index
            )
        }
    })
}

fn handle_default_streaming_callback(
    s: &State,
    key: &str,
    content_encoding: &str,
    sha256: Option<ByteBuf>,
    index: Nat
) -> StreamingCallbackHttpResponse {
    let assets = s.assets.borrow();
    let asset = assets
        .get(key)
        .expect("Invalid token on streaming: key not found.");
    let enc = asset
        .encodings
        .get(content_encoding)
        .expect("Invalid token on streaming: encoding not found.");

    if let Some(expected_hash) = sha256 {
        if expected_hash != enc.sha256 {
            trap("sha256 mismatch");
        }
    }

    // MAX is good enough. This means a chunk would be above 64-bits, which is impossible...
    let chunk_index = index.0.to_usize().unwrap_or(usize::MAX);

    StreamingCallbackHttpResponse {
        body: enc.content_chunks[chunk_index].content.clone(),
        token: create_default_token(&asset, &content_encoding, enc, &key, chunk_index),
    }
}

fn handle_range_streaming_callback(
    key_contents: &Vec<&str>,
    s: &State,
    content_encoding: &str,
    sha256: Option<ByteBuf>,
    index: Nat
) -> StreamingCallbackHttpResponse {
    let key = key_contents[1];

    let start_byte = key_contents[2].parse::<usize>().expect("Invalid RANGE encoding: start_byte");
    let end_byte = key_contents[3].parse::<usize>().expect("Invalid RANGE encoding: end_byte");
    let partition_size = key_contents[4].parse::<usize>().expect("Invalid RANGE encoding: partition_size");

    let assets = s.assets.borrow();
    let asset = assets
        .get(key)
        .expect("Invalid token on streaming: key not found.");
    let enc = asset
        .encodings
        .get(content_encoding)
        .expect("Invalid token on streaming: encoding not found.");

    if let Some(expected_hash) = sha256 {
        if expected_hash != enc.sha256 {
            trap("sha256 mismatch");
        }
    }

    let full_body = get_range_request_body(
        enc,
        start_byte as u64,
        end_byte as u64 // TODO probably bad conversion
    ).expect("Invalid range request body");

    // MAX is good enough. This means a chunk would be above 64-bits, which is impossible...
    let chunk_index = index.0.to_usize().unwrap_or(usize::MAX);

    let partial_start_byte = chunk_index * partition_size;
    let partial_end_byte = if partial_start_byte + partition_size > full_body.len() { full_body.len() - 1 } else { partial_start_byte + partition_size - 1 };

    let partial_body = &full_body[partial_start_byte..=partial_end_byte];

    let token = if partial_start_byte + partition_size <= full_body.len() { Some(create_partial_content_token(
        &content_encoding,
        enc,
        key,
        chunk_index,
        start_byte as u64,
        end_byte as u64
    )) } else { None };

    StreamingCallbackHttpResponse {
        body: RcBytes(std::rc::Rc::new(ByteBuf::from(partial_body))),
        token
    }
}

// TODO is u64 the appropriate unit to use?
#[derive(Debug)]
struct RangeRequestInfo {
    ranges: Vec<Range>,
    if_range: Option<u64>
}

#[derive(Debug)]
struct Range {
    start_byte: Option<u64>,
    end_byte: Option<u64>
}

fn get_range_request_info(req: &HttpRequest) -> Result<Option<RangeRequestInfo>, HttpResponse> {
    let range_header_option = req.headers.iter().find(|header| {
        header.0.to_lowercase() == "range"
    });

    if let Some(range_header) = range_header_option {
        if req.method.to_lowercase() == "post" {
            return Ok(None);
        }

        let ranges_result = get_ranges(&range_header.1);

        match ranges_result {
            Ok(ranges) => Ok(Some(RangeRequestInfo {
                ranges,
                if_range: None // TODO implement if_range
            })),
            Err(http_416_response) => Err(http_416_response)
        }
    }
    else {
        Ok(None)
    }
}

fn get_ranges(range_header_value: &str) -> Result<Vec<Range>, HttpResponse> {
    let range_strings = range_header_value.split(",");

    range_strings
        .map(|range_string| {
            let range_string_without_prefix = range_string.replace("bytes=", "");

            let bytes_string = range_string_without_prefix.split("-").collect::<Vec<&str>>();

            let start_byte_string_option = bytes_string.get(0);
            let end_byte_string_option = bytes_string.get(1);

            match (start_byte_string_option, end_byte_string_option) {
                (Some(start_byte_string), Some(end_byte_string)) => {
                    let start_byte_result = get_range_start_byte(start_byte_string);
                    let end_byte_result = get_range_end_byte(end_byte_string);

                    match (start_byte_result, end_byte_result) {
                        (Ok(start_byte), Ok(end_byte)) => {
                            Ok(Range {
                                start_byte,
                                end_byte
                            })
                        },
                        _ => Err(build_416("*"))
                    }
                },
                _ => Err(build_416("*"))
            }
        })
        .collect::<Result<Vec<Range>, HttpResponse>>()
}

fn get_range_start_byte(start_byte_string: &str) -> Result<Option<u64>, ()> {
    if start_byte_string == "" {
        return Ok(None);
    }
    else {
        let start_byte_result = start_byte_string.parse::<u64>();
        
        match start_byte_result {
            Ok(start_byte) => Ok(Some(start_byte)),
            Err(_) => Err(())
        }
    }
}

fn get_range_end_byte(end_byte_string: &str) -> Result<Option<u64>, ()> {
    if end_byte_string == "" {
        return Ok(None);
    }
    else {
        let end_byte_result = end_byte_string.parse::<u64>();
        
        match end_byte_result {
            Ok(end_byte) => Ok(Some(end_byte)),
            Err(_) => Err(())
        }
    }
}

fn do_create_asset(arg: CreateAssetArguments) {
    STATE.with(|s| {
        let mut assets = s.assets.borrow_mut();
        if let Some(asset) = assets.get(&arg.key) {
            if asset.content_type != arg.content_type {
                trap("create_asset: content type mismatch");
            }
        } else {
            assets.insert(
                arg.key,
                Asset {
                    content_type: arg.content_type,
                    encodings: HashMap::new(),
                },
            );
        }
    })
}

fn do_set_asset_content(arg: SetAssetContentArguments) {
    STATE.with(|s| {
        if arg.chunk_ids.is_empty() {
            trap("encoding must have at least one chunk");
        }

        let mut assets = s.assets.borrow_mut();
        let asset = assets
            .get_mut(&arg.key)
            .unwrap_or_else(|| trap("asset not found"));
        let now = Int::from(time() as u64);

        let mut chunks = s.chunks.borrow_mut();

        let mut content_chunks = vec![];
        let mut previous_byte = 0;

        // TODO I am assuming the chunks are stored in order
        for chunk_id in arg.chunk_ids.iter() {
            let chunk = chunks.remove(chunk_id).expect("chunk not found");

            let content_length = chunk.content.len() as u64;

            content_chunks.push(ContentChunk {
                content: chunk.content,
                start_byte: previous_byte,
                end_byte: previous_byte + content_length - 1
            });

            previous_byte = previous_byte + content_length;
        }

        let sha256: [u8; 32] = match arg.sha256 {
            Some(bytes) => bytes
                .into_vec()
                .try_into()
                .unwrap_or_else(|_| trap("invalid SHA-256")),
            None => {
                let mut hasher = sha2::Sha256::new();
                for chunk in content_chunks.iter() {
                    hasher.update(&chunk.content);
                }
                hasher.finalize().into()
            }
        };

        let total_length: usize = content_chunks.iter().map(|c| c.content.len()).sum();
        let enc = AssetEncoding {
            modified: now,
            content_chunks,
            certified: false,
            total_length,
            sha256,
        };
        asset.encodings.insert(arg.content_encoding, enc);

        on_asset_change(&arg.key, asset);
    })
}

fn do_unset_asset_content(arg: UnsetAssetContentArguments) {
    STATE.with(|s| {
        let mut assets = s.assets.borrow_mut();
        let asset = assets
            .get_mut(&arg.key)
            .unwrap_or_else(|| trap("asset not found"));

        if asset.encodings.remove(&arg.content_encoding).is_some() {
            on_asset_change(&arg.key, asset);
        }
    })
}

fn do_delete_asset(arg: DeleteAssetArguments) {
    STATE.with(|s| {
        let mut assets = s.assets.borrow_mut();
        assets.remove(&arg.key);
    });
    delete_asset_hash(&arg.key);
}

fn do_clear() {
    STATE.with(|s| {
        s.assets.borrow_mut().clear();
        s.batches.borrow_mut().clear();
        s.chunks.borrow_mut().clear();
        *s.next_batch_id.borrow_mut() = Nat::from(1);
        *s.next_chunk_id.borrow_mut() = Nat::from(1);
    })
}

pub fn is_authorized() -> Result<(), String> {
    STATE.with(|s| {
        s.authorized
            .borrow()
            .contains(&caller())
            .then(|| ())
            .ok_or_else(|| "Caller is not authorized".to_string())
    })
}

fn on_asset_change(key: &str, asset: &mut Asset) {
    // If the most preferred encoding is present and certified,
    // there is nothing to do.
    for enc_name in ENCODING_CERTIFICATION_ORDER.iter() {
        if let Some(enc) = asset.encodings.get(*enc_name) {
            if enc.certified {
                return;
            } else {
                break;
            }
        }
    }

    if asset.encodings.is_empty() {
        delete_asset_hash(key);
        return;
    }

    // An encoding with a higher priority was added, let's certify it
    // instead.

    for enc in asset.encodings.values_mut() {
        enc.certified = false;
    }

    for enc_name in ENCODING_CERTIFICATION_ORDER.iter() {
        if let Some(enc) = asset.encodings.get_mut(*enc_name) {
            certify_asset(key.to_string(), &enc.sha256);
            enc.certified = true;
            return;
        }
    }

    // No known encodings found. Just pick the first one. The exact
    // order is hard to predict because we use a hash map. Should
    // almost never happen anyway.
    if let Some(enc) = asset.encodings.values_mut().next() {
        certify_asset(key.to_string(), &enc.sha256);
        enc.certified = true;
    }
}

fn certify_asset(key: Key, content_hash: &Hash) {
    ASSET_HASHES.with(|t| {
        let mut tree = t.borrow_mut();
        tree.insert(key, *content_hash);
        set_root_hash(&*tree);
    });
}

fn delete_asset_hash(key: &str) {
    ASSET_HASHES.with(|t| {
        let mut tree = t.borrow_mut();
        tree.delete(key.as_bytes());
        set_root_hash(&*tree);
    });
}

fn set_root_hash(tree: &AssetHashes) {
    use ic_certified_map::labeled_hash;
    let full_tree_hash = labeled_hash(b"http_assets", &tree.root_hash());
    set_certified_data(&full_tree_hash);
}

fn witness_to_header(witness: HashTree) -> HeaderField {
    use ic_certified_map::labeled;

    let hash_tree = labeled(b"http_assets", witness);
    let mut serializer = serde_cbor::ser::Serializer::new(vec![]);
    serializer.self_describe().unwrap();
    hash_tree.serialize(&mut serializer).unwrap();

    let certificate = data_certificate().unwrap_or_else(|| trap("no data certificate available"));

    (
        "IC-Certificate".to_string(),
        String::from("certificate=:")
            + &base64::encode(&certificate)
            + ":, tree=:"
            + &base64::encode(&serializer.into_inner())
            + ":",
    )
}

fn merge_hash_trees<'a>(lhs: HashTree<'a>, rhs: HashTree<'a>) -> HashTree<'a> {
    use HashTree::{Empty, Fork, Labeled, Leaf, Pruned};

    match (lhs, rhs) {
        (Pruned(l), Pruned(r)) => {
            if l != r {
                trap("merge_hash_trees: inconsistent hashes");
            }
            Pruned(l)
        }
        (Pruned(_), r) => r,
        (l, Pruned(_)) => l,
        (Fork(l), Fork(r)) => Fork(Box::new((
            merge_hash_trees(l.0, r.0),
            merge_hash_trees(l.1, r.1),
        ))),
        (Labeled(l_label, l), Labeled(r_label, r)) => {
            if l_label != r_label {
                trap("merge_hash_trees: inconsistent hash tree labels");
            }
            Labeled(l_label, Box::new(merge_hash_trees(*l, *r)))
        }
        (Empty, Empty) => Empty,
        (Leaf(l), Leaf(r)) => {
            if l != r {
                trap("merge_hash_trees: inconsistent leaves");
            }
            Leaf(l)
        }
        (_l, _r) => {
            trap("merge_hash_trees: inconsistent tree structure");
        }
    }
}

fn hash_bytes(bytes: &[u8]) -> Hash {
    let mut hash = sha2::Sha256::new();
    hash.update(bytes);
    hash.finalize().into()
}

pub fn init() {
    do_clear();
    STATE.with(|s| s.authorized.borrow_mut().push(caller()));
}

pub fn pre_upgrade() -> StableState {
    STATE.with(|s| StableState {
        authorized: s.authorized.take(),
        stable_assets: s.assets.take(),
    })
}

pub fn post_upgrade(stable_state: StableState) {
    do_clear();
    STATE.with(|s| {
        s.authorized.replace(stable_state.authorized);
        s.assets.replace(stable_state.stable_assets);

        for (asset_name, asset) in s.assets.borrow_mut().iter_mut() {
            for enc in asset.encodings.values_mut() {
                enc.certified = false;
            }
            on_asset_change(asset_name, asset);
        }
    });
}
