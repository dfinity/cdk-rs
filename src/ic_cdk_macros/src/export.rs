use crate::error::Errors;
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use serde::Deserialize;
use serde_tokenstream::from_tokenstream;
use std::sync::atomic::{AtomicBool, Ordering};
use syn::export::Formatter;
use syn::{spanned::Spanned, FnArg, ItemFn, Pat, PatIdent, PatType, ReturnType, Signature, Type};

#[derive(Default, Deserialize)]
struct ExportAttributes {
    pub name: Option<String>,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum MethodType {
    Init,
    Update,
    Query,
}

impl std::fmt::Display for MethodType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            MethodType::Init => f.write_str("init"),
            MethodType::Query => f.write_str("query"),
            MethodType::Update => f.write_str("update"),
        }
    }
}

fn get_args(method: MethodType, signature: &Signature) -> Result<Vec<(Ident, Box<Type>)>, Errors> {
    // We only need the tuple of arguments, not their types. Magic of type inference.
    let mut args = vec![];
    for ref arg in &signature.inputs {
        let (ident, ty) = match arg {
            FnArg::Receiver(r) => {
                return Err(Errors::single(
                    format!(
                        "#[{}] cannot be above functions with `self` as a parameter",
                        method
                    ),
                    r.span(),
                ))
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
) -> Result<TokenStream, Errors> {
    let attrs = match from_tokenstream::<ExportAttributes>(&attr) {
        Ok(a) => a,
        Err(err) => return Err(Errors::message(format!("{}", err.to_compile_error()))),
    };

    let fun: ItemFn = syn::parse2::<syn::ItemFn>(item.clone()).map_err(|e| {
        Errors::single(
            format!("#[{0}] must be above a function, \n{1}", method, e),
            item.span(),
        )
    })?;
    let signature = &fun.sig;
    let generics = &signature.generics;

    if !generics.params.is_empty() {
        return Err(Errors::single(
            format!(
                "#[{}] must be above a function with no generic parameters",
                method
            ),
            generics,
        ));
    }

    let is_async = signature.asyncness.is_some();

    let empty_return = match &signature.output {
        ReturnType::Default => true,
        ReturnType::Type(_, ty) => match ty.as_ref() {
            Type::Tuple(tuple) => tuple.elems.is_empty(),
            _ => false,
        },
    };

    if method == MethodType::Init && !empty_return {
        return Err(Errors::message(
            "#[init] function cannot have a return value.",
        ));
    }

    let (arg_tuple, _): (Vec<Ident>, Vec<Box<Type>>) =
        get_args(method, signature)?.iter().cloned().unzip();
    let name = &signature.ident;

    let outer_function_ident = Ident::new(
        &format!("{}_{}_", name.to_string(), crate::id()),
        Span::call_site(),
    );

    let export_name = if method == MethodType::Init {
        "canister_init".to_string()
    } else {
        format!(
            "canister_{0} {1}",
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
    let arg_decode = syn::Ident::new(&format!("arg_data_{}", arg_count), Span::call_site());

    let return_encode = if method == MethodType::Init {
        quote! {}
    } else if empty_return {
        quote! { ic_cdk::context::reply_empty() }
    } else {
        quote! { ic_cdk::context::reply(result) }
    };

    // On initialization we can actually not receive any input and it's okay, only if
    // we don't have any arguments either.
    // If the data we receive is not empty, then try to unwrap it as if it's DID.
    let arg_decode = if method == MethodType::Init && arg_count == 0 {
        quote! {
            if !ic_cdk::context::arg_data_is_empty() {
                let _ = ic_cdk::context::arg_data_0();
            }
        }
    } else {
        quote! { let ( #( #arg_tuple ),* ) = ic_cdk::context::#arg_decode(); }
    };

    Ok(quote! {
        #[export_name = #export_name]
        fn #outer_function_ident() {
            ic_cdk::setup();

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
) -> Result<proc_macro::TokenStream, Errors> {
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
) -> Result<proc_macro::TokenStream, Errors> {
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
) -> Result<proc_macro::TokenStream, Errors> {
    if IS_INIT.swap(true, Ordering::SeqCst) {
        return Err(Errors::message("Init function already declared."));
    }

    dfn_macro(
        MethodType::Init,
        TokenStream::from(attr),
        TokenStream::from(item),
    )
    .map(proc_macro::TokenStream::from)
}
