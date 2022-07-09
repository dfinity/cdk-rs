use ic_cdk_macros::query;

#[query]
fn instruction_counter() -> u64 {
    ic_cdk::api::instruction_counter()
}

fn main() {}
