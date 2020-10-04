use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use serde::Deserialize;
use serde_tokenstream::from_tokenstream;
use std::sync::atomic::{AtomicBool, Ordering};
use syn::export::Formatter;
use syn::Error;
use syn::{spanned::Spanned, FnArg, ItemFn, Pat, PatIdent, PatType, ReturnType, Signature, Type};

#[derive(Default, Deserialize)]
struct ExportAttributes {
    pub name: Option<String>,
    pub guard: Option<String>,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum MethodType {
    Init,
    PreUpgrade,
    PostUpgrade,
    Update,
    Query,
}

impl MethodType {
    pub fn is_lifecycle(&self) -> bool {
        match self {
            MethodType::Init | MethodType::PreUpgrade | MethodType::PostUpgrade => true,
            _ => false,
        }
    }
}

impl std::fmt::Display for MethodType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            MethodType::Init => f.write_str("canister_init"),
            MethodType::PreUpgrade => f.write_str("canister_pre_upgrade"),
            MethodType::PostUpgrade => f.write_str("canister_post_upgrade"),
            MethodType::Query => f.write_str("canister_query"),
            MethodType::Update => f.write_str("canister_update"),
        }
    }
}

fn get_args(method: MethodType, signature: &Signature) -> Result<Vec<(Ident, Box<Type>)>, Error> {
    // We only need the tuple of arguments, not their types. Magic of type inference.
    let mut args = vec![];
    for ref arg in &signature.inputs {
        let (ident, ty) = match arg {
            FnArg::Receiver(r) => {
                return Err(Error::new(
                    r.span(),
                    format!(
                        "#[{}] cannot be above functions with `self` as a parameter",
                        method
                    ),
                ));
            }
            FnArg::Typed(PatType { pat, ty, .. }) => {
                if let Pat::Ident(PatIdent { ident, .. }) = pat.as_ref() {
                    (ident.clone(), ty.clone())
                } else {
                    (
                        syn::Ident::new(&format!("arg_{}", crate::id()), pat.span()),
                        ty.clone(),
                    )
                }
            }
        };

        args.push((ident, ty));
    }

    Ok(args)
}

fn dfn_macro(
    method: MethodType,
    attr: TokenStream,
    item: TokenStream,
) -> Result<TokenStream, Error> {
    let attrs = from_tokenstream::<ExportAttributes>(&attr)?;

    let fun: ItemFn = syn::parse2::<syn::ItemFn>(item.clone()).map_err(|e| {
        Error::new(
            item.span(),
            format!("#[{0}] must be above a function, \n{1}", method, e),
        )
    })?;
    let signature = &fun.sig;
    let generics = &signature.generics;

    if !generics.params.is_empty() {
        return Err(Error::new(
            generics.span(),
            format!(
                "#[{}] must be above a function with no generic parameters",
                method
            ),
        ));
    }

    let is_async = signature.asyncness.is_some();

    let return_length = match &signature.output {
        ReturnType::Default => 0,
        ReturnType::Type(_, ty) => match ty.as_ref() {
            Type::Tuple(tuple) => tuple.elems.len(),
            _ => 1,
        },
    };

    match method {
        MethodType::Init | MethodType::PreUpgrade | MethodType::PostUpgrade
            if return_length > 0 =>
        {
            return Err(Error::new(
                Span::call_site(),
                format!("#[{}] function cannot have a return value.", method),
            ));
        }
        _ => {}
    }

    let (arg_tuple, _): (Vec<Ident>, Vec<Box<Type>>) =
        get_args(method, signature)?.iter().cloned().unzip();
    let name = &signature.ident;

    let outer_function_ident = Ident::new(
        &format!("{}_{}_", name.to_string(), crate::id()),
        Span::call_site(),
    );

    let export_name = if method.is_lifecycle() {
        format!("{}", method)
    } else {
        format!(
            "{0} {1}",
            method,
            attrs.name.unwrap_or_else(|| name.to_string())
        )
    };

    let function_call = if is_async {
        quote! { #name ( #(#arg_tuple),* ) .await }
    } else {
        quote! { #name ( #(#arg_tuple),* ) }
    };

    let arg_count = arg_tuple.len();

    let return_encode = if method.is_lifecycle() {
        quote! {}
    } else {
        match return_length {
            0 => quote! { ic_cdk::api::call::reply(()) },
            1 => quote! { ic_cdk::api::call::reply((result,)) },
            _ => quote! { ic_cdk::api::call::reply(result) },
        }
    };

    // On initialization we can actually not receive any input and it's okay, only if
    // we don't have any arguments either.
    // If the data we receive is not empty, then try to unwrap it as if it's DID.
    let arg_decode = if method.is_lifecycle() && arg_count == 0 {
        quote! {}
    } else {
        quote! { let ( #( #arg_tuple, )* ) = ic_cdk::api::call::arg_data(); }
    };

    let guard = if let Some(guard_name) = attrs.guard {
        let guard_ident = syn::Ident::new(&guard_name, Span::call_site());

        quote! {
            let r: Result<(), String> = #guard_ident ();
            if let Err(e) = r {
                ic_cdk::api::call::reject(&e);
                return;
            }
        }
    } else {
        quote! {}
    };

    Ok(quote! {
        #[export_name = #export_name]
        fn #outer_function_ident() {
            ic_cdk::setup();

            #guard

            ic_cdk::block_on(async {
                #arg_decode
                let result = #function_call;
                #return_encode
            });
        }

        #item
    })
}

pub(crate) fn ic_query(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> Result<proc_macro::TokenStream, Error> {
    dfn_macro(
        MethodType::Query,
        TokenStream::from(attr),
        TokenStream::from(item),
    )
    .map(proc_macro::TokenStream::from)
}
pub(crate) fn ic_update(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> Result<proc_macro::TokenStream, Error> {
    dfn_macro(
        MethodType::Update,
        TokenStream::from(attr),
        TokenStream::from(item),
    )
    .map(proc_macro::TokenStream::from)
}

#[derive(Default, Deserialize)]
struct InitAttributes {}

static IS_INIT: AtomicBool = AtomicBool::new(false);

pub(crate) fn ic_init(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> Result<proc_macro::TokenStream, Error> {
    if IS_INIT.swap(true, Ordering::SeqCst) {
        return Err(Error::new(
            Span::call_site(),
            "Init function already declared.",
        ));
    }

    dfn_macro(
        MethodType::Init,
        TokenStream::from(attr),
        TokenStream::from(item),
    )
    .map(proc_macro::TokenStream::from)
}

static HAS_PRE_UPGRADE: AtomicBool = AtomicBool::new(false);

pub(crate) fn ic_pre_upgrade(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> Result<proc_macro::TokenStream, Error> {
    if HAS_PRE_UPGRADE.swap(true, Ordering::SeqCst) {
        return Err(Error::new(
            Span::call_site(),
            "Pre-upgrade function already declared.",
        ));
    }

    dfn_macro(
        MethodType::PreUpgrade,
        TokenStream::from(attr),
        TokenStream::from(item),
    )
    .map(proc_macro::TokenStream::from)
}

static HAS_POST_UPGRADE: AtomicBool = AtomicBool::new(false);

pub(crate) fn ic_post_upgrade(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> Result<proc_macro::TokenStream, Error> {
    if HAS_POST_UPGRADE.swap(true, Ordering::SeqCst) {
        return Err(Error::new(
            Span::call_site(),
            "Post-upgrade function already declared.",
        ));
    }

    dfn_macro(
        MethodType::PostUpgrade,
        TokenStream::from(attr),
        TokenStream::from(item),
    )
    .map(proc_macro::TokenStream::from)
}
