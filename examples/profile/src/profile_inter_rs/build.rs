fn main() {
    // println!("cargo:rerun-if-changed=build.rs");
    ic_cdk_bindgen::Builder::new("profile_rs")
        .candid_path("../profile_rs/profile.did")
        .generate_consumer()
        .unwrap();

    ic_cdk_bindgen::Builder::new("profile_inter_rs")
        .candid_path("profile.did")
        .out_dir("declarations")
        .generate_type()
        .unwrap();
}
