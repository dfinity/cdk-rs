use candid::parser::types::Dec;
use candid::parser::types::IDLType;
use proc_macro2::{Span, TokenStream};
use quote::quote;
use serde::Deserialize;
use serde_tokenstream::from_tokenstream;
use std::env;
use std::path::PathBuf;
use std::str::FromStr;
use syn::Error;
use syn::Result;
use syn::Token;
use ic_cdk_codegen::Processor;

#[derive(Default, Deserialize)]
struct ImportAttributes {
    pub canister: Option<String>,
    pub canister_id: Option<String>,
    pub candid_path: Option<PathBuf>,
}

fn get_env_id_and_candid(canister_name: &str) -> Result<(String, PathBuf)> {
    let canister_id_var_name = format!("CANISTER_ID_{}", canister_name);
    let candid_path_var_name = format!("CANISTER_CANDID_{}", canister_name);

    Ok((
        std::env::var(canister_id_var_name).map_err(|_| {
            Error::new(
                Span::call_site(),
                &format!(
                    "Could not find DFX bindings for canister named '{}'. Did you build using DFX?",
                    canister_name
                ),
            )
        })?,
        std::env::var_os(candid_path_var_name)
            .ok_or_else(|| Error::new(Span::call_site(), "Could not find DFX bindings."))
            .map(PathBuf::from)?,
    ))
}

pub(crate) fn ic_import(attr: TokenStream, item: TokenStream) -> Result<TokenStream> {
    let config = from_tokenstream::<ImportAttributes>(&attr)?;

    // We expect both fields to have values for now.
    let (canister_id, candid_path) = {
        if let Some(canister_name) = config.canister {
            get_env_id_and_candid(&canister_name)?
        } else if let Some(canister_id) = config.canister_id {
            if let Some(candid_path) = config.candid_path {
                // CWD of a proc macro is the workspace dir, not the package dir
                let qualified_path = if candid_path.exists() {
                    candid_path
                } else {
                    env::var_os("CARGO_MANIFEST_DIR")
                        .map(PathBuf::from)
                        .unwrap_or_default()
                        .join(candid_path)
                };
                (canister_id, qualified_path)
            } else {
                return Err(Error::new(
                    Span::call_site(),
                    "Must specify both candid_path and canister_id.",
                ));
            }
        } else {
            return Err(Error::new(
                Span::call_site(),
                "Must specify both candid_path and canister_id.",
            ));
        }
    };

    let principal = canister_id
        .parse()
        .map_err(|e| Error::new(Span::call_site(), e))?;

    let item = syn::parse2::<syn::Item>(item)?;

    // Validate that the item is a module.
    let item = match item {
        syn::Item::Mod(item) => item,
        // Compat
        syn::Item::Struct(syn::ItemStruct {
            attrs,
            vis,
            ident,
            fields: syn::Fields::Unit,
            semi_token: Some(semi),
            struct_token,
            ..
        }) => syn::ItemMod {
            attrs,
            vis,
            ident,
            content: None,
            semi: Some(semi),
            mod_token: Token![mod](struct_token.span),
        },
        _ => {
            return Err(Error::new(
                Span::call_site(),
                "#[import] must be used on a module.",
            ))
        }
    };

    let vis = item.vis;
    let mod_name = item.ident;

    let candid_str = std::fs::read_to_string(&candid_path)
        .map_err(|e| Error::new(Span::call_site(), format!("{}", e)))?;
    let prog = candid::IDLProg::from_str(&candid_str).map_err(|e| {
        Error::new(
            Span::call_site(),
            format!("Could not parse the candid file: {}", e),
        )
    })?;
    let mut processor = Processor::new(item.content.map(|(_, body)| body).unwrap_or_default())?;
    for decl in prog.decs {
        if let Dec::TypD(decl) = decl {
            processor.add_decl(decl)?;
        }
    }
    if let Some(actor) = prog.actor {
        if let IDLType::ServT(bindings) = actor {
            processor.add_primary_actor(bindings)?;
        }
    }

    let mod_body = processor.generate(principal)?;
    Ok(quote! {
        #[allow(nonstandard_style)]
        #vis mod #mod_name {
            #mod_body
        }
    })
}
