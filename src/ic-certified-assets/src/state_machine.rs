//! This module contains a pure implementation of the certified assets state machine.

// NB. This module should not depend on ic_cdk, it contains only pure state transition functions.
// All the environment (time, certificates, etc.) is passed to the state transition functions
// as formal arguments.  This approach makes it very easy to test the state machine.

use crate::{rc_bytes::RcBytes, types::*, url_decode::url_decode};
use candid::{CandidType, Deserialize, Func, Int, Nat, Principal};
use ic_certified_map::{AsHashTree, Hash, HashTree, RbTree};
use num_traits::ToPrimitive;
use serde::Serialize;
use serde_bytes::ByteBuf;
use sha2::Digest;
use std::collections::HashMap;
use std::convert::TryInto;

/// The amount of time a batch is kept alive. Modifying the batch
/// delays the expiry further.
pub const BATCH_EXPIRY_NANOS: u64 = 300_000_000_000;

/// The order in which we pick encodings for certification.
const ENCODING_CERTIFICATION_ORDER: &[&str] = &["identity", "gzip", "compress", "deflate", "br"];

/// The file to serve if the requested file wasn't found.
const INDEX_FILE: &str = "/index.html";

type AssetHashes = RbTree<Key, Hash>;
type Timestamp = Int;

#[derive(Default, Clone, Debug, CandidType, Deserialize)]
pub struct AssetEncoding {
    pub modified: Timestamp,
    pub content_chunks: Vec<RcBytes>,
    pub total_length: usize,
    pub certified: bool,
    pub sha256: [u8; 32],
}

