#[ic_cdk_macros::query]
fn print(_name: String) {
    ic_cdk::print("Hello World");
}

#[ic_cdk_macros::update]
fn test(name: String) -> (usize, String) {
    (name.len(), name)
}

ic_cdk_macros::export_candid!();
