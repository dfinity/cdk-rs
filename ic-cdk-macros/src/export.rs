use proc_macro2::{Ident, Span, TokenStream};
use quote::{format_ident, quote};
use serde::Deserialize;
use serde_tokenstream::from_tokenstream;
use std::fmt::Formatter;
use syn::Error;
use syn::{spanned::Spanned, FnArg, ItemFn, Pat, PatIdent, PatType, ReturnType, Signature, Type};

#[derive(Default, Deserialize)]
struct ExportAttributes {
    pub name: Option<String>,
    pub guard: Option<String>,
    #[serde(default)]
    pub manual_reply: bool,
    #[serde(default)]
    pub composite: bool,
    #[serde(default)]
    pub hidden: bool,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum MethodType {
    Init,
    PreUpgrade,
    PostUpgrade,
    Update,
    Query,
    Heartbeat,
    InspectMessage,
    OnLowWasmMemory,
}

impl MethodType {
    /// A lifecycle method is a method that is called by the system and not by the user.
    /// So far, `update` and `query` are the only methods that are not lifecycle methods.
    ///
    /// We have a few assumptions for lifecycle methods:
    /// - They cannot have a return value.
    /// - The export name is prefixed with `canister_`, e.g. `init` => `canister_init`.
    pub fn is_lifecycle(&self) -> bool {
        match self {
            MethodType::Init
            | MethodType::PreUpgrade
            | MethodType::PostUpgrade
            | MethodType::Heartbeat
            | MethodType::InspectMessage
            | MethodType::OnLowWasmMemory => true,
            MethodType::Update | MethodType::Query => false,
        }
    }
}

impl std::fmt::Display for MethodType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            MethodType::Init => f.write_str("init"),
            MethodType::PreUpgrade => f.write_str("pre_upgrade"),
            MethodType::PostUpgrade => f.write_str("post_upgrade"),
            MethodType::Query => f.write_str("query"),
            MethodType::Update => f.write_str("update"),
            MethodType::Heartbeat => f.write_str("heartbeat"),
            MethodType::InspectMessage => f.write_str("inspect_message"),
            MethodType::OnLowWasmMemory => f.write_str("on_low_wasm_memory"),
        }
    }
}