#[derive(Default, Clone, Debug, CandidType, Deserialize)]
pub struct Asset {
    pub content_type: String,
    pub encodings: HashMap<String, AssetEncoding>,
    pub max_age: Option<u64>,
    pub headers: Option<HashMap<String, String>>,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct EncodedAsset {
    pub content: RcBytes,
    pub content_type: String,
    pub content_encoding: String,
    pub total_length: Nat,
    pub sha256: Option<ByteBuf>,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct AssetDetails {
    pub key: String,
    pub content_type: String,
    pub encodings: Vec<AssetEncodingDetails>,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct AssetEncodingDetails {
    pub content_encoding: String,
    pub sha256: Option<ByteBuf>,
    pub length: Nat,
    pub modified: Timestamp,
}

pub struct Chunk {
    pub batch_id: BatchId,
    pub content: RcBytes,
}

pub struct Batch {
    pub expires_at: Timestamp,
}

#[derive(Default)]
pub struct State {
    assets: HashMap<Key, Asset>,

    chunks: HashMap<ChunkId, Chunk>,
    next_chunk_id: ChunkId,

    batches: HashMap<BatchId, Batch>,
    next_batch_id: BatchId,

    authorized: Vec<Principal>,

    asset_hashes: AssetHashes,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct StableState {
    authorized: Vec<Principal>,
    stable_assets: HashMap<String, Asset>,
}

impl State {
    pub fn authorize_unconditionally(&mut self, principal: Principal) {
        if !self.is_authorized(&principal) {
            self.authorized.push(principal);
        }
    }

    pub fn authorize(&mut self, caller: &Principal, other: Principal) -> Result<(), String> {
        if !self.is_authorized(caller) {
            return Err("the caller is not authorized".to_string());
        }
        self.authorize_unconditionally(other);
        Ok(())
    }

    pub fn root_hash(&self) -> Hash {
        use ic_certified_map::labeled_hash;
        labeled_hash(b"http_assets", &self.asset_hashes.root_hash())
    }

    pub fn create_asset(&mut self, arg: CreateAssetArguments) -> Result<(), String> {
        if let Some(asset) = self.assets.get(&arg.key) {
            if asset.content_type != arg.content_type {
                return Err("create_asset: content type mismatch".to_string());
            }
        } else {
            self.assets.insert(
                arg.key,
                Asset {
                    content_type: arg.content_type,
                    encodings: HashMap::new(),
                    max_age: arg.max_age,
                    headers: arg.headers,
                },
            );
        }
        Ok(())
    }

    pub fn set_asset_content(
        &mut self,
        arg: SetAssetContentArguments,
        now: u64,
    ) -> Result<(), String> {
        if arg.chunk_ids.is_empty() {
            return Err("encoding must have at least one chunk".to_string());
        }

        let asset = self
            .assets
            .get_mut(&arg.key)
            .ok_or_else(|| "asset not found".to_string())?;

        let now = Int::from(now);

        let mut content_chunks = vec![];
        for chunk_id in arg.chunk_ids.iter() {
            let chunk = self.chunks.remove(chunk_id).expect("chunk not found");
            content_chunks.push(chunk.content);
        }

        let sha256: [u8; 32] = match arg.sha256 {
            Some(bytes) => bytes
                .into_vec()
                .try_into()
                .map_err(|_| "invalid SHA-256".to_string())?,
            None => {
                let mut hasher = sha2::Sha256::new();
                for chunk in content_chunks.iter() {
                    hasher.update(chunk);
                }
                hasher.finalize().into()
            }
        };

        let total_length: usize = content_chunks.iter().map(|c| c.len()).sum();
        let enc = AssetEncoding {
            modified: now,
            content_chunks,
            certified: false,
            total_length,
            sha256,
        };
        asset.encodings.insert(arg.content_encoding, enc);

        on_asset_change(&mut self.asset_hashes, &arg.key, asset);

        Ok(())
    }

    pub fn unset_asset_content(&mut self, arg: UnsetAssetContentArguments) -> Result<(), String> {
        let asset = self
            .assets
            .get_mut(&arg.key)
            .ok_or_else(|| "asset not found".to_string())?;

        if asset.encodings.remove(&arg.content_encoding).is_some() {
            on_asset_change(&mut self.asset_hashes, &arg.key, asset);
        }

        Ok(())
    }

    pub fn delete_asset(&mut self, arg: DeleteAssetArguments) {
        self.assets.remove(&arg.key);
        self.asset_hashes.delete(arg.key.as_bytes());
    }

    pub fn clear(&mut self) {
        self.assets.clear();
        self.batches.clear();
        self.chunks.clear();
        self.next_batch_id = Nat::from(1);
        self.next_chunk_id = Nat::from(1);
    }

    pub fn is_authorized(&self, principal: &Principal) -> bool {
        self.authorized.contains(principal)
    }

    pub fn retrieve(&self, key: &Key) -> Result<RcBytes, String> {
        let asset = self
            .assets
            .get(key)
            .ok_or_else(|| "asset not found".to_string())?;

        let id_enc = asset
            .encodings
            .get("identity")
            .ok_or_else(|| "no identity encoding".to_string())?;

        if id_enc.content_chunks.len() > 1 {
            return Err("Asset too large. Use get() and get_chunk() instead.".to_string());
        }

        Ok(id_enc.content_chunks[0].clone())
    }

    pub fn store(&mut self, arg: StoreArg, time: u64) -> Result<(), String> {
        let asset = self.assets.entry(arg.key.clone()).or_default();
        asset.content_type = arg.content_type;

        let hash = sha2::Sha256::digest(&arg.content).into();
        if let Some(provided_hash) = arg.sha256 {
            if hash != provided_hash.as_ref() {
                return Err("sha256 mismatch".to_string());
            }
        }

        let encoding = asset.encodings.entry(arg.content_encoding).or_default();
        encoding.total_length = arg.content.len();
        encoding.content_chunks = vec![RcBytes::from(arg.content)];
        encoding.modified = Int::from(time);
        encoding.sha256 = hash;

        on_asset_change(&mut self.asset_hashes, &arg.key, asset);
        Ok(())
    }

    pub fn create_batch(&mut self, now: u64) -> BatchId {
        let batch_id = self.next_batch_id.clone();
        self.next_batch_id += 1;

        self.batches.insert(
            batch_id.clone(),
            Batch {
                expires_at: Int::from(now + BATCH_EXPIRY_NANOS),
            },
        );
        self.chunks.retain(|_, c| {
            self.batches
                .get(&c.batch_id)
                .map(|b| b.expires_at > now)
                .unwrap_or(false)
        });
        self.batches.retain(|_, b| b.expires_at > now);

        batch_id
    }

    pub fn create_chunk(&mut self, arg: CreateChunkArg, now: u64) -> Result<ChunkId, String> {
        let mut batch = self
            .batches
            .get_mut(&arg.batch_id)
            .ok_or_else(|| "batch not found".to_string())?;

        batch.expires_at = Int::from(now + BATCH_EXPIRY_NANOS);

        let chunk_id = self.next_chunk_id.clone();
        self.next_chunk_id += 1;

        self.chunks.insert(
            chunk_id.clone(),
            Chunk {
                batch_id: arg.batch_id,
                content: RcBytes::from(arg.content),
            },
        );

        Ok(chunk_id)
    }

    pub fn commit_batch(&mut self, arg: CommitBatchArguments, now: u64) -> Result<(), String> {
        let batch_id = arg.batch_id;
        for op in arg.operations {
            match op {
                BatchOperation::CreateAsset(arg) => self.create_asset(arg)?,
                BatchOperation::SetAssetContent(arg) => self.set_asset_content(arg, now)?,
                BatchOperation::UnsetAssetContent(arg) => self.unset_asset_content(arg)?,
                BatchOperation::DeleteAsset(arg) => self.delete_asset(arg),
                BatchOperation::Clear(_) => self.clear(),
            }
        }
        self.batches.remove(&batch_id);
        Ok(())
    }

    pub fn list_assets(&self) -> Vec<AssetDetails> {
        self.assets
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
    }

    pub fn get(&self, arg: GetArg) -> Result<EncodedAsset, String> {
        let asset = self
            .assets
            .get(&arg.key)
            .ok_or_else(|| "asset not found".to_string())?;

        for enc in arg.accept_encodings.iter() {
            if let Some(asset_enc) = asset.encodings.get(enc) {
                return Ok(EncodedAsset {
                    content: asset_enc.content_chunks[0].clone(),
                    content_type: asset.content_type.clone(),
                    content_encoding: enc.clone(),
                    total_length: Nat::from(asset_enc.total_length as u64),
                    sha256: Some(ByteBuf::from(asset_enc.sha256)),
                });
            }
        }
        Err("no such encoding".to_string())
    }

    pub fn get_chunk(&self, arg: GetChunkArg) -> Result<RcBytes, String> {
        let asset = self
            .assets
            .get(&arg.key)
            .ok_or_else(|| "asset not found".to_string())?;

        let enc = asset
            .encodings
            .get(&arg.content_encoding)
            .ok_or_else(|| "no such encoding".to_string())?;

        if let Some(expected_hash) = arg.sha256 {
            if expected_hash != enc.sha256 {
                return Err("sha256 mismatch".to_string());
            }
        }
        if arg.index >= enc.content_chunks.len() {
            return Err("chunk index out of bounds".to_string());
        }
        let index: usize = arg.index.0.to_usize().unwrap();

        Ok(enc.content_chunks[index].clone())
    }

    fn build_http_response(
        &self,
        certificate: &[u8],
        path: &str,
        encodings: Vec<String>,
        index: usize,
        callback: Func,
        etags: Vec<Hash>,
    ) -> HttpResponse {
        let index_redirect_certificate = if self.asset_hashes.get(path.as_bytes()).is_none()
            && self.asset_hashes.get(INDEX_FILE.as_bytes()).is_some()
        {
            let absence_proof = self.asset_hashes.witness(path.as_bytes());
            let index_proof = self.asset_hashes.witness(INDEX_FILE.as_bytes());
            let combined_proof = merge_hash_trees(absence_proof, index_proof);
            Some(witness_to_header(combined_proof, certificate))
        } else {
            None
        };

        if let Some(certificate_header) = index_redirect_certificate {
            if let Some(asset) = self.assets.get(INDEX_FILE) {
                for enc_name in encodings.iter() {
                    if let Some(enc) = asset.encodings.get(enc_name) {
                        if enc.certified {
                            return build_ok(
                                asset,
                                enc_name,
                                enc,
                                INDEX_FILE,
                                index,
                                Some(certificate_header),
                                callback,
                                etags,
                            );
                        }
                    }
                }
            }
        }

        let certificate_header =
            witness_to_header(self.asset_hashes.witness(path.as_bytes()), certificate);

        if let Some(asset) = self.assets.get(path) {
            for enc_name in encodings.iter() {
                if let Some(enc) = asset.encodings.get(enc_name) {
                    if enc.certified {
                        return build_ok(
                            asset,
                            enc_name,
                            enc,
                            path,
                            index,
                            Some(certificate_header),
                            callback,
                            etags,
                        );
                    } else {
                        // Find if identity is certified, if it's not.
                        if let Some(id_enc) = asset.encodings.get("identity") {
                            if id_enc.certified {
                                return build_ok(
                                    asset,
                                    enc_name,
                                    enc,
                                    path,
                                    index,
                                    Some(certificate_header),
                                    callback,
                                    etags,
                                );
                            }
                        }
                    }
                }
            }
        }

        build_404(certificate_header)
    }

    pub fn http_request(
        &self,
        req: HttpRequest,
        certificate: &[u8],
        callback: Func,
    ) -> HttpResponse {
        let mut encodings = vec![];
        let mut etags = Vec::new();
        for (name, value) in req.headers.iter() {
            if name.eq_ignore_ascii_case("Accept-Encoding") {
                for v in value.split(',') {
                    encodings.push(v.trim().to_string());
                }
            }
            if name.eq_ignore_ascii_case("Host") {
                if let Some(replacement_url) = redirect_to_url(value, &req.url) {
                    return HttpResponse {
                        status_code: 308,
                        headers: vec![("Location".to_string(), replacement_url)],
                        body: RcBytes::from(ByteBuf::default()),
                        streaming_strategy: None,
                    };
                }
            }
            if name.eq_ignore_ascii_case("If-None-Match") {
                match decode_etag_seq(value) {
                    Ok(decoded_etags) => {
                        etags = decoded_etags;
                    }
                    Err(err) => {
                        return HttpResponse {
                            status_code: 400,
                            headers: vec![],
                            body: RcBytes::from(ByteBuf::from(format!(
                                "Invalid {} header value: {}",
                                name, err
                            ))),
                            streaming_strategy: None,
                        };
                    }
                }
            }
        }
        encodings.push("identity".to_string());

        let path = match req.url.find('?') {
            Some(i) => &req.url[..i],
            None => &req.url[..],
        };

        match url_decode(path) {
            Ok(path) => self.build_http_response(certificate, &path, encodings, 0, callback, etags),
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
    }

    pub fn http_request_streaming_callback(
        &self,
        StreamingCallbackToken {
            key,
            content_encoding,
            index,
            sha256,
        }: StreamingCallbackToken,
    ) -> Result<StreamingCallbackHttpResponse, String> {
        let asset = self
            .assets
            .get(&key)
            .ok_or_else(|| "Invalid token on streaming: key not found.".to_string())?;
        let enc = asset
            .encodings
            .get(&content_encoding)
            .ok_or_else(|| "Invalid token on streaming: encoding not found.".to_string())?;

        if let Some(expected_hash) = sha256 {
            if expected_hash != enc.sha256 {
                return Err("sha256 mismatch".to_string());
            }
        }

        // MAX is good enough. This means a chunk would be above 64-bits, which is impossible...
        let chunk_index = index.0.to_usize().unwrap_or(usize::MAX);

        Ok(StreamingCallbackHttpResponse {
            body: enc.content_chunks[chunk_index].clone(),
            token: create_token(asset, &content_encoding, enc, &key, chunk_index),
        })
    }
}

impl From<State> for StableState {
    fn from(state: State) -> Self {
        Self {
            authorized: state.authorized,
            stable_assets: state.assets,
        }
    }
}

impl From<StableState> for State {
    fn from(stable_state: StableState) -> Self {
        let mut state = Self {
            authorized: stable_state.authorized,
            assets: stable_state.stable_assets,
            ..Self::default()
        };

        for (asset_name, asset) in state.assets.iter_mut() {
            for enc in asset.encodings.values_mut() {
                enc.certified = false;
            }
            on_asset_change(&mut state.asset_hashes, asset_name, asset);
        }
        state
    }
}

fn decode_etag_seq(value: &str) -> Result<Vec<Hash>, String> {
    // Hex-encoded 32-byte hash + 2 quotes
    const EXPECTED_ETAG_LEN: usize = 66;
    let mut etags = Vec::with_capacity(1);
    for etag in value.split(',') {
        let etag = etag.trim();
        if etag.len() != EXPECTED_ETAG_LEN {
            return Err(format!(
                "invalid length of component {}: expected {}, got {}",
                etag,
                EXPECTED_ETAG_LEN,
                etag.len()
            ));
        }
        if !etag.starts_with('"') {
            return Err(format!("missing first quote of component {}", etag));
        }
        if !etag.ends_with('"') {
            return Err(format!("missing final quote of component {}", etag));
        }
        let mut hash = Hash::default();
        match hex::decode_to_slice(&etag[1..EXPECTED_ETAG_LEN - 1], &mut hash) {
            Ok(()) => {
                etags.push(hash);
            }
            Err(e) => return Err(format!("invalid hex of component {}: {}", etag, e)),
        }
    }
    Ok(etags)
}

#[test]
fn test_decode_seq() {
    for (value, expected) in [
        (
            r#""0000000000000000000000000000000000000000000000000000000000000000""#,
            vec![[0u8; 32]],
        ),
        (
            r#""0000000000000000000000000000000000000000000000000000000000000000", "1111111111111111111111111111111111111111111111111111111111111111""#,
            vec![[0u8; 32], [17u8; 32]],
        ),
    ] {
        let decoded = decode_etag_seq(value)
            .unwrap_or_else(|e| panic!("failed to parse good ETag value {}: {}", value, e));
        assert_eq!(decoded, expected);
    }

    for value in [
        r#""00000000000000000000000000000000""#,
        r#"0000000000000000000000000000000000000000000000000000000000000000"#,
        r#""0000000000000000000000000000000000000000000000000000000000000000" "1111111111111111111111111111111111111111111111111111111111111111""#,
        r#"0000000000000000000000000000000000000000000000000000000000000000 1111111111111111111111111111111111111111111111111111111111111111"#,
    ] {
        let result = decode_etag_seq(value);
        assert!(
            result.is_err(),
            "should have failed to parse invalid ETag value {}, got: {:?}",
            value,
            result
        );
    }
}

fn on_asset_change(asset_hashes: &mut AssetHashes, key: &str, asset: &mut Asset) {
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
        asset_hashes.delete(key.as_bytes());
        return;
    }

    // An encoding with a higher priority was added, let's certify it
    // instead.

    for enc in asset.encodings.values_mut() {
        enc.certified = false;
    }

    for enc_name in ENCODING_CERTIFICATION_ORDER.iter() {
        if let Some(enc) = asset.encodings.get_mut(*enc_name) {
            asset_hashes.insert(key.to_string(), enc.sha256);
            enc.certified = true;
            return;
        }
    }

    // No known encodings found. Just pick the first one. The exact
    // order is hard to predict because we use a hash map. Should
    // almost never happen anyway.
    if let Some(enc) = asset.encodings.values_mut().next() {
        asset_hashes.insert(key.to_string(), enc.sha256);
        enc.certified = true;
    }
}

fn witness_to_header(witness: HashTree, certificate: &[u8]) -> HeaderField {
    use ic_certified_map::labeled;

    let hash_tree = labeled(b"http_assets", witness);
    let mut serializer = serde_cbor::ser::Serializer::new(vec![]);
    serializer.self_describe().unwrap();
    hash_tree.serialize(&mut serializer).unwrap();

    (
        "IC-Certificate".to_string(),
        String::from("certificate=:")
            + &base64::encode(certificate)
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
                panic!("merge_hash_trees: inconsistent hashes");
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
                panic!("merge_hash_trees: inconsistent hash tree labels");
            }
            Labeled(l_label, Box::new(merge_hash_trees(*l, *r)))
        }
        (Empty, Empty) => Empty,
        (Leaf(l), Leaf(r)) => {
            if l != r {
                panic!("merge_hash_trees: inconsistent leaves");
            }
            Leaf(l)
        }
        (_l, _r) => {
            panic!("merge_hash_trees: inconsistent tree structure");
        }
    }
}

fn create_token(
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

#[allow(clippy::too_many_arguments)]
fn build_ok(
    asset: &Asset,
    enc_name: &str,
    enc: &AssetEncoding,
    key: &str,
    chunk_index: usize,
    certificate_header: Option<HeaderField>,
    callback: Func,
    etags: Vec<Hash>,
) -> HttpResponse {
    let mut headers = vec![("Content-Type".to_string(), asset.content_type.to_string())];
    if enc_name != "identity" {
        headers.push(("Content-Encoding".to_string(), enc_name.to_string()));
    }
    if let Some(head) = certificate_header {
        headers.push(head);
    }
    if let Some(max_age) = asset.max_age {
        headers.push(("Cache-Control".to_string(), format!("max-age={}", max_age)));
    }
    if let Some(arg_headers) = asset.headers.as_ref() {
        for (k, v) in arg_headers {
            headers.push((k.to_owned(), v.to_owned()));
        }
    }

    let streaming_strategy = create_token(asset, enc_name, enc, key, chunk_index)
        .map(|token| StreamingStrategy::Callback { callback, token });

    let (status_code, body) = if etags.contains(&enc.sha256) {
        (304, RcBytes::default())
    } else {
        headers.push((
            "ETag".to_string(),
            format!("\"{}\"", hex::encode(enc.sha256)),
        ));
        (200, enc.content_chunks[chunk_index].clone())
    };

    HttpResponse {
        status_code,
        headers,
        body,
        streaming_strategy,
    }
}

fn build_404(certificate_header: HeaderField) -> HttpResponse {
    HttpResponse {
        status_code: 404,
        headers: vec![certificate_header],
        body: RcBytes::from(ByteBuf::from("not found")),
        streaming_strategy: None,
    }
}

fn redirect_to_url(host: &str, url: &str) -> Option<String> {
    if let Some(host) = host.split(':').next() {
        let host = host.trim();
        if host == "raw.ic0.app" {
            return Some(format!("https://ic0.app{}", url));
        } else if let Some(base) = host.strip_suffix(".raw.ic0.app") {
            return Some(format!("https://{}.ic0.app{}", base, url));
        }
    }
    None
}
