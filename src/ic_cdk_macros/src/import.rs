use crate::error::Errors;
use quote::quote;
use serde::Deserialize;
use serde_tokenstream::from_tokenstream;
use std::path::PathBuf;
use std::str::FromStr;

#[derive(Default, Deserialize)]
struct ImportAttributes {
    pub canister: Option<String>,
    pub canister_id: Option<String>,
    pub candid_path: Option<PathBuf>,
}

fn get_env_id_and_candid(canister_name: &str) -> Result<(String, PathBuf), Errors> {
    let canister_id_var_name = format!("CANISTER_ID_{}", canister_name);
    let candid_path_var_name = format!("CANISTER_CANDID_{}", canister_name);

    Ok((
        std::env::var(canister_id_var_name).map_err(|_| {
            Errors::message(&format!(
                "Could not find DFX bindings for canister named '{}'. Did you build using DFX?",
                canister_name
            ))
        })?,
        std::env::var_os(candid_path_var_name)
            .ok_or_else(|| Errors::message("Could not find DFX bindings."))
            .map(|p| PathBuf::from(p))?,
    ))
}

struct RustLanguageBinding {
    visibility: String,
    canister_id: String,
}

impl candid::codegen::rust::RustBindings for RustLanguageBinding {
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
        Ok(format!(
            r#"
                #[derive(Clone, Debug, Default, candid_derive::CandidType, serde::Deserialize)]
                pub struct {} {{ {} }}
            "#,
            id, all_fields
        ))
    }

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
        return_type: &str,
        _is_query: bool,
    ) -> Result<String, candid::error::Error> {
        let canister_id = &self.canister_id;

        let arguments = if arguments.is_empty() {
            "Option::<()>::None".to_string()
        } else if arguments.len() == 1 {
            format!("Some({})", arguments[0].0)
        } else {
            format!(
                "Some(({}))",
                arguments
                    .iter()
                    .map(|(name, _)| name.clone())
                    .collect::<Vec<String>>()
                    .join(",")
            )
        };

        let call = if return_type.is_empty() {
            "ic_cdk::call_no_return"
        } else {
            "ic_cdk::call"
        };

        Ok(format!(
            r#"
            {{
                {call}(
                  ic_cdk::CanisterId::from_str_unchecked("{canister_id}").unwrap(),
                  "{name}",
                  {arguments}
                 )
                 .await
                 .unwrap()
            }}
        "#,
            call = call,
            canister_id = canister_id,
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

        // Add Future binding.
        let return_type = if returns.is_empty() {
            "".to_string()
        } else if returns.len() == 1 {
            returns[0].clone()
        } else {
            format!("( {} )", returns.join(" , "))
        };

        let arguments_list = arguments
            .iter()
            .map(|(name, ty)| format!("{} : {}", name, ty))
            .collect::<Vec<String>>()
            .join(" , ");

        let body = self.actor_function_body(name, arguments, &return_type, is_query)?;

        Ok(format!(
            "async fn {id}( {arguments} ) {return_type} {body}",
            id = id,
            arguments = arguments_list,
            body = body,
            return_type = if return_type == "" {
                format!("")
            } else {
                format!(" -> {}", return_type)
            }
        ))
    }
}

pub(crate) fn ic_import(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> Result<proc_macro::TokenStream, Errors> {
    let config = match from_tokenstream::<ImportAttributes>(&proc_macro2::TokenStream::from(attr)) {
        Ok(c) => c,
        Err(err) => return Err(Errors::message(format!("{}", err.to_compile_error()))),
    };

    // We expect both fields to have values for now.
    let (canister_id, candid_path) = {
        if let Some(canister_name) = config.canister {
            get_env_id_and_candid(&canister_name)?
        } else if let Some(canister_id) = config.canister_id {
            if let Some(candid_path) = config.candid_path {
                (canister_id, candid_path)
            } else {
                return Err(Errors::message("Must specify both candid and canister_id."));
            }
        } else {
            return Err(Errors::message("Must specify both candid and canister_id."));
        }
    };

    let item =
        syn::parse2::<syn::Item>(proc_macro2::TokenStream::from(item)).map_err(Errors::from)?;

    // Validate that the item is a struct.
    let item = match item {
        syn::Item::Struct(item) => item,
        _ => return Err(Errors::message("import must be used on a struct.")),
    };

    let visibility = {
        let vis = item.vis;
        format!("{}", quote! { #vis })
    };
    let struct_name = item.ident.to_string();

    let candid_str = std::fs::read_to_string(&candid_path).unwrap();
    let prog = candid::IDLProg::from_str(&candid_str)
        .map_err(|e| Errors::message(format!("Could not parse the candid file: {}", e)))?;

    let bindings = Box::new(RustLanguageBinding {
        visibility,
        canister_id: canister_id.clone(),
    });

    let config = candid::codegen::rust::Config::default()
        .with_actor_name(struct_name)
        .with_biguint_type("u64".to_string())
        .with_bigint_type("i64".to_string())
        .with_bindings(bindings);

    let rust_str =
        candid::codegen::idl_to_rust(&prog, &config).map_err(|e| Errors::message(e.to_string()))?;

    let rust_str = format!("{} {}", "type principal = Vec<u8>;", rust_str);

    Ok(proc_macro::TokenStream::from_str(&rust_str).unwrap())
}
