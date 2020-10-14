use candid::{CandidType, Deserialize};

#[ic_cdk_macros::query]
fn print() {
    ic_cdk::print("Hello World");
}

#[ic_cdk_macros::update(name = "🐂")]
fn test(name: String) -> (usize, String) {
    (name.len(), name)
}

#[derive(CandidType, Deserialize)]
struct List { head: i8, tail: Option<Box<List>> }

#[derive(CandidType, Deserialize)]
enum A { A1(u16), A2(List), A3(String, candid::Principal) }

#[ic_cdk_macros::query]
fn id_struct(a:List) -> List { a }

#[ic_cdk_macros::update]
fn id_variant(a:A) -> A { a }

ic_cdk_macros::export_candid!();