fn get_args(method: MethodType, signature: &Signature) -> Result<Vec<(Ident, Box<Type>)>, Error> {
    // We only need the tuple of arguments, not their types. Magic of type inference.
    let mut args = vec![];
    for (i, arg) in signature.inputs.iter().enumerate() {
        let (ident, ty) = match arg {
            FnArg::Receiver(r) => {
                return Err(Error::new(
                    r.span(),
                    format!(
                        "#[{}] cannot be above functions with `self` as a parameter.",
                        method
                    ),
                ));
            }
            FnArg::Typed(PatType { pat, ty, .. }) => {
                let ident = if let Pat::Ident(PatIdent { ident, .. }) = pat.as_ref() {
                    // If the argument is named the same as the function, we need to rename it.
                    if ident == &signature.ident {
                        format_ident!("__arg_{}", ident, span = pat.span())
                    } else {
                        ident.clone()
                    }
                } else {
                    format_ident!("__unnamed_arg_{i}", span = pat.span())
                };
                (ident, ty.clone())
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
    let attrs = from_tokenstream::<ExportAttributes>(&attr)
        .map_err(|e| Error::new(attr.span(), format!("Failed to deserialize {attr}. \n{e}")))?;

    let fun: ItemFn = syn::parse2::<syn::ItemFn>(item.clone()).map_err(|e| {
        Error::new(
            item.span(),
            format!("#[{0}] must be above a function. \n{1}", method, e),
        )
    })?;
    let signature = &fun.sig;
    let generics = &signature.generics;

    if !generics.params.is_empty() {
        return Err(Error::new(
            generics.span(),
            format!(
                "#[{}] must be above a function with no generic parameters.",
                method
            ),
        ));
    }

    // 1. function name(s)
    let name = &signature.ident;
    let outer_function_ident = format_ident!("__canister_method_{name}");
    let function_name = attrs.name.unwrap_or_else(|| name.to_string());
    let export_name = if method.is_lifecycle() {
        format!("canister_{}", method)
    } else if method == MethodType::Query && attrs.composite {
        format!("canister_composite_query {function_name}",)
    } else {
        if function_name.starts_with("<ic-cdk internal>") {
            return Err(Error::new(
                Span::call_site(),
                "Functions starting with `<ic-cdk internal>` are reserved for CDK internal use.",
            ));
        }
        format!("canister_{method} {function_name}")
    };
    let host_compatible_name = export_name.replace(' ', ".").replace(['-', '<', '>'], "_");

    // 2. guard
    let guard = if let Some(guard_name) = attrs.guard {
        // ic_cdk::api::call::reject calls ic0::msg_reject which is only allowed in update/query
        if method.is_lifecycle() {
            return Err(Error::new(
                attr.span(),
                format!("#[{}] cannot have a guard function.", method),
            ));
        }
        let guard_ident = syn::Ident::new(&guard_name, Span::call_site());

        quote! {
            let r: Result<(), String> = #guard_ident ();
            if let Err(e) = r {
                ::ic_cdk::api::msg_reject(&e);
                return;
            }
        }
    } else {
        quote! {}
    };

    // 3. decode arguments
    let (arg_tuple, _): (Vec<Ident>, Vec<Box<Type>>) =
        get_args(method, signature)?.iter().cloned().unzip();
    let arg_decode = if method.is_lifecycle() && arg_tuple.len() == 0 {
        quote! {}
    } else {
        quote! {
        let arg_bytes = ::ic_cdk::api::msg_arg_data();
        let ( #( #arg_tuple, )* ) = ::candid::utils::decode_args(&arg_bytes).unwrap(); }
    };

    // 4. function call
    let function_call = if signature.asyncness.is_some() {
        quote! { #name ( #(#arg_tuple),* ) .await }
    } else {
        quote! { #name ( #(#arg_tuple),* ) }
    };

    // 5. return
    let return_length = match &signature.output {
        ReturnType::Default => 0,
        ReturnType::Type(_, ty) => match ty.as_ref() {
            Type::Tuple(tuple) => tuple.elems.len(),
            _ => 1,
        },
    };
    if method.is_lifecycle() && return_length > 0 {
        return Err(Error::new(
            Span::call_site(),
            format!("#[{}] function cannot have a return value.", method),
        ));
    }
    let return_encode = if method.is_lifecycle() || attrs.manual_reply {
        quote! {}
    } else {
        let return_bytes = match return_length {
            0 => quote! { ::candid::utils::encode_one(()).unwrap() },
            1 => quote! { ::candid::utils::encode_one(result).unwrap() },
            _ => quote! { ::candid::utils::encode_args(result).unwrap() },
        };
        quote! {
            ::ic_cdk::api::msg_reply(#return_bytes);
        }
    };

    // 6. candid attributes for export_candid!()
    let candid_method_attr = if attrs.hidden {
        quote! {}
    } else {
        match method {
            MethodType::Query if attrs.composite => {
                quote! { #[::candid::candid_method(composite_query, rename = #function_name)] }
            }
            MethodType::Query => {
                quote! { #[::candid::candid_method(query, rename = #function_name)] }
            }
            MethodType::Update => {
                quote! { #[::candid::candid_method(update, rename = #function_name)] }
            }
            MethodType::Init => quote! { #[::candid::candid_method(init)] },
            _ => quote! {},
        }
    };
    let item = quote! {
        #candid_method_attr
        #item
    };

    Ok(quote! {
        #[cfg_attr(target_family = "wasm", export_name = #export_name)]
        #[cfg_attr(not(target_family = "wasm"), export_name = #host_compatible_name)]
        fn #outer_function_ident() {
            ::ic_cdk::setup();

            #guard

            ::ic_cdk::spawn(async {
                #arg_decode
                let result = #function_call;
                #return_encode
            });
        }

        #item
    })
}

pub(crate) fn ic_query(attr: TokenStream, item: TokenStream) -> Result<TokenStream, Error> {
    dfn_macro(MethodType::Query, attr, item)
}

pub(crate) fn ic_update(attr: TokenStream, item: TokenStream) -> Result<TokenStream, Error> {
    dfn_macro(MethodType::Update, attr, item)
}

pub(crate) fn ic_init(attr: TokenStream, item: TokenStream) -> Result<TokenStream, Error> {
    dfn_macro(MethodType::Init, attr, item)
}

pub(crate) fn ic_pre_upgrade(attr: TokenStream, item: TokenStream) -> Result<TokenStream, Error> {
    dfn_macro(MethodType::PreUpgrade, attr, item)
}

pub(crate) fn ic_post_upgrade(attr: TokenStream, item: TokenStream) -> Result<TokenStream, Error> {
    dfn_macro(MethodType::PostUpgrade, attr, item)
}

pub(crate) fn ic_heartbeat(attr: TokenStream, item: TokenStream) -> Result<TokenStream, Error> {
    dfn_macro(MethodType::Heartbeat, attr, item)
}

pub(crate) fn ic_inspect_message(
    attr: TokenStream,
    item: TokenStream,
) -> Result<TokenStream, Error> {
    dfn_macro(MethodType::InspectMessage, attr, item)
}

pub(crate) fn ic_on_low_wasm_memory(
    attr: TokenStream,
    item: TokenStream,
) -> Result<TokenStream, Error> {
    dfn_macro(MethodType::OnLowWasmMemory, attr, item)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn ic_query_empty() {
        let generated = ic_query(
            quote!(),
            quote! {
                fn query() {}
            },
        )
        .unwrap();
        let parsed = syn::parse2::<syn::File>(generated).unwrap();
        let fn_name = match parsed.items[0] {
            syn::Item::Fn(ref f) => &f.sig.ident,
            _ => panic!("Incorrect parsed AST."),
        };

        let expected = quote! {
            #[cfg_attr(target_family = "wasm", export_name = "canister_query query")]
            #[cfg_attr(not(target_family = "wasm"), export_name = "canister_query.query")]
            fn #fn_name() {
                ::ic_cdk::setup();
                ::ic_cdk::spawn(async {
                    let arg_bytes = ::ic_cdk::api::msg_arg_data();
                    let () = ::candid::utils::decode_args(&arg_bytes).unwrap();
                    let result = query();
                    ::ic_cdk::api::msg_reply(::candid::utils::encode_one(()).unwrap());
                });
            }
        };
        let expected = syn::parse2::<syn::ItemFn>(expected).unwrap();

        assert!(parsed.items.len() == 2);
        match &parsed.items[0] {
            syn::Item::Fn(f) => {
                assert_eq!(*f, expected);
            }
            _ => panic!("not a function"),
        };
    }

    #[test]
    fn ic_query_return_one_value() {
        let generated = ic_query(
            quote!(),
            quote! {
                fn query() -> u32 {}
            },
        )
        .unwrap();
        let parsed = syn::parse2::<syn::File>(generated).unwrap();
        let fn_name = match parsed.items[0] {
            syn::Item::Fn(ref f) => &f.sig.ident,
            _ => panic!("Incorrect parsed AST."),
        };

        let expected = quote! {
            #[cfg_attr(target_family = "wasm", export_name = "canister_query query")]
            #[cfg_attr(not(target_family = "wasm"), export_name = "canister_query.query")]
            fn #fn_name() {
                ::ic_cdk::setup();
                ::ic_cdk::spawn(async {
                    let arg_bytes = ::ic_cdk::api::msg_arg_data();
                    let () = ::candid::utils::decode_args(&arg_bytes).unwrap();
                    let result = query();
                    ::ic_cdk::api::msg_reply(::candid::utils::encode_one(result).unwrap());
                });
            }
        };
        let expected = syn::parse2::<syn::ItemFn>(expected).unwrap();

        assert!(parsed.items.len() == 2);
        match &parsed.items[0] {
            syn::Item::Fn(f) => {
                assert_eq!(*f, expected);
            }
            _ => panic!("not a function"),
        };
    }

    #[test]
    fn ic_query_return_tuple() {
        let generated = ic_query(
            quote!(),
            quote! {
                fn query() -> (u32, u32) {}
            },
        )
        .unwrap();
        let parsed = syn::parse2::<syn::File>(generated).unwrap();
        let fn_name = match parsed.items[0] {
            syn::Item::Fn(ref f) => &f.sig.ident,
            _ => panic!("Incorrect parsed AST."),
        };

        let expected = quote! {
            #[cfg_attr(target_family = "wasm", export_name = "canister_query query")]
            #[cfg_attr(not(target_family = "wasm"), export_name = "canister_query.query")]
            fn #fn_name() {
                ::ic_cdk::setup();
                ::ic_cdk::spawn(async {
                    let arg_bytes = ::ic_cdk::api::msg_arg_data();
                    let () = ::candid::utils::decode_args(&arg_bytes).unwrap();
                    let result = query();
                    ::ic_cdk::api::msg_reply(::candid::utils::encode_args(result).unwrap());
                });
            }
        };
        let expected = syn::parse2::<syn::ItemFn>(expected).unwrap();

        assert!(parsed.items.len() == 2);
        match &parsed.items[0] {
            syn::Item::Fn(f) => {
                assert_eq!(*f, expected);
            }
            _ => panic!("not a function"),
        };
    }

    #[test]
    fn ic_query_one_arg() {
        let generated = ic_query(
            quote!(),
            quote! {
                fn query(a: u32) {}
            },
        )
        .unwrap();
        let parsed = syn::parse2::<syn::File>(generated).unwrap();
        let fn_name = match parsed.items[0] {
            syn::Item::Fn(ref f) => &f.sig.ident,
            _ => panic!("Incorrect parsed AST."),
        };

        let expected = quote! {
            #[cfg_attr(target_family = "wasm", export_name = "canister_query query")]
            #[cfg_attr(not(target_family = "wasm"), export_name = "canister_query.query")]
            fn #fn_name() {
                ::ic_cdk::setup();
                ::ic_cdk::spawn(async {
                    let arg_bytes = ::ic_cdk::api::msg_arg_data();
                    let (a,) = ::candid::utils::decode_args(&arg_bytes).unwrap();
                    let result = query(a);
                    ::ic_cdk::api::msg_reply(::candid::utils::encode_one(()).unwrap());
                });
            }
        };
        let expected = syn::parse2::<syn::ItemFn>(expected).unwrap();

        assert!(parsed.items.len() == 2);
        match &parsed.items[0] {
            syn::Item::Fn(f) => {
                assert_eq!(*f, expected);
            }
            _ => panic!("not a function"),
        };
    }

    #[test]
    fn ic_query_two_args() {
        let generated = ic_query(
            quote!(),
            quote! {
                fn query(a: u32, b: u32) {}
            },
        )
        .unwrap();
        let parsed = syn::parse2::<syn::File>(generated).unwrap();
        let fn_name = match parsed.items[0] {
            syn::Item::Fn(ref f) => &f.sig.ident,
            _ => panic!("Incorrect parsed AST."),
        };

        let expected = quote! {
            #[cfg_attr(target_family = "wasm", export_name = "canister_query query")]
            #[cfg_attr(not(target_family = "wasm"), export_name = "canister_query.query")]
            fn #fn_name() {
                ::ic_cdk::setup();
                ::ic_cdk::spawn(async {
                    let arg_bytes = ::ic_cdk::api::msg_arg_data();
                    let (a, b,) = ::candid::utils::decode_args(&arg_bytes).unwrap();
                    let result = query(a, b);
                    ::ic_cdk::api::msg_reply(::candid::utils::encode_one(()).unwrap());
                });
            }
        };
        let expected = syn::parse2::<syn::ItemFn>(expected).unwrap();

        assert!(parsed.items.len() == 2);
        match &parsed.items[0] {
            syn::Item::Fn(f) => {
                assert_eq!(*f, expected);
            }
            _ => panic!("not a function"),
        };
    }

    #[test]
    fn ic_query_two_args_return_value() {
        let generated = ic_query(
            quote!(),
            quote! {
                fn query(a: u32, b: u32) -> u64 {}
            },
        )
        .unwrap();
        let parsed = syn::parse2::<syn::File>(generated).unwrap();
        let fn_name = match parsed.items[0] {
            syn::Item::Fn(ref f) => &f.sig.ident,
            _ => panic!("Incorrect parsed AST."),
        };
        let expected = quote! {
            #[cfg_attr(target_family = "wasm", export_name = "canister_query query")]
            #[cfg_attr(not(target_family = "wasm"), export_name = "canister_query.query")]
            fn #fn_name() {
                ::ic_cdk::setup();
                ::ic_cdk::spawn(async {
                    let arg_bytes = ::ic_cdk::api::msg_arg_data();
                    let (a, b,) = ::candid::utils::decode_args(&arg_bytes).unwrap();
                    let result = query(a, b);
                    ::ic_cdk::api::msg_reply(::candid::utils::encode_one(result).unwrap());
                });
            }
        };
        let expected = syn::parse2::<syn::ItemFn>(expected).unwrap();

        assert!(parsed.items.len() == 2);
        match &parsed.items[0] {
            syn::Item::Fn(f) => {
                assert_eq!(*f, expected);
            }
            _ => panic!("not a function"),
        };
    }

    #[test]
    fn ic_query_export_name() {
        let generated = ic_query(
            quote!(name = "custom_query"),
            quote! {
                fn query() {}
            },
        )
        .unwrap();
        let parsed = syn::parse2::<syn::File>(generated).unwrap();
        let fn_name = match parsed.items[0] {
            syn::Item::Fn(ref f) => &f.sig.ident,
            _ => panic!("Incorrect parsed AST."),
        };

        let expected = quote! {
            #[cfg_attr(target_family = "wasm", export_name = "canister_query custom_query")]
            #[cfg_attr(not(target_family = "wasm"), export_name = "canister_query.custom_query")]
            fn #fn_name() {
                ::ic_cdk::setup();
                ::ic_cdk::spawn(async {
                    let arg_bytes = ::ic_cdk::api::msg_arg_data();
                    let () = ::candid::utils::decode_args(&arg_bytes).unwrap();
                    let result = query();
                    ::ic_cdk::api::msg_reply(::candid::utils::encode_one(()).unwrap());
                });
            }
        };
        let expected = syn::parse2::<syn::ItemFn>(expected).unwrap();

        assert!(parsed.items.len() == 2);
        match &parsed.items[0] {
            syn::Item::Fn(f) => {
                assert_eq!(*f, expected);
            }
            _ => panic!("not a function"),
        };
    }
}
