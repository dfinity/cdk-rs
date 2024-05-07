use codespan_reporting::{
    diagnostic::{Diagnostic, Label},
    files::SimpleFile,
    term::{self, termcolor::StandardStream},
};
use syn::{Expr, ExprLit, FnArg, Lit, Meta, ReturnType, Attribute, Signature};
use candid_parser::bindings::rust::{Output, Config, emit_bindgen};
use candid_parser::configs::Configs;
use candid_parser::{Result, utils::CandidSource};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::fs;
use std::ops::Range;

pub fn check_rust(rust: &Path, candid: &Path, config: &Option<PathBuf>) -> Result<()> {
    let candid = CandidSource::File(candid);
    let (env, actor) = candid.load()?;
    let config = if let Some(config) = config {
        fs::read_to_string(config)?
    } else {
        "".to_string()
    };
    let config: Configs = config.parse()?;
    let config = Config::new(config);
    let (output, unused) = emit_bindgen(&config, &env, &actor);
    let name = rust.file_name().unwrap().to_str().unwrap();
    let source = fs::read_to_string(rust)?;
    report_errors(&name, &source, &output);
    Ok(())
}

fn report_errors(name: &str, source: &str, candid: &Output) {
    let rust = get_endpoint_from_rust_source(source);
    let diags = diff_did_and_rust(candid, &rust);
    let writer = StandardStream::stderr(term::termcolor::ColorChoice::Auto);
    let config = term::Config::default();
    let file = SimpleFile::new(name, source);
    for diag in diags {
        term::emit(&mut writer.lock(), &config, &file, &diag).unwrap();
    }
}
fn get_endpoint_from_rust_source(source: &str) -> Vec<CDKMethod> {
    use syn::visit::{self, Visit};
    use syn::{ImplItemFn, ItemFn};
    struct FnVisitor(Vec<CDKMethod>);
    impl<'ast> Visit<'ast> for FnVisitor {
        fn visit_item_fn(&mut self, node: &'ast ItemFn) {
            if let Some(m) = get_cdk_function(&node.attrs, &node.sig) {
                self.0.push(m);
            }
            // handle nested functions
            visit::visit_item_fn(self, node);
        }
        fn visit_impl_item_fn(&mut self, node: &'ast ImplItemFn) {
            if let Some(m) = get_cdk_function(&node.attrs, &node.sig) {
                self.0.push(m);
            }
            // handle nested functions
            visit::visit_impl_item_fn(self, node);
        }
    }
    let ast = syn::parse_file(source).unwrap();
    let mut visitor = FnVisitor(Vec::new());
    visitor.visit_file(&ast);
    for m in &visitor.0 {
        m.debug_print(source);
    }
    visitor.0
}
fn diff_did_and_rust(candid: &Output, rust: &[CDKMethod]) -> Vec<Diagnostic<()>> {
    use syn::spanned::Spanned;
    let mut res = Vec::new();
    let rust: BTreeMap<_, _> = rust
        .iter()
        .map(|m| {
            let name = m
                .export_name
                .as_ref()
                .map(|x| x.0.clone())
                .unwrap_or(m.func_name.to_string());
            (name, m)
        })
        .collect();
    for m in &candid.methods {
        let diag = Diagnostic::error()
            .with_message(format!("Error with Candid method {}", m.original_name));
        let mut labels = Vec::new();
        let mut notes = Vec::new();
        if let Some(func) = rust.get(&m.original_name) {
            if m.original_name == m.name {
            } else {
                if let Some((name, meta)) = &func.export_name {
                    if *name != m.original_name {
                        labels
                            .push(Label::primary((), meta.span().byte_range()).with_message(
                                format!("expect {}", m.original_name.escape_debug()),
                            ));
                    }
                } else {
                    labels.push(
                        Label::primary((), func.mode.span().byte_range()).with_message(format!(
                            "missing (name = \"{}\")",
                            m.original_name.escape_debug()
                        )),
                    );
                }
            }
            let args = func.args.iter().zip(m.args.iter().map(|x| &x.1));
            for (rust_arg, candid_arg) in args {
                let parsed_candid_arg: syn::Type = syn::parse_str(candid_arg).unwrap();
                if parsed_candid_arg != *rust_arg {
                    labels.push(
                        Label::primary((), rust_arg.span().byte_range())
                            .with_message(format!("expect type: {}", candid_arg)),
                    );
                }
            }
        } else {
            notes.push(format!(
                "method \"{}\" missing from Rust code",
                m.original_name
            ));
        }
        if labels.is_empty() && notes.is_empty() {
            continue;
        }
        res.push(diag.with_labels(labels).with_notes(notes));
    }
    res
}
struct CDKMethod {
    func_name: syn::Ident,
    export_name: Option<(String, syn::Meta)>,
    composite: Option<syn::Meta>,
    mode: syn::Ident,
    args: Vec<syn::Type>,
    rets: Vec<syn::Type>,
    attr_span: Range<usize>,
    fn_span: Range<usize>,
    args_span: Range<usize>,
    rets_span: Range<usize>,
}
fn get_cdk_function(attrs: &[Attribute], sig: &Signature) -> Option<CDKMethod> {
    use syn::parse::Parser;
    use syn::spanned::Spanned;
    let func_name = sig.ident.clone();
    let mut mode = None;
    let mut export_name = None;
    let mut composite = None;
    let mut attr_span = None;
    let mut fn_span = None;
    for attr in attrs {
        let attr_name = &attr.meta.path().segments.last().unwrap().ident;
        if attr_name != "update" && attr_name != "query" && attr_name != "init" {
            continue;
        }
        mode = Some(attr_name.clone());
        attr_span = Some(attr.span().byte_range());
        fn_span = Some(sig.span().byte_range());
        if let Meta::List(list) = &attr.meta {
            let nested = syn::punctuated::Punctuated::<Meta, syn::Token![,]>::parse_terminated
                .parse2(list.tokens.clone())
                .unwrap();
            for meta in nested {
                if let Meta::NameValue(ref m) = meta {
                    if m.path.is_ident("name") {
                        if let Expr::Lit(ExprLit {
                            lit: Lit::Str(name),
                            ..
                        }) = &m.value
                        {
                            export_name = Some((name.value(), meta));
                        }
                    } else if m.path.is_ident("composite") {
                        if let Expr::Lit(ExprLit {
                            lit: Lit::Bool(b), ..
                        }) = &m.value
                        {
                            if b.value {
                                composite = Some(meta);
                            }
                        }
                    }
                }
            }
        }
    }
    let args = sig
        .inputs
        .iter()
        .filter_map(|arg| match arg {
            FnArg::Receiver(_) => None,
            FnArg::Typed(pat) => Some(*pat.ty.clone()),
        })
        .collect();
    let rets = match &sig.output {
        ReturnType::Default => Vec::new(),
        ReturnType::Type(_, ty) => match ty.as_ref() {
            syn::Type::Tuple(ty) => ty.elems.iter().map(|t| (*t).clone()).collect(),
            _ => vec![*ty.clone()],
        },
    };
    mode.map(|mode| CDKMethod {
        func_name,
        export_name,
        composite,
        args,
        rets,
        mode,
        attr_span: attr_span.unwrap(),
        fn_span: fn_span.unwrap(),
        args_span: sig.inputs.span().byte_range(),
        rets_span: sig.output.span().byte_range(),
    })
}
impl CDKMethod {
    fn debug_print(&self, source: &str) {
        use syn::spanned::Spanned;
        println!("{} {}", self.func_name, self.mode);
        if let Some((_, meta)) = &self.export_name {
            let range = meta.span().byte_range();
            println!(" export {}", &source[range]);
        }
        if let Some(composite) = &self.composite {
            let range = composite.span().byte_range();
            println!(" composite {}", &source[range]);
        }
        for arg in &self.args {
            let range = arg.span().byte_range();
            println!(" arg {}", &source[range]);
        }
        for ret in &self.rets {
            let range = ret.span().byte_range();
            println!(" ret {}", &source[range]);
        }
    }
}
