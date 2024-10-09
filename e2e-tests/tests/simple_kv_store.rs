use ic_cdk_e2e_tests::cargo_build_canister;
use pocket_ic::common::rest::RawEffectivePrincipal;
use pocket_ic::{call_candid, query_candid, PocketIc};
use serde_bytes::ByteBuf;

/// Checks that a canister that uses [`ic_cdk::storage::stable_save`]
/// and [`ic_cdk::storage::stable_restore`] functions can keep its data
/// across upgrades.
#[test]
fn test_storage_roundtrip() {
    let pic = PocketIc::new();
    let wasm = cargo_build_canister("simple_kv_store");
    let canister_id = pic.create_canister();
    pic.add_cycles(canister_id, 2_000_000_000_000);
    pic.install_canister(canister_id, wasm.clone(), vec![], None);

    let () = call_candid(
        &pic,
        canister_id,
        RawEffectivePrincipal::None,
        "insert",
        (&"candid", &b"did"),
    )
    .expect("failed to insert 'candid'");

    pic.upgrade_canister(canister_id, wasm, vec![], None)
        .expect("failed to upgrade the simple_kv_store canister");

    let (result,): (Option<ByteBuf>,) =
        query_candid(&pic, canister_id, "lookup", (&"candid",)).expect("failed to lookup 'candid'");
    assert_eq!(result, Some(ByteBuf::from(b"did".to_vec())));
}
