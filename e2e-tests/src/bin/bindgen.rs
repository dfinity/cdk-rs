use ic_cdk::update;

// It's likely that not all the types/methods in the binding will be used.
// So it's a common practice to allow dead_code and unused_imports.
// The lint only affects this inner module.
#[allow(dead_code, unused_imports)]
mod management_canister {
    include!(concat!(env!("OUT_DIR"), "/management_canister.rs"));
}

#[update]
async fn call_management_canister() {
    // In modern IDE/editors like VSCode, you can often use "Go to Definition" or similar features
    // to quickly navigate to the generated bindings.
    let _rand = management_canister::raw_rand().await.unwrap();
}

#[allow(dead_code, unused_imports)]
mod bindgen_callee {
    include!(concat!(env!("OUT_DIR"), "/bindgen_callee.rs"));
}

#[update]
async fn call_bindgen_callee() {
    assert_eq!(bindgen_callee::add(&1, &2).await.unwrap(), 3);
}

fn main() {}
