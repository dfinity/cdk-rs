//! Codegen backend for [`ic_cdk_macros`](https://docs.rs/ic-cdk-macros).
//! The intended use of this crate is indirectly via [`#[ic_cdk_macros::import]`](https://docs.rs/ic-cdk-macros/*/ic_cdk_macros/attr.import.html),
//! but you can also use this in a build script to pregenerate the code.

#![deny(missing_debug_implementations, rust_2018_idioms, missing_docs)]

#[macro_use]
extern crate quote;
pub use candid;
pub use syn;

use candid::parser::types::Dec;
use candid::parser::types::{Binding, IDLType, PrimType};
use candid::types::internal::Label;
use candid::IDLProg;
use candid::Principal;
use num_bigint::BigUint;
use proc_macro2::Span;
use proc_macro2::TokenStream;
use std::collections::HashMap;
use std::collections::VecDeque;
use std::convert::TryInto;
use std::env;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::result::Result as StdResult;
use std::str::FromStr;
use syn::spanned::Spanned;
use syn::*;

mod error;
pub use error::ProcessingError;

/// Processes Candid declarations into Rust symbols.
#[derive(Debug, Clone)]
pub struct Processor {
    decls: HashMap<String, TokenStream>,
    bindings: HashMap<String, Signature>,
    record_id: usize,
    variant_id: usize,
}

impl Default for Processor {
    fn default() -> Self {
        Self {
            decls: initial_prim_map(),
            bindings: HashMap::new(),
            record_id: 0,
            variant_id: 0,
        }
    }
}

impl Processor {
    /// Constructs a `Processor` from an existing set of symbols.
    ///
    /// These symbols will override those in the Candid file, as well as built-in types like `Nat`.
    /// Buit-in type names draw from [`PrimType`], not the Candid language.
    ///
    /// # Rules
    ///
    /// - Symbols must be `struct`s, `enum`s, `type`s, `use`s, or `async fn`s.
    /// - All symbols must be `pub`.
    /// - `use`s must not be globs (`*`).
    /// - Nothing may be generic.
    /// - Functions may neither be `const` nor `extern`.
    /// - Functions should, but do not have to, end in `;` instead of `{}`.
    pub fn new(body: Vec<Item>) -> Result<Self> {
        let (predecls, bindings) = body_to_decls(body)?;
        let mut decls = initial_prim_map();
        decls.extend(predecls);
        Ok(Self {
            decls,
            bindings,
            record_id: 0,
            variant_id: 0,
        })
    }

    fn next_record(&mut self) -> usize {
        let ret = self.record_id;
        self.record_id += 1;
        ret
    }

    fn next_variant(&mut self) -> usize {
        let ret = self.variant_id;
        self.variant_id += 1;
        ret
    }

    /// Adds a [`Binding`] to the type declarations.
    ///
    /// These types will be attempted to be exported from the generated code,
    /// but in the case of name conflicts, actor functions will override them.
    /// In this case the type will be accessible from a module named `t`.
    pub fn add_decl(&mut self, decl: Binding) -> Result<()> {
        self.idl_to_rust(&decl.typ, Some(&decl.id))?;
        Ok(())
    }

