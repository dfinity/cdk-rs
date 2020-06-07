use crate::error::Errors;
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use serde::Deserialize;
use serde_tokenstream::from_tokenstream;
use syn::export::Formatter;
use syn::{spanned::Spanned, FnArg, ItemFn, Pat, PatIdent, PatType, ReturnType, Type};

#[derive(Default, Deserialize)]
struct ExportAttributes {
    pub name: Option<String>,
}

#[derive(Copy, Clone, Debug)]
enum MethodType {
    Update,
    Query,
}

impl std::fmt::Display for MethodType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            MethodType::Query => f.write_str("query"),
            MethodType::Update => f.write_str("update"),
        }
    }
}

fn dfn_macro(
    method: MethodType,
    attr: TokenStream,
    item: TokenStream,
) -> Result<TokenStream, Errors> {
    let attrs = match from_tokenstream::<ExportAttributes>(&proc_macro2::TokenStream::from(attr)) {
        Ok(a) => a,
        Err(err) => return Err(Errors::message(format!("{}", err.to_compile_error()))),
    };

    let fun: ItemFn = syn::parse2::<syn::ItemFn>(item.clone()).map_err(|e| {
        Errors::single(
            format!("#[ic_{0}] must be above a function, \n{1}", method, e),
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

    let function_args = &signature.inputs;
    let empty_args = function_args.len() == 0;
    let empty_return = match &signature.output {
        ReturnType::Default => true,
        ReturnType::Type(_, ty) => match ty.as_ref() {
            Type::Tuple(tuple) => tuple.elems.len() == 0,
            _ => false,
        },
    };

    // We only need the tuple of arguments, not their types. Magic of type inference.
    let mut arg_tuple = vec![];
    let mut arg_types = vec![];
    for ref arg in function_args {
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

        arg_tuple.push(ident);
        arg_types.push(ty);
    }

    let name = &signature.ident;

    let outer_function_ident = Ident::new(
        &format!("{}_{}_", name.to_string(), crate::id()),
        Span::call_site(),
    );

    let export_name = format!(
        "canister_{0} {1}",
        method,
        attrs.name.unwrap_or(name.to_string())
    );

    let function_call = if is_async {
        quote! { #name ( #(#arg_tuple),* ) .await }
    } else {
        quote! { #name ( #(#arg_tuple),* ) }
    };

    let arg_decode = if empty_args {
        quote! { ic_cdk::context::arg_data_empty() }
    } else {
        quote! { ic_cdk::context::arg_data() }
    };

    let return_encode = if empty_return {
        quote! { ic_cdk::context::reply_empty() }
    } else {
        quote! { ic_cdk::context::reply(result) }
    };

    Ok(quote! {
        #[export_name = #export_name]
        fn #outer_function_ident() {
            ic_cdk::setup();

            ic_cdk::block_on(async {
                let ( #( #arg_tuple ),* ) = #arg_decode;
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
