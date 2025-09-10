fn main() {
    // For the `macros` test
    prost_build::compile_protos(&["src/bin/macros/canister.proto"], &["src/"])
        .expect("Failed to compile protos");

    // For the `bindgen` test
    ic_cdk_bindgen::Config::new(
        "management_canister",
        "../ic-management-canister-types/tests/ic.did",
    )
    .static_callee(candid::Principal::management_canister())
    .generate();
    ic_cdk_bindgen::Config::new("bindgen_callee", "src/bin/bindgen_callee/callee.did")
        .dynamic_callee("ICP_CANISTER_ID:bindgen_callee")
        .generate();
}
