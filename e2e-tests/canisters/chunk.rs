use candid::Principal;
use ic_cdk::api::management_canister::main::{
    clear_chunk_store, create_canister, install_chunked_code, stored_chunks, upload_chunk,
    CanisterInstallMode, ChunkHash, ClearChunkStoreArgument, CreateCanisterArgument,
    InstallChunkedCodeArgument, StoredChunksArgument, UploadChunkArgument,
};
use ic_cdk::update;

#[update]
async fn call_create_canister() -> Principal {
    let arg = CreateCanisterArgument::default();
    let canister_id = create_canister(arg, 100_000_000_000u128)
        .await
        .unwrap()
        .0
        .canister_id;
    canister_id
}

#[update]
async fn call_upload_chunk(canister_id: Principal, chunk: Vec<u8>) -> Vec<u8> {
    let arg = UploadChunkArgument {
        canister_id,
        chunk: chunk.to_vec(),
    };
    let hash = upload_chunk(arg).await.unwrap().0.hash;
    hash
}

#[update]
async fn call_stored_chunks(canister_id: Principal) -> Vec<Vec<u8>> {
    let arg = StoredChunksArgument { canister_id };
    let hashes = stored_chunks(arg).await.unwrap().0;
    hashes.into_iter().map(|v| v.hash).collect()
}

#[update]
async fn call_clear_chunk_store(canister_id: Principal) {
    let arg = ClearChunkStoreArgument { canister_id };
    clear_chunk_store(arg).await.unwrap();
}

#[update]
async fn call_install_chunked_code(
    canister_id: Principal,
    chunk_hashes: Vec<Vec<u8>>,
    wasm_module_hash: Vec<u8>,
) {
    let chunk_hashes_list = chunk_hashes
        .into_iter()
        .map(|hash| ChunkHash { hash })
        .collect();
    let arg = InstallChunkedCodeArgument {
        mode: CanisterInstallMode::Install,
        target_canister: canister_id,
        storage_canister: None,
        chunk_hashes_list,
        wasm_module_hash,
        arg: vec![],
    };
    install_chunked_code(arg).await.unwrap();
}

fn main() {}