    fn idl_to_rust(&mut self, ty: &IDLType, name: Option<&str>) -> Result<TokenStream> {
        if let Some(name) = name {
            if self.decls.contains_key(name) {
                let (ident, _) = str_to_ident(name);
                return Ok(quote!(#ident));
            }
        }
        let ret = match ty {
            IDLType::RecordT(fields) => {
                let name = name
                    .map(String::from)
                    .unwrap_or_else(|| format!("_R{}", self.next_record()));
                let name_ident = format_ident!("{}", name);
                let types = fields
                    .iter()
                    .map(|field| self.idl_to_rust(&field.typ, None))
                    .collect::<Result<Vec<_>>>()?;
                if fields
                    .iter()
                    .all(|field| matches!(field.label, Label::Unnamed(_)))
                {
                    let body = quote! {
                        #[derive(::std::fmt::Debug, ::std::clone::Clone, ::std::cmp::PartialEq, ::ic_cdk::export::serde::Deserialize, ::ic_cdk::export::candid::CandidType)]
                        #[serde(crate = "::ic_cdk::export::serde")]
                        pub struct #name_ident ( #( pub #types ),* ) ;
                    };
                    self.decls.insert(name, body);
                } else {
                    let (names, attrs) = fields
                        .iter()
                        .map(|field| label_to_ident(&field.label))
                        .map(|(name, id)| {
                            (
                                name,
                                id.map(|id| {
                                    let id = id.to_string();
                                    quote!(#[serde(rename = #id)])
                                }),
                            )
                        })
                        .unzip::<_, _, Vec<_>, Vec<_>>();
                    let body = quote! {
                        #[derive(::std::fmt::Debug, ::std::clone::Clone, ::std::cmp::PartialEq, ::ic_cdk::export::serde::Deserialize, ::ic_cdk::export::candid::CandidType)]
                        #[serde(crate = "::ic_cdk::export::serde")]
                        pub struct #name_ident {
                            #(
                                #attrs
                                pub #names : #types ,
                            )*
                        }
                    };
                    self.decls.insert(name, body);
                }
                return Ok(quote!(#name_ident));
            }
            IDLType::VariantT(fields) => {
                let name = name
                    .map(String::from)
                    .unwrap_or_else(|| format!("_V{}", self.next_variant()));
                let name_ident = format_ident!("{}", name);
                let variants = fields
                    .iter()
                    .map(|field| {
                        let (variant_name, idx) = label_to_ident(&field.label);
                        let attr = idx.map(|idx| {
                            let idx = idx.to_string();
                            quote!(#[serde(rename = #idx)])
                        });
                        Ok(match &field.typ {
                            IDLType::PrimT(PrimType::Null) => quote!(#variant_name),
                            IDLType::RecordT(fields) => {
                                let types = fields
                                    .iter()
                                    .map(|field| self.idl_to_rust(&field.typ, None))
                                    .collect::<Result<Vec<_>>>()?;
                                if fields
                                    .iter()
                                    .all(|field| matches!(field.label, Label::Unnamed(_)))
                                {
                                    quote!(#variant_name ( #( #types ),* ))
                                } else {
                                    let (names, attrs) = fields
                                        .iter()
                                        .map(|field| label_to_ident(&field.label))
                                        .map(|(name, id)| {
                                            (
                                                name,
                                                id.map(|id| {
                                                    let id = id.to_string();
                                                    quote!(#[serde(rename = #id)])
                                                }),
                                            )
                                        })
                                        .unzip::<_, _, Vec<_>, Vec<_>>();
                                    quote! {
                                        #attr
                                        #variant_name {
                                            #(
                                                #attrs
                                                #names : #types ,
                                             )*
                                        }
                                    }
                                }
                            }
                            r#type => {
                                let inner = self.idl_to_rust(r#type, None)?;
                                quote!(#variant_name ( #inner ))
                            }
                        })
                    })
                    .collect::<Result<Vec<_>>>()?;
                let body = quote! {
                    #[derive(::std::fmt::Debug, ::std::clone::Clone, ::std::cmp::PartialEq, ::ic_cdk::export::serde::Deserialize, ::ic_cdk::export::candid::CandidType)]
                    #[serde(crate = "::ic_cdk::export::serde")]
                    pub enum #name_ident {
                        #( #variants , )*
                    }
                };
                self.decls.insert(name, body);
                return Ok(quote!(#name_ident));
            }
            IDLType::VecT(inner) => {
                let inner = self.idl_to_rust(inner, None)?;
                quote!(::std::vec::Vec< #inner >)
            }
            IDLType::OptT(inner) => {
                let inner = self.idl_to_rust(inner, None)?;
                quote!(::std::option::Option< #inner >)
            }
            IDLType::PrimT(prim) => {
                let prim_ident = format_ident!("{}", prim_to_name(prim.clone()));
                quote!(#prim_ident)
            }
            IDLType::PrincipalT => quote!(Principal),
            IDLType::VarT(name) => {
                if !self.decls.contains_key(name) {
                    return Err(Error::new(
                        Span::call_site(),
                        format!("Undefined symbol {} in candid file", name),
                    ));
                }
                let ident = format_ident!("{}", name);
                quote!(#ident)
            }
            _ => {
                todo!("service")
            }
        };
        if let Some(name) = name {
            let ident = format_ident!("{}", name);
            self.decls
                .insert(name.to_owned(), quote!(type #ident = #ret ;));
        }
        Ok(ret)
    }

    /// Adds the bindings for the primary actor (service).
    ///
    /// These will be exported as free functions from the generated code.
    ///
    /// In the case of naming conflicts (including if this function is called twice), the first function wins.
    pub fn add_primary_actor(&mut self, actor: Vec<Binding>) -> Result<()> {
        for binding in actor {
            let func = if let IDLType::FuncT(func) = binding.typ {
                func
            } else {
                return Err(Error::new(
                    Span::call_site(),
                    format!("Actor binding {} is not a function", binding.id),
                ));
            };
            if self.bindings.contains_key(&binding.id) {
                continue;
            }
            let ident = format_ident!("{}", binding.id);
            let args = func
                .args
                .iter()
                .enumerate()
                .map(|(i, arg)| {
                    let ty = self.idl_to_rust(arg, None)?;
                    let ident = format_ident!("arg{}", i);
                    Ok(quote!(#ident : #ty))
                })
                .collect::<Result<Vec<_>>>()?;
            let rets = func
                .rets
                .iter()
                .map(|arg| self.idl_to_rust(arg, None))
                .collect::<Result<Vec<_>>>()?;
            self.bindings.insert(
                binding.id,
                parse2(quote!(async fn #ident ( #( #args ),* ) -> ( #( #rets ,)* ) )).unwrap(),
            );
        }
        Ok(())
    }

    /// Generates the final `TokenStream` from all the bindings, with the actor functions referencing the provided principal.
    /// This token stream is not encased in a surrounding module.
    pub fn generate(self, principal: Principal) -> Result<TokenStream> {
        let decls = self.decls.into_iter().map(|(_, decl)| decl);
        let type_mod = quote! {
            pub mod t {
                #( #decls )*
            }
            pub use t::*;
        };
        let funcs = self.bindings.into_iter().map(|(_, sig)| {
            let args = sig.inputs.iter().map(|arg| match arg {
                FnArg::Receiver(recv) => Err(Error::new(recv.span(), "Not valid here")),
                FnArg::Typed(typed) => {
                    match &*typed.pat {
                        Pat::Ident(i) => Ok(&i.ident),
                        other => Err(Error::new(other.span(), "Must be a plain parameter"))
                    }
                }
            }).collect::<Result<Vec<_>>>()?;
            let name = &sig.ident;
            let name_str = name.to_string();
            let principal_bytes = principal.as_slice();
            let expr = (args.len() == 1).then(|| quote!(.0));
            let qual = (args.len() == 1).then(|| quote!(::<_, (_,)>));
            let body = quote! {
                pub #sig {
                    ::ic_cdk::call #qual (
                        ::ic_cdk::export::candid::Principal::from_slice(&[ #( #principal_bytes ),* ][..]),
                        #name_str,
                        ( #( #args ,)* ),
                    ).await.unwrap() #expr
                }
            };
            Ok(body)
        }).collect::<Result<Vec<_>>>()?;
        Ok(quote! {
            #type_mod
            #( #funcs )*
        })
    }
}

macro_rules! prim_funcs {
    ($($name:ident : $type:ty),* $(,)?) => {
        fn prim_to_name(prim: PrimType) -> &'static str {
            match prim {
                $(PrimType::$name => stringify!($name)),*
            }
        }
        fn initial_prim_map() -> HashMap<String, TokenStream> {
            let mut map = HashMap::new();
            $(map.insert(stringify!($name).to_string(), quote!(pub type $name = $type;));)*
            map.insert(String::from("Principal"), quote!(pub type Principal = ::ic_cdk::export::Principal;));
            map
        }
    };
}

prim_funcs![
    Bool: bool,
    Float32: f32,
    Float64: f64,
    Int: ::ic_cdk::export::candid::Int,
    Int8: i8,
    Int16: i16,
    Int32: i32,
    Int64: i64,
    Nat: ::ic_cdk::export::candid::Nat,
    Nat8: u8,
    Nat16: u16,
    Nat32: u32,
    Nat64: u64,
    Null: (()),
    Reserved: ::ic_cdk::export::candid::Reserved,
    Empty: ::ic_cdk::export::candid::Empty,
    Text: String,
];

fn body_to_decls(
    body: Vec<Item>,
) -> Result<(HashMap<String, TokenStream>, HashMap<String, Signature>)> {
    let mut decls = HashMap::new();
    let mut bindings = HashMap::new();
    for item in body {
        match &item {
            Item::Enum(ItemEnum {
                ident,
                vis,
                generics,
                ..
            })
            | Item::Struct(ItemStruct {
                ident,
                vis,
                generics,
                ..
            })
            | Item::Type(ItemType {
                ident,
                vis,
                generics,
                ..
            }) => {
                if !matches!(vis, Visibility::Public(_)) {
                    return Err(Error::new(
                        generics.span(),
                        "Type definitions must be `pub`",
                    ));
                }
                if generics.params.is_empty() {
                    let attrs = if let Item::Type(_) = item {
                        TokenStream::default()
                    } else {
                        quote! {
                            #[derive(::ic_cdk::export::candid::CandidType, ::ic_cdk::export::serde::Deserialize)]
                            #[serde(crate = "::ic_cdk::export::serde")]
                        }
                    };
                    decls.insert(ident.to_string(), quote!(#attrs #item));
                } else {
                    return Err(Error::new(
                        generics.span(),
                        "Type definitions can't be generic",
                    ));
                }
            }
            Item::Use(r#use) => {
                if !matches!(r#use.vis, Visibility::Public(_)) {
                    return Err(Error::new(r#use.span(), "`use` statements must be `pub`"));
                }
                let mut deque = VecDeque::with_capacity(10);
                deque.push_back(&r#use.tree);
                while let Some(elem) = deque.pop_front() {
                    match elem {
                        UseTree::Glob(_) => {
                            return Err(Error::new(
                                elem.span(),
                                "`use` statements cannot contain globs",
                            ))
                        }
                        UseTree::Path(path) => deque.push_back(&*path.tree),
                        UseTree::Name(name) => {
                            decls.insert(name.ident.to_string(), quote!(#item));
                        }
                        UseTree::Rename(rename) => {
                            decls.insert(rename.ident.to_string(), quote!(#item));
                        }
                        UseTree::Group(group) => deque.extend(&group.items),
                    }
                }
            }
            Item::Fn(func) => {
                let sig = &func.sig;
                check_sig(&func.vis, sig)?;
                bindings.insert(sig.ident.to_string(), sig.clone());
            }
            Item::Verbatim(tokens) => {
                let span = tokens.span();
                match parse2::<ForeignItemFn>(tokens.clone()) {
                    Ok(func) => {
                        let sig = func.sig;
                        check_sig(&func.vis, &sig)?;
                        bindings.insert(sig.ident.to_string(), sig);
                    }
                    Err(_) => {
                        return Err(Error::new(
                            span,
                            "Not valid here: Expected a function, import, or type",
                        ))
                    }
                }
            }
            other => {
                return Err(Error::new(
                    other.span(),
                    "Not valid here: Expected a function, import, or type",
                ))
            }
        }
        fn check_sig(vis: &Visibility, sig: &Signature) -> Result<()> {
            if !matches!(vis, Visibility::Public(_)) {
                return Err(Error::new(vis.span(), "Functions must be `pub`"));
            }
            if sig.asyncness == None {
                return Err(Error::new(
                    sig.asyncness.span(),
                    "Functions must be `async`",
                ));
            }
            if !sig.generics.params.is_empty() {
                return Err(Error::new(
                    sig.generics.span(),
                    "Functions must not be generic",
                ));
            }
            if sig.abi.is_some() {
                return Err(Error::new(sig.abi.span(), "Functions must not be `extern`"));
            }
            if sig.constness.is_some() {
                return Err(Error::new(
                    sig.constness.span(),
                    "Functions must not be `const`",
                ));
            }
            if sig.variadic.is_some() {
                return Err(Error::new(sig.variadic.span(), "Unexpected token `...`"));
            }
            Ok(())
        }
    }
    Ok((decls, bindings))
}

fn label_to_ident(label: &Label) -> (Ident, Option<u32>) {
    match label {
        Label::Id(id) => (format_ident!("__{}", id), Some(*id)),
        Label::Named(name) => str_to_ident(name),
        Label::Unnamed(idx) => (format_ident!("_{}", idx), Some(*idx)),
    }
}

fn str_to_ident(name: &str) -> (Ident, Option<u32>) {
    if let Ok(ident) = parse_str::<Ident>(name) {
        (ident, None)
    } else {
        let hash = ident_hash(name);
        (format_ident!("__{}", hash), Some(hash))
    }
}

fn ident_hash(name: &str) -> u32 {
    assert!(
        name.len() <= u32::MAX as usize,
        "Symbol too long in Candid file! (Perhaps an unclosed quote?)"
    );
    let mut a = BigUint::from(0u8);
    let k = name.len() - 1;
    for (i, b) in name.bytes().enumerate() {
        a += BigUint::from(223u8).pow((k - i) as u32) * b;
    }
    (a % 2u64.pow(32)).try_into().unwrap()
}

/// Quick convenience function to take a Candid file and write the bindings to an output file.
///
/// The input file is relative to the manifest dir, and the output file is relative to `$OUT_DIR` if it exists, or the manifest dir if it doesn't.
///
/// # Examples
///
/// `build.rs`:
/// ```rust
/// fn main() {
///     ic_cdk_codegen::process_file("ic.did", "ic.rs", "aaaaa-aa".parse().unwrap()).unwrap();
/// }
/// ```
/// `lib.rs`:
/// ```rust,ignore
/// mod ic {
///     include!(concat!(env!("OUT_DIR"), "ic.rs"));
/// }
/// ```
pub fn process_file(
    in_file: impl AsRef<Path>,
    out_file: impl AsRef<Path>,
    principal: Principal,
) -> StdResult<(), ProcessingError> {
    let mut candid_file = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap());
    candid_file.extend(in_file.as_ref());
    let file = fs::read_to_string(candid_file)?;
    let prog = IDLProg::from_str(&file)?;
    let mut include_file = PathBuf::from(
        env::var_os("OUT_DIR").unwrap_or_else(|| env::var_os("CARGO_MANIFEST_DIR").unwrap()),
    );
    include_file.extend(out_file.as_ref());
    fs::write(include_file, process(prog, principal)?.to_string())?;
    Ok(())
}

/// Quick convenience function to take a Candid document and produce the bindings.
pub fn process(prog: IDLProg, principal: Principal) -> StdResult<TokenStream, ProcessingError> {
    let mut processor = Processor::default();
    for dec in prog.decs {
        if let Dec::TypD(decl) = dec {
            processor.add_decl(decl)?;
        }
    }
    if let Some(IDLType::ServT(actor)) = prog.actor {
        processor.add_primary_actor(actor)?;
    }
    let bindings = processor.generate(principal)?;
    Ok(bindings)
}
