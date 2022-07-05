//! This module defines types shared by the certified assets state machine and the canister
//! endpoints.
use std::collections::HashMap;

use crate::rc_bytes::RcBytes;
use candid::{CandidType, Deserialize, Func, Nat};
use serde_bytes::ByteBuf;

pub type BatchId = Nat;
pub type ChunkId = Nat;
pub type Key = String;

// IDL Types

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct CreateAssetArguments {
    pub key: Key,
    pub content_type: String,
    pub max_age: Option<u64>,
    pub headers: Option<HashMap<String, String>>,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct SetAssetContentArguments {
    pub key: Key,
    pub content_encoding: String,
    pub chunk_ids: Vec<ChunkId>,
    pub sha256: Option<ByteBuf>,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct UnsetAssetContentArguments {
    pub key: Key,
    pub content_encoding: String,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct DeleteAssetArguments {
    pub key: Key,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct ClearArguments {}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub enum BatchOperation {
    CreateAsset(CreateAssetArguments),
    SetAssetContent(SetAssetContentArguments),
    UnsetAssetContent(UnsetAssetContentArguments),
    DeleteAsset(DeleteAssetArguments),
    Clear(ClearArguments),
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct CommitBatchArguments {
    pub batch_id: BatchId,
    pub operations: Vec<BatchOperation>,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct StoreArg {
    pub key: Key,
    pub content_type: String,
    pub content_encoding: String,
    pub content: ByteBuf,
    pub sha256: Option<ByteBuf>,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct GetArg {
    pub key: Key,
    pub accept_encodings: Vec<String>,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct GetChunkArg {
    pub key: Key,
    pub content_encoding: String,
    pub index: Nat,
    pub sha256: Option<ByteBuf>,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct GetChunkResponse {
    pub content: RcBytes,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct CreateBatchResponse {
    pub batch_id: BatchId,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct CreateChunkArg {
    pub batch_id: BatchId,
    pub content: ByteBuf,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct CreateChunkResponse {
    pub chunk_id: ChunkId,
}
// HTTP interface

pub type HeaderField = (String, String);

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct HttpRequest {
    pub method: String,
    pub url: String,
    pub headers: Vec<(String, String)>,
    pub body: ByteBuf,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct HttpResponse {
    pub status_code: u16,
    pub headers: Vec<HeaderField>,
    pub body: RcBytes,
    pub streaming_strategy: Option<StreamingStrategy>,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct StreamingCallbackToken {
    pub key: String,
    pub content_encoding: String,
    pub index: Nat,
    // We don't care about the sha, we just want to be backward compatible.
    pub sha256: Option<ByteBuf>,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub enum StreamingStrategy {
    Callback {
        callback: Func,
        token: StreamingCallbackToken,
    },
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct StreamingCallbackHttpResponse {
    pub body: RcBytes,
    pub token: Option<StreamingCallbackToken>,
}
