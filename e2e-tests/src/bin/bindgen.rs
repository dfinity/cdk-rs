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

// Running this main function can refresh the build script execution.
fn main() {
    println!("{}", env!("OUT_DIR"));
}
