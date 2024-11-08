fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    ic_cdk_bindgen::Builder::new("inter_mo")
        .generate_consumer()
        .unwrap();
}
