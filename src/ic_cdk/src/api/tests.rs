use super::*;

#[test]
fn canister_id_parse_str() {
    let canister_id_bytes = [0xAB, 0xCD, 0x01];
    let canister_id = CanisterId::from(Vec::from(canister_id_bytes));
    let canister_id_str_expected = "em77e-bvlzu-aq";
    assert_eq!(format!("{}", canister_id), canister_id_str_expected);
    assert_eq!(
        canister_id,
        CanisterId::from_str(canister_id_str_expected).unwrap()
    );

    // Check that a long CanisterId in string form is also parsed successfully
    // and that it doesn't match the short CanisterId parsed earlier
    assert_ne!(
        CanisterId::from_str("cxeji-wacaa-aaaaa-aaaaa-aaaaa-aaaaa-aaaaa-q").unwrap(),
        canister_id
    );
}
