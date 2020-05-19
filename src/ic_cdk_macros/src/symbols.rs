use syn::Ident;

#[derive(Copy, Clone, Debug)]
pub(crate) struct Symbol(&'static str);

pub(crate) const CANDID_PATH: Symbol = Symbol("candid_path");
pub(crate) const CANISTER: Symbol = Symbol("canister");
pub(crate) const CANISTER_ID: Symbol = Symbol("canister_id");

impl PartialEq<Symbol> for Ident {
    fn eq(&self, other: &Symbol) -> bool {
        self == other.0
    }
}
