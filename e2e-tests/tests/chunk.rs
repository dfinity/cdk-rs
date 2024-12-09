use candid::Principal;
use ic_cdk_e2e_tests::cargo_build_canister;
use pocket_ic::common::rest::RawEffectivePrincipal;
use pocket_ic::{call_candid, PocketIc};
use sha2::Digest;

#[test]
fn test_chunk() {
    let pic = PocketIc::new();
    let wasm = cargo_build_canister("chunk");
    let canister_id = pic.create_canister();
    pic.add_cycles(canister_id, 100_000_000_000_000);
    pic.install_canister(canister_id, wasm, vec![], None);
    let (target_canister_id,): (Principal,) = call_candid(
        &pic,
        canister_id,
        RawEffectivePrincipal::None,
        "call_create_canister",
        (),
    )
    .expect("Error calling call_create_canister");

    let wasm_module = b"\x00asm\x01\x00\x00\x00".to_vec();
    let wasm_module_hash = sha2::Sha256::digest(&wasm_module).to_vec();
    let chunk1 = wasm_module[..4].to_vec();
    let chunk2 = wasm_module[4..].to_vec();
    let hash1_expected = sha2::Sha256::digest(&chunk1).to_vec();
    let hash2_expected = sha2::Sha256::digest(&chunk2).to_vec();

    let (hash1_return,): (Vec<u8>,) = call_candid(
        &pic,
        canister_id,
        RawEffectivePrincipal::None,
        "call_upload_chunk",
        (target_canister_id, chunk1.clone()),
    )
    .expect("Error calling call_upload_chunk");
    assert_eq!(&hash1_return, &hash1_expected);

    let () = call_candid(
        &pic,
        canister_id,
        RawEffectivePrincipal::None,
        "call_clear_chunk_store",
        (target_canister_id,),
    )
    .expect("Error calling call_clear_chunk_store");

    let (_hash1_return,): (Vec<u8>,) = call_candid(
        &pic,
        canister_id,
        RawEffectivePrincipal::None,
        "call_upload_chunk",
        (target_canister_id, chunk1),
    )
    .expect("Error calling call_upload_chunk");
    let (_hash2_return,): (Vec<u8>,) = call_candid(
        &pic,
        canister_id,
        RawEffectivePrincipal::None,
        "call_upload_chunk",
        (target_canister_id, chunk2),
    )
    .expect("Error calling call_upload_chunk");

    let (hashes,): (Vec<Vec<u8>>,) = call_candid(
        &pic,
        canister_id,
        RawEffectivePrincipal::None,
        "call_stored_chunks",
        (target_canister_id,),
    )
    .expect("Error calling call_stored_chunks");
    // the hashes returned are not guaranteed to be in order
    assert_eq!(hashes.len(), 2);
    assert!(hashes.contains(&hash1_expected));
    assert!(hashes.contains(&hash2_expected));

    let () = call_candid(
        &pic,
        canister_id,
        RawEffectivePrincipal::None,
        "call_install_chunked_code",
        (
            target_canister_id,
            // the order of the hashes matters
            vec![hash1_expected, hash2_expected],
            wasm_module_hash,
        ),
    )
    .expect("Error calling call_install_chunked_code");
}