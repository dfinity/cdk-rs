use crate::error::Errors;
use quote::quote;
use std::convert::TryFrom;
use std::path::PathBuf;
use std::str::FromStr;
use syn::{AttributeArgs, Lit, Meta, MetaNameValue, NestedMeta};

#[derive(Default)]
struct ImportAttributes {
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

fn parse_meta_attr(meta: Meta) -> Result<(Option<String>, Option<PathBuf>), Errors> {
    match meta.path().get_ident() {
        Some(id) if id == &crate::symbols::CANISTER_ID => {
            if let Meta::NameValue(MetaNameValue { ref lit, .. }) = meta {
                match lit {
                    Lit::Str(value) => Ok((Some(value.value()), None)),
                    _ => Err(Errors::message(format!(
                        "Argument {} requires a string value.",
                        id
                    ))),
                }
            } else {
                return Err(Errors::message(format!(
                    "Argument {} requires a value.",
                    id
                )));
            }
        }
        Some(id) if id == &crate::symbols::CANDID_PATH => {
            if let Meta::NameValue(MetaNameValue { ref lit, .. }) = meta {
                match lit {
                    Lit::Str(value) => Ok((None, Some(PathBuf::from(value.value())))),
                    _ => Err(Errors::message(format!(
                        "Argument {} requires a string value.",
                        id
                    ))),
                }
            } else {
                return Err(Errors::message(format!(
                    "Argument {} requires a value.",
                    id
                )));
            }
        }
        Some(id) if id == &crate::symbols::CANISTER => {
            if let Meta::NameValue(MetaNameValue { ref lit, .. }) = meta {
                match lit {
                    Lit::Str(value) => {
                        let id = value.value();
                        let (canister_id, candid_path) = get_env_id_and_candid(&id)?;

                        Ok((Some(canister_id), Some(candid_path)))
                    }
                    _ => Err(Errors::message(format!(
                        "Argument {} requires a string value.",
                        id
                    ))),
                }
            } else {
                return Err(Errors::message(format!(
                    "Argument {} requires a value.",
                    id
                )));
            }
        }
        Some(other) => {
            // This is a special shorthand for using a local DFX canister.
            // Check if the CANISTER_ID and CANDID_PATH environment variables
            // for this canister are set.
            let id = format!("{}", other);
            let (canister_id, candid_path) = get_env_id_and_candid(&id)?;

            Ok((Some(canister_id), Some(candid_path)))
        }
        None => return Err(Errors::message("Must specify an identifier.")),
    }
}

impl TryFrom<syn::AttributeArgs> for ImportAttributes {
    type Error = Errors;

    fn try_from(args: AttributeArgs) -> Result<Self, Self::Error> {
        let mut attrs = ImportAttributes::default();
        for arg in args {
            match arg {
                NestedMeta::Meta(meta) => {
                    let (canister_id, candid_path) = parse_meta_attr(meta)?;
                    if let Some(cid) = canister_id {
                        attrs.canister_id = Some(cid);
                    }
                    if let Some(cpath) = candid_path {
                        attrs.candid_path = Some(cpath);
                    }
                }
                _ => {
                    return Err(Errors::message(
                        "Arguments must be tagged (no direct values).",
                    ))
                }
            }
        }
        Ok(attrs)
    }
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
    let attr: syn::AttributeArgs = syn::parse_macro_input::parse::<syn::AttributeArgs>(attr)?;

    let ImportAttributes {
        canister_id,
        candid_path,
    } = ImportAttributes::try_from(attr)?;

    // We expect both fields to have values for now.
    let canister_id = canister_id.unwrap();
    let candid_path = candid_path.unwrap();

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

    Ok(proc_macro::TokenStream::from_str(&rust_str).unwrap())
}
