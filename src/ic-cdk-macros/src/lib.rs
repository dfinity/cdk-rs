use proc_macro::TokenStream;
use std::sync::atomic::{AtomicU32, Ordering};
use syn::Error;

mod export;
mod import;

static NEXT_ID: AtomicU32 = AtomicU32::new(0);
pub(crate) fn id() -> u32 {
    NEXT_ID.fetch_add(1, Ordering::SeqCst)
}

fn handle_debug_and_errors<F>(
    cb: F,
    name: &str,
    attr: TokenStream,
    item: TokenStream,
) -> TokenStream
where
    F: FnOnce(TokenStream, TokenStream) -> Result<TokenStream, Error>,
{
    if std::env::var_os("IC_CDK_DEBUG").is_some() {
        eprintln!("--- IC_CDK_MACROS DEBUG ---");
        eprintln!("{}\n    attr: {}\n    item: {}", name, attr, item);
    }

    let result = cb(attr, item);

    if std::env::var_os("IC_CDK_DEBUG").is_some() {
        eprintln!("--------- RESULT  ---------");
        if let Ok(ref stream) = result {
            eprintln!("{}", stream);
        }
        eprintln!("---------------------------");
    }

    result.map_or_else(|e| e.to_compile_error().into(), Into::into)
}

/// Create a query call endpoint.
#[proc_macro_attribute]
pub fn query(attr: TokenStream, item: TokenStream) -> TokenStream {
    handle_debug_and_errors(export::ic_query, "ic_query", attr, item)
}

/// Create an update call endpoint.
#[proc_macro_attribute]
pub fn update(attr: TokenStream, item: TokenStream) -> TokenStream {
    handle_debug_and_errors(export::ic_update, "ic_update", attr, item)
}

#[proc_macro_attribute]
pub fn init(attr: TokenStream, item: TokenStream) -> TokenStream {
    handle_debug_and_errors(export::ic_init, "ic_init", attr, item)
}

#[proc_macro_attribute]
pub fn pre_upgrade(attr: TokenStream, item: TokenStream) -> TokenStream {
    handle_debug_and_errors(export::ic_pre_upgrade, "ic_pre_upgrade", attr, item)
}

#[proc_macro_attribute]
pub fn post_upgrade(attr: TokenStream, item: TokenStream) -> TokenStream {
    handle_debug_and_errors(export::ic_post_upgrade, "ic_post_upgrade", attr, item)
}

#[proc_macro_attribute]
pub fn import(attr: TokenStream, item: TokenStream) -> TokenStream {
    handle_debug_and_errors(import::ic_import, "ic_import", attr, item)
}

#[proc_macro]
pub fn export_candid(_input: TokenStream) -> TokenStream {
    let methods = export::METHODS.lock().unwrap().take();
    if let Some(methods) = methods {
        let methods: Vec<_> = methods
            .iter()
            .map(|m| syn::parse_str::<proc_macro2::TokenStream>(m).unwrap())
            .collect();
        quote::quote! {
            #[ic_cdk_macros::query(name = "__get_candid_interface_tmp_hack")]
            fn export_candid() -> String {
                let mut service = Vec::new();
                #(
                    service.push(#methods());
                )*
                let ty = ::candid::types::Type::Service(service);
                let env = ::candid::TypeEnv::new();
                let actor = Some(ty);
                let result = ::candid::bindings::candid::compile(&env, &actor);
                format!("{}", result)
            }
            #[cfg(feature = "export_candid")]
            #[no_mangle]
            pub unsafe extern "C" fn _start() {
                let result = export_candid();
                let ret = unsafe { ::ic_cdk::api::wasi::print(&result) };
                ic_cdk::api::wasi::proc_exit(ret as u32);
            }
        }
    } else {
        panic!("export_candid called twice")
    }
    .into()
}
