use ic_cdk_macros::query;

#[query]
fn instruction_counter() -> u64 {
    ic_cdk::api::call::performance_counter(0)
}

fn main() {}
