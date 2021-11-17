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
use proc_macro2::Span;
use proc_macro2::TokenStream;
use std::borrow::Borrow;
use std::collections::HashMap;
use std::collections::VecDeque;
use std::fmt::Arguments;
use std::fs;
use std::hash::Hash;
use std::path::Path;
use std::path::PathBuf;
use std::result::Result as StdResult;
use std::str::FromStr;
use std::{env, iter};
use syn::spanned::Spanned;
use syn::*;

mod error;
pub use error::ProcessingError;

/// Processes Candid declarations into Rust symbols.
#[derive(Debug, Clone)]
pub struct Processor {
    /// Map of Candid name (not necessarily a valid ident, use `str_to_ident`) to full declaration in code
    decls: HashMap<String, TokenStream>,
    bindings: HashMap<String, Signature>,
    /// List of types already being defined in the current `idl_to_rust` pass; used to resolve recursion
    current_pass: Vec<String>,
    /// Map of unresolved `IDLType::VarT`s to the type that contains them, used to handle mutual recursion
    unresolved: HashMap<String, Vec<String>>,
    /// Inverse of `unresolved`
    unresolved_containers: HashMap<String, Vec<String>>,
}

impl Default for Processor {
    fn default() -> Self {
        Self {
            decls: initial_prim_map(),
            bindings: HashMap::new(),
            current_pass: Vec::new(),
            unresolved: HashMap::new(),
            unresolved_containers: HashMap::new(),
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
    /// - Functions should, but do not have to, end in `;` instead of `{}`. The former is more descriptive,
    ///   but `rustfmt` messes with it.
    pub fn new(body: Vec<Item>) -> Result<Self> {
        let (predecls, bindings) = body_to_decls(body)?;
        let mut decls = initial_prim_map();
        decls.extend(predecls); // RHS overrides LHS
        Ok(Self {
            decls,
            bindings,
            current_pass: Vec::new(),
            unresolved: HashMap::new(),
            unresolved_containers: HashMap::new(),
        })
    }

    /// Adds a [`Binding`] to the type declarations.
    ///
    /// These types will be attempted to be exported from the generated code,
    /// but in the case of name conflicts, actor functions will override them.
    /// In this case the type will be accessible from a module named `t`.
    pub fn add_decl(&mut self, decl: Binding) -> Result<()> {
        self.idl_to_rust(&decl.typ, Some(&decl.id), format_args!("{}", decl.id))?;
        self.current_pass.clear();
        Ok(())
    }

    /// Takes an IDL type and returns Rust code to mention that type. If `name` is `Some` then this IDL type is the root RHS of a `type <name> =` IDL expression.
    ///
    /// `gen_prefix` is the prefix, not including a leading `_`, for any inline types that need generated names. It is `Arguments` instead of a string type to avoid a whole lot of allocation.
    fn idl_to_rust(
        &mut self,
        ty: &IDLType,
        name: Option<&str>,
        gen_prefix: Arguments<'_>,
    ) -> Result<TokenStream> {
        if let Some(name) = name {
            // this type was already defined, all that's left is to name it
            if self.decls.contains_key(name) {
                let ident = str_to_ident(name);
                return Ok(quote!(#ident));
            }
            // given `b = a; c = b; a = c;`:
            // detect that we are finally defining `a`
            if let Some(previous) = self.unresolved.get(name) {
                // push `b` and `c` into the current pass, so they will get boxed when referred to
                // this could possibly result in a false positive for boxing, if the type is indirected through a function
                // but it doesn't really matter as long as it's pared down to the minimum
                self.current_pass.extend(previous.iter().cloned());
                // and then clear
                for s in previous {
                    let coll = self.unresolved_containers.get_mut(s).unwrap();
                    let idx = coll.iter().position(|x| x == name).unwrap();
                    coll.remove(idx);
                }
                self.unresolved.remove(name);
            }
            self.current_pass.push(name.to_string());
        }
        let ret = match ty {
            IDLType::RecordT(fields) => {
                let name = name
                    .map(String::from)
                    .unwrap_or_else(|| format!("_{}", gen_prefix));
                let name_ident = format_ident!("{}", name);
                let types = fields
                    .iter()
                    .map(|field| {
                        self.idl_to_rust(
                            &field.typ,
                            None,
                            format_args!("{}_field{}", gen_prefix, field.label),
                        )
                    })
                    .collect::<Result<Vec<_>>>()?;
                if fields
                    .iter()
                    .all(|field| matches!(field.label, Label::Unnamed(_)))
                {
                    // all unnamed fields are better represented as a tuple struct
                    let body = quote! {
                        #[derive(::std::fmt::Debug, ::std::clone::Clone, ::std::cmp::PartialEq, ::ic_cdk::export::serde::Deserialize, ::ic_cdk::export::candid::CandidType)]
                        #[serde(crate = "::ic_cdk::export::serde")]
                        pub struct #name_ident ( #( pub #types ),* ) ;
                    };
                    self.decls.insert(name, body);
                } else {
                    let (names, serde_attrs) = fields
                        .iter()
                        .map(|field| label_to_ident(&field.label))
                        .map(|(name, id)| (name, id.map(|id| quote!(#[serde(rename = #id)]))))
                        .unzip::<_, _, Vec<_>, Vec<_>>();
                    let body = quote! {
                        #[derive(::std::fmt::Debug, ::std::clone::Clone, ::std::cmp::PartialEq, ::ic_cdk::export::serde::Deserialize, ::ic_cdk::export::candid::CandidType)]
                        #[serde(crate = "::ic_cdk::export::serde")]
                        pub struct #name_ident {
                            #(
                                #serde_attrs
                                pub #names : #types ,
                            )*
                        }
                    };
                    self.decls.insert(name, body);
                }
                return Ok(quote!(#name_ident)); // not a `type`
            }
            IDLType::VariantT(variants) => {
                let name = name
                    .map(String::from)
                    .unwrap_or_else(|| format!("_{}", gen_prefix));
                let name_ident = format_ident!("{}", name);
                let variants = variants
                    .iter()
                    .map(|variant| {
                        let (variant_name, idx) = label_to_ident(&variant.label);
                        let serde_attr = idx.map(|idx| quote!(#[serde(rename = #idx)]));
                        Ok(match &variant.typ {
                            // a single null field means it may as well be a unit variant
                            IDLType::PrimT(PrimType::Null) => quote!(#variant_name),
                            // handle individual fields of a record variant to avoid needing to generate an unnamed struct
                            IDLType::RecordT(fields) => {
                                let types = fields
                                    .iter()
                                    .map(|field| {
                                        self.idl_to_rust(
                                            &field.typ,
                                            None,
                                            format_args!(
                                                "{}_{}_field{}",
                                                gen_prefix, variant.label, field.label
                                            ),
                                        )
                                    })
                                    .collect::<Result<Vec<_>>>()?;
                                if fields
                                    .iter()
                                    .all(|field| matches!(field.label, Label::Unnamed(_)))
                                {
                                    // all unnamed fields means it's best represented as a tuple variant
                                    quote!(#variant_name ( #( #types ),* ))
                                } else {
                                    // otherwise, struct variant
                                    let (names, serde_field_attrs) = fields
                                        .iter()
                                        .map(|field| label_to_ident(&field.label))
                                        .map(|(name, id)| {
                                            (name, id.map(|id| quote!(#[serde(rename = #id)])))
                                        })
                                        .unzip::<_, _, Vec<_>, Vec<_>>();
                                    quote! {
                                        #serde_attr
                                        #variant_name {
                                            #(
                                                #serde_field_attrs
                                                #names : #types ,
                                             )*
                                        }
                                    }
                                }
                            }
                            r#type => {
                                let inner = self.idl_to_rust(
                                    r#type,
                                    None,
                                    format_args!("{}_{}", gen_prefix, variant.label),
                                )?;
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
                return Ok(quote!(#name_ident)); // not a `type`
            }
            IDLType::FuncT(func) => {
                let params = func
                    .args
                    .iter()
                    .enumerate()
                    .map(|(i, arg)| {
                        self.idl_to_rust(arg, None, format_args!("{}_arg{}", gen_prefix, i))
                    })
                    .collect::<Result<Vec<_>>>()?;
                let param_list = quote!(( #( #params ,)*));
                let rets = func
                    .rets
                    .iter()
                    .enumerate()
                    .map(|(i, ret)| {
                        self.idl_to_rust(ret, None, format_args!("{}_ret{}", gen_prefix, i))
                    })
                    .collect::<Result<Vec<_>>>()?;
                let ret_list = quote!(( #( #rets ,)* ));
                quote!(::ic_cdk::api::call::MethodRef< #param_list , #ret_list >)
            }
            IDLType::ServT(bindings) => {
                let bindings = bindings
                    .iter()
                    .map(|binding| {
                        let func = match &binding.typ {
                            IDLType::FuncT(func) => (
                                func.args
                                    .iter()
                                    .enumerate()
                                    .map(|(i, arg)| {
                                        self.idl_to_rust(
                                            arg,
                                            None,
                                            format_args!("{}_{}_arg{}", gen_prefix, binding.id, i),
                                        )
                                    })
                                    .collect::<Result<Vec<_>>>()?,
                                func.rets
                                    .iter()
                                    .enumerate()
                                    .map(|(i, ret)| {
                                        self.idl_to_rust(
                                            ret,
                                            None,
                                            format_args!("{}_{}_ret{}", gen_prefix, binding.id, i),
                                        )
                                    })
                                    .collect::<Result<Vec<_>>>()?,
                            ),
                            unknown => {
                                return Err(Error::new(
                                    Span::call_site(),
                                    format!(
                                        "Expected function at service {}.{}, found {:?}",
                                        name.unwrap_or("{anonymous}"),
                                        binding.id,
                                        unknown
                                    ),
                                ))
                            }
                        };
                        let ident = str_to_ident(&binding.id);
                        Ok((func, (ident, binding.id.clone())))
                    })
                    .collect::<Result<Vec<_>>>()?;
                let (functions, names) = bindings.into_iter().unzip::<_, _, Vec<_>, Vec<_>>();
                let (params, rets) = functions.into_iter().unzip::<_, _, Vec<_>, Vec<_>>();
                let (names, idl_names) = names.into_iter().unzip::<_, _, Vec<_>, Vec<_>>();
                let service_name = name
                    .map(String::from)
                    .unwrap_or_else(|| format!("_{}", gen_prefix));
                let param_names = params
                    .iter()
                    .map(|param| {
                        (0..param.len())
                            .map(|n| format_ident!("arg{}", n))
                            .collect::<Vec<_>>()
                    })
                    .collect::<Vec<_>>();
                let service_ident = str_to_ident(&service_name);
                let (full_ret, qual) = rets
                    .iter()
                    .map(|ret| {
                        if ret.len() == 1 {
                            let ret0 = &ret[0];
                            (quote!(#ret0), Some(quote!(.0)))
                        } else {
                            (quote!(#(#ret,)*), None)
                        }
                    })
                    .unzip::<_, _, Vec<_>, Vec<_>>();
                let body = quote! {
                    #[derive(::std::fmt::Debug, ::std::clone::Clone, ::std::cmp::PartialEq)]
                    pub struct #service_ident {
                        __id: ::ic_cdk::export::Principal,
                        #( pub #names : ::ic_cdk::api::call::MethodRef<( #( #params ,)* ), ( #( #rets ,)* )> ,)*
                    }
                    #[automatically_derived]
                    impl ::ic_cdk::export::candid::CandidType for #service_ident {
                        fn _ty() -> ::ic_cdk::export::candid::types::internal::Type {
                            ::ic_cdk::export::candid::types::internal::Type::Unknown
                        }
                        fn idl_serialize<S>(&self, serializer: S) -> ::std::result::Result<(), S::Error>
                        where
                            S: ic_cdk::export::candid::types::Serializer
                        {
                            serializer.serialize_principal(self.__id.as_slice())
                        }
                    }
                    #[automatically_derived]
                    impl<'de> ::ic_cdk::export::serde::Deserialize<'de> for #service_ident {
                        fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
                        where
                            D: ::ic_cdk::export::serde::Deserializer<'de>,
                        {
                            let principal = <::ic_cdk::export::Principal as ::ic_cdk::export::serde::Deserialize<'de>>::deserialize(deserializer)?;
                            ::std::result::Result::Ok(Self {
                                __id: principal,
                                #(
                                    #names : ::ic_cdk::api::call::MethodRef::new(principal, ::std::string::ToString::to_string( #idl_names )),
                                )*
                            })
                        }
                    }
                    impl #service_ident {
                        #(
                            pub async fn #names (&self, #( #param_names : #params ),* ) -> ::ic_cdk::api::call::CallResult< #full_ret > {
                                ::ic_cdk::api::call::CallResult::Ok(self. #names .invoke(( #( #param_names ,)* )).await? #qual)
                            }
                        )*
                    }
                };
                self.decls.insert(service_name, body);
                return Ok(quote!(#service_ident)); // not a `type`
            }
            IDLType::VecT(inner) => {
                let inner = self.idl_to_rust(inner, None, gen_prefix)?;
                quote!(::std::vec::Vec< #inner >)
            }
            IDLType::OptT(inner) => {
                let inner = self.idl_to_rust(inner, None, gen_prefix)?;
                quote!(::std::option::Option< #inner >)
            }
            IDLType::PrimT(prim) => {
                // Don't actually insert the corresponding Rust primitive here, we did that earlier so that the user could override it
                let prim_ident = format_ident!("{}", prim_to_name(prim.clone()));
                quote!(#prim_ident)
            }
            IDLType::PrincipalT => quote!(Principal),
            IDLType::VarT(name) => {
                // given `b = a; c = b; a = c;`:
                let ident = str_to_ident(name);
                // detect the possibility of `c`
                if let Some(v) = self.unresolved_containers.get(name) {
                    for s in v.clone() {
                        vecmap_insert(&mut self.unresolved, &s, name.clone());
                        vecmap_insert(&mut self.unresolved_containers, name, s);
                    }
                }
                // handle recursion
                if self.current_pass.iter().any(|x| x == name) {
                    quote!(::std::boxed::Box< #ident >)
                } else {
                    // starting here at `b`
                    if !self.decls.contains_key(name) {
                        // hope it will be resolved later, otherwise the compile error should be meaningful enough
                        // this also means it may be a mutually recursive type with something else
                        for referent in &self.current_pass {
                            vecmap_insert(&mut self.unresolved, name, referent.clone());
                            vecmap_insert(&mut self.unresolved_containers, referent, name.clone());
                        }
                    }
                    quote!(#ident)
                }
            }
            IDLType::ClassT(_, _) => return Err(Error::new(Span::call_site(), "Unexpected class type outside primary service")),
        };
        if let Some(name) = name {
            let ident = format_ident!("{}", name);
            self.decls
                .insert(name.to_owned(), quote!(pub type #ident = #ret ;));
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
            let ident = str_to_ident(&binding.id);
            let args = func
                .args
                .iter()
                .enumerate()
                .map(|(i, arg)| {
                    let ty = self.idl_to_rust(arg, None, format_args!("_service"))?;
                    let ident = format_ident!("arg{}", i);
                    Ok(quote!(#ident : #ty))
                })
                .collect::<Result<Vec<_>>>()?;
            let rets = func
                .rets
                .iter()
                .map(|arg| self.idl_to_rust(arg, None, format_args!("_service")))
                .collect::<Result<Vec<_>>>()?;
            let ret_coll = if rets.len() != 1 {
                quote!(( #( #rets ),* ))
            } else {
                quote!( #( #rets )* )
            };
            self.bindings.insert(
                binding.id,
                parse2(quote!(async fn #ident ( #( #args ),* ) -> #ret_coll )).unwrap(),
            );
        }
        Ok(())
    }

    /// Generates the final `TokenStream` from all the bindings, with the actor functions referencing the provided principal.
    ///
    /// This token stream is not encased in a surrounding module. User code should probably pair this with `#[allow(nonstandard_style)]`;
    /// there is no built-in `#!` version of that to avoid requiring these to be the first lines in a module.
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
                FnArg::Receiver(recv) => Err(Error::new(recv.span(), "Not valid here")), // `self`
                FnArg::Typed(typed) => {
                    match &*typed.pat {
                        Pat::Ident(i) => Ok(&i.ident),
                        other => Err(Error::new(other.span(), "Must be a plain parameter")) // e.g. `(x, y): (i32, i32)`
                    }
                }
            }).collect::<Result<Vec<_>>>()?;
            let name = &sig.ident;
            let name_str = name.to_string();
            // no sense adding parsing overhead when we can do it now
            let principal_bytes = principal.as_slice();
            // if there is only one return value, the return type should be the inner value; manually collect a 1-tuple and then access the field
            // this should be opaque to candid as tuples are not actually regular values but only valid in functions; `((x,),)` should render as `(x,)`
            // so the user shouldn't be locked out of using a 1-tuple as the return type if they really want to
            let one_return = matches!(&sig.output, ReturnType::Type(_, t) if !matches!(&**t, Type::Tuple(..)));
            let expr = one_return.then(|| quote!(.0));
            let qual = one_return.then(|| quote!(::<_, (_,)>));
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
    Null: (),
    Reserved: ::ic_cdk::export::candid::Reserved,
    Empty: ::ic_cdk::export::candid::Empty,
    Text: String,
];

/// Converts a series of items to the initial fields in `Processor`.
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
                // recursively navigate annoying `use` trees
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
                        // `pub use Foo as Bar`
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
            // Functions ending in `;` are not semantically valid `Item`s because they never appear at the top level, so this must be reparsed
            // The choice here is between `ForeignItemFn` and `TraitItemMethod`; we choose the former because foreign functions can have a visibility and trait functions can't.
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
        // separate function because of duplicated code between Fn and Verbatim(ForeignItemFn)
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

fn label_to_ident(label: &Label) -> (Ident, Option<String>) {
    match label {
        Label::Id(id) => (format_ident!("__{}", id), Some(id.to_string())),
        Label::Named(name) => {
            let ident = str_to_ident(name);
            let real = (ident != name).then(|| name.clone());
            (ident, real)
        }
        Label::Unnamed(idx) => (format_ident!("_{}", idx), Some(idx.to_string())),
    }
}

/// Converts a string name perhaps obtained from Candid to a valid Rust ident and an associated hash if the ident had to be hashed
fn str_to_ident(name: &str) -> Ident {
    if let Ok(ident) = parse_str::<Ident>(name) {
        ident
    } else {
        let chars = name
            .chars()
            .map(|c| if !c.is_alphanumeric() { '_' } else { c });
        let valid = iter::once('_').chain(chars).collect::<String>();
        format_ident!("{}", valid)
    }
}

fn vecmap_insert<K: Hash + Eq, K2: Hash + Eq + ToOwned<Owned = K>, V>(
    m: &mut HashMap<K, Vec<V>>,
    key: &K2,
    value: V,
) where
    K: Borrow<K2>,
{
    if let Some(vec) = m.get_mut(key) {
        vec.push(value);
    } else {
        m.insert(key.to_owned(), vec![value]);
    }
}

/// Quick convenience function to take a Candid file and write the bindings to an output file.
///
/// The input file is relative to the manifest dir, and the output file is relative to `$OUT_DIR` if it exists, or the manifest dir if it doesn't.
///
/// # Examples
///
/// `build.rs`:
/// ```rust,no_run
/// # fn main() {
///     ic_cdk_codegen::process_file("ic.did", "ic.rs", "aaaaa-aa".parse().unwrap()).unwrap();
/// # }
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
        // OUT_DIR is only present if there is a build script
        // this is somewhat inconsistent but this function is in the name of maximum convenience
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
    let mut actor = prog.actor;
    if let Some(IDLType::ClassT(_, ret)) = actor {
        actor = Some(*ret)
    }
    if let Some(IDLType::ServT(actor)) = actor {
        processor.add_primary_actor(actor)?;
    }
    let bindings = processor.generate(principal)?;
    Ok(bindings)
}
