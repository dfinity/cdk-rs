use proc_macro2::{Span, TokenStream};
use quote::quote;
use serde::Deserialize;
use serde_tokenstream::from_tokenstream;
use std::path::PathBuf;
use std::str::FromStr;
use syn::Error;

#[derive(Default, Deserialize)]
struct ImportAttributes {
    pub canister: Option<String>,
    pub canister_id: Option<String>,
    pub candid_path: Option<PathBuf>,
}

fn get_env_id_and_candid(canister_name: &str) -> Result<(String, PathBuf), Error> {
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

struct RustLanguageBinding {
    visibility: String,
    canister_id: String,
}

impl candid::codegen::rust::RustBindings for RustLanguageBinding {
    fn actor(&self, name: &str, all_functions: &[String]) -> Result<String, candid::error::Error> {
        let mut all_functions_str = String::new();
        for f in all_functions {
            all_functions_str += f;
        }

        Ok(format!(
            r#"{vis} struct {name} {{ }}
                impl {name} {{
                    {functions}
                }}"#,
            vis = self.visibility,
            name = name,
            functions = all_functions_str
        ))
    }

    fn actor_function_body(
        &self,
        name: &str,
        arguments: &[(String, String)],
        _returns: &str,
        _is_query: bool,
    ) -> Result<String, candid::error::Error> {
        let canister_id = &self.canister_id;

        let arguments = if arguments.is_empty() {
            "()".to_string()
        } else {
            format!(
                "({},)",
                arguments
                    .iter()
                    .map(|(name, _)| name.clone())
                    .collect::<Vec<String>>()
                    .join(",")
            )
        };

        let call = "ic_cdk::call";

        // We check the validity of the canister_id early so it fails if the
        // ID isn't in the right text format.
        let principal: ic_cdk::export::Principal =
            ic_cdk::export::Principal::from_text(canister_id).unwrap();

        Ok(format!(
            r#"
            {{
                {call}(
                  ic_cdk::export::Principal::from_text("{principal}").unwrap() as ic_cdk::export::Principal,
                  "{name}",
                  {arguments}
                 )
                 .await
                 .unwrap()
            }}
        "#,
            call = call,
            principal = &principal.to_text(),
            name = name.escape_debug(),
            arguments = arguments,
        ))
    }

    fn actor_function(
        &self,
        name: &str,
        arguments: &[(String, String)],
        returns: &[String],
        is_query: bool,
    ) -> Result<String, candid::error::Error> {
        let id = candid::codegen::rust::candid_id_to_rust(name);

        let arguments_list = arguments
            .iter()
            .map(|(name, ty)| format!("{} : {}", name, ty))
            .collect::<Vec<String>>()
            .join(" , ");

        let body = self.actor_function_body(name, arguments, &returns.join(","), is_query)?;

        Ok(format!(
            "async fn {id}( {arguments} ) {return_type} {body}",
            id = id,
            arguments = arguments_list,
            body = body,
            return_type = if returns.is_empty() {
                String::new()
            } else {
                format!("-> ({},)", returns.to_vec().join(","))
            }
        ))
    }

    fn record(
        &self,
        id: &str,
        fields: &[(String, String)],
    ) -> Result<String, candid::error::Error> {
        let all_fields = fields
            .iter()
            .map(|(name, ty)| format!("pub {} : {}", name, ty))
            .collect::<Vec<String>>()
            .join(" , ");
        // The following #[serde(crate = ...)] line was from https://github.com/serde-rs/serde/issues/1465#issuecomment-800686252
        // It is necessary when use re-exported serde
        Ok(format!(
            r#"
                #[derive(Clone, Debug, Default, ic_cdk::export::candid::CandidType, ic_cdk::export::serde::Deserialize)]
                #[serde(crate = "ic_cdk::export::serde")]
                pub struct {} {{ {} }}
            "#,
            id, all_fields
        ))
    }
}

pub(crate) fn ic_import(attr: TokenStream, item: TokenStream) -> Result<TokenStream, Error> {
    let config = from_tokenstream::<ImportAttributes>(&attr)?;

    // We expect both fields to have values for now.
    let (canister_id, candid_path) = {
        if let Some(canister_name) = config.canister {
            get_env_id_and_candid(&canister_name)?
        } else if let Some(canister_id) = config.canister_id {
            if let Some(candid_path) = config.candid_path {
                (canister_id, candid_path)
            } else {
                return Err(Error::new(
                    Span::call_site(),
                    "Must specify both candid and canister_id.",
                ));
            }
        } else {
            return Err(Error::new(
                Span::call_site(),
                "Must specify both candid and canister_id.",
            ));
        }
    };

    let item = syn::parse2::<syn::Item>(item)?;

    // Validate that the item is a struct.
    let item = match item {
        syn::Item::Struct(item) => item,
        _ => {
            return Err(Error::new(
                Span::call_site(),
                "import must be used on a struct.",
            ))
        }
    };

    let visibility = {
        let vis = item.vis;
        format!("{}", quote! { #vis })
    };
    let struct_name = item.ident.to_string();

    let candid_str = std::fs::read_to_string(&candid_path).unwrap();
    let prog = candid::IDLProg::from_str(&candid_str).map_err(|e| {
        Error::new(
            Span::call_site(),
            format!("Could not parse the candid file: {}", e),
        )
    })?;

    let bindings = Box::new(RustLanguageBinding {
        visibility,
        canister_id,
    });

    let config = candid::codegen::rust::Config::default()
        .with_actor_name(struct_name)
        .with_biguint_type("candid::Nat".to_string())
        .with_bigint_type("candid::Int".to_string())
        .with_bindings(bindings);

    let rust_str = candid::codegen::idl_to_rust(&prog, &config)
        .map_err(|e| Error::new(Span::call_site(), e.to_string()))?;

    let rust_str = format!("{} {}", "type principal = Vec<u8>;", rust_str);

    Ok(TokenStream::from_str(&rust_str).unwrap())
}
