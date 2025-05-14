#[ic_cdk::query]
fn print() {
    ic_cdk::print("Hello World");
}

ic_cdk::export_candid!();
