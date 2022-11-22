use quote::quote;
use syn::parse::{Parse, ParseStream, Result};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::token::Comma;
use syn::{parenthesized, Error};
use syn::{FnArg, Ident, Token, TypePath};

use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;

#[derive(Clone, Debug)]
pub struct SystemAPI {
    pub name: Ident,
    pub args: Vec<FnArg>,
    pub output: Option<TypePath>,
}

impl Parse for SystemAPI {
    fn parse(input: ParseStream) -> Result<Self> {
        let ic0_token: Ident = input.parse()?;
        if ic0_token != "ic0" {
            return Err(Error::new(ic0_token.span(), "expected `ic0`"));
        }
        input.parse::<Token![.]>()?;
        let name: Ident = input.parse()?;
        input.parse::<Token![:]>()?;

        // args
        let content;
        parenthesized!(content in input);
        let args = Punctuated::<FnArg, Comma>::parse_terminated(&content)?;
        let args: Vec<FnArg> = args.iter().cloned().collect();
        for arg in &args {
            match arg {
                FnArg::Receiver(r) => return Err(Error::new(r.span(), "receiver not expected")),
                FnArg::Typed(pat_type) => match &*pat_type.ty {
                    syn::Type::Path(ty) => {
                        type_supported(ty)?;
                    }
                    _ => return Err(Error::new(pat_type.span(), "expected type as i32")),
                },
            }
        }

        input.parse::<Token![->]>()?;

        // output
        let output = if input.peek(syn::token::Paren) {
            let content;
            parenthesized!(content in input);
            if content.is_empty() {
                None
            } else {
                let _output_name: Ident = content.parse()?;
                content.parse::<Token![:]>()?;
                let ty: TypePath = content.parse()?;
                if !content.is_empty() {
                    return Err(Error::new(ty.span(), "expected only one return type"));
                }
                type_supported(&ty)?;
                Some(ty)
            }
        } else {
            let ty: TypePath = input.parse()?;
            type_supported(&ty)?;
            Some(ty)
        };

        input.parse::<Token![;]>()?;

        Ok(Self { name, args, output })
    }
}

fn type_supported(ty: &TypePath) -> Result<()> {
    let supported = match ty.path.get_ident() {
        Some(i) => i == "i32" || i == "i64",
        None => false,
    };
    match supported {
        true => Ok(()),
        false => Err(Error::new(ty.span(), "expected i32 or i64")),
    }
}

#[derive(Clone, Debug)]
pub struct IC0 {
    pub apis: Vec<SystemAPI>,
}

impl Parse for IC0 {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            apis: {
                let mut apis = vec![];
                while !input.is_empty() {
                    apis.push(input.parse()?);
                }
                apis
            },
        })
    }
}

fn main() {
    let s = include_str!("../ic0.txt");
    let ic0: IC0 = syn::parse_str(s).unwrap();

    let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    d.push("src/ic0.rs");

    let mut f = fs::File::create(d).unwrap();

    writeln!(
        f,
        r#"// This file is generated from ic0.txt.
// Don't manually modify it.
#[cfg(target_arch = "wasm32")]
#[link(wasm_import_module = "ic0")]
extern "C" {{"#,
    )
    .unwrap();

    for api in &ic0.apis {
        let fn_name = &api.name;
        let args = &api.args;

        let mut r = quote! {
            pub fn #fn_name(#(#args),*)
        };

        if let Some(output) = &api.output {
            r = quote! {
                #r -> #output
            }
        }

        r = quote! {#r;};
        writeln!(f, "{}", r).unwrap();
    }

    writeln!(f, "}}").unwrap();

    writeln!(
        f,
        r#"
#[cfg(not(target_arch = "wasm32"))]
#[allow(unused_variables)]
#[allow(clippy::missing_safety_doc)]
#[allow(clippy::too_many_arguments)]
mod non_wasm{{"#,
    )
    .unwrap();

    for api in &ic0.apis {
        let fn_name = &api.name;
        let args = &api.args;

        let mut r = quote! {
            pub unsafe fn #fn_name(#(#args),*)
        };

        if let Some(output) = &api.output {
            r = quote! {
                #r -> #output
            }
        }

        let panic_str = format!("{} should only be called inside canisters.", fn_name);

        r = quote! {
        #r {
            panic!(#panic_str);
        }};
        writeln!(f, "{}", r).unwrap();
    }

    writeln!(
        f,
        r#"}}

#[cfg(not(target_arch = "wasm32"))]
pub use non_wasm::*;
"#
    )
    .unwrap();

    Command::new("cargo")
        .args(["fmt"])
        .output()
        .expect("`cargo fmt` failed");
}
