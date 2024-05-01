fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    ic_cdk_bindgen::Builder::new("profile_rs")
        .generate_consumer()
        .unwrap();
}
