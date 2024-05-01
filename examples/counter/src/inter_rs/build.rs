fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    ic_cdk_bindgen::Builder::new("counter_mo")
        .generate_consumer()
        .unwrap();
}
