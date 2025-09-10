fn main() {
    // For the `macros` test
    prost_build::compile_protos(&["src/bin/macros/canister.proto"], &["src/"])
        .expect("Failed to compile protos");

    // For the `bindgen` test
    ic_cdk_bindgen::static_callee(
        "management_canister",
        "../ic-management-canister-types/tests/ic.did",
        candid::Principal::management_canister(),
    );
}
