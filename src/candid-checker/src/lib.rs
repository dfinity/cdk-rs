use candid_parser::bindings::rust::{emit_bindgen, Config, Method, Output};
use candid_parser::configs::Configs;
use candid_parser::{utils::CandidSource, Result};
use codespan_reporting::{
    diagnostic::{Diagnostic, Label},
    files::SimpleFile,
    term::{self, termcolor::StandardStream},
};
use std::collections::BTreeMap;
use std::fs;
use std::ops::Range;
use std::path::{Path, PathBuf};
use syn::spanned::Spanned;
use syn::{Attribute, Expr, ExprLit, FnArg, Lit, Meta, ReturnType, Signature};

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
    report_unused(unused);
    let name = rust.file_name().unwrap().to_str().unwrap();
    let source = fs::read_to_string(rust)?;
    let rust = get_endpoint_from_rust_source(&source);
    let diags = diff_did_and_rust(&output, &rust);
    report_errors(name, &source, &diags);
    Ok(())
}
fn report_unused(unused: Vec<String>) {
    if !unused.is_empty() {
        let unused = unused
            .iter()
            .map(|x| format!("{x} is not used"))
            .collect::<Vec<_>>();
        let diag = Diagnostic::warning()
            .with_message("Unused paths from the config file")
            .with_notes(unused);
        report_errors("config", "", &[diag]);
    }
}
fn report_errors(name: &str, source: &str, diags: &[Diagnostic<()>]) {
    let writer = StandardStream::stderr(term::termcolor::ColorChoice::Auto);
    let config = term::Config::default();
    let file = SimpleFile::new(name, source);
    for diag in diags {
        term::emit(&mut writer.lock(), &config, &file, diag).unwrap();
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
    visitor.0
}
fn diff_did_and_rust(candid: &Output, rust_list: &[CDKMethod]) -> Vec<Diagnostic<()>> {
    let mut res = Vec::new();
    let mut ids: BTreeMap<_, _> = rust_list
        .iter()
        .map(|m| (m.func_name.to_string(), m))
        .collect();
    let rust: BTreeMap<_, _> = rust_list
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
        let diag =
            Diagnostic::error().with_message(format!("Checking Candid method {}", m.original_name));
        let mut labels = Vec::new();
        let mut notes = Vec::new();
        if let Some(func) = rust.get(&m.original_name) {
            ids.remove(&func.func_name.to_string());
            // check function name
            if func.func_name != m.name {
                labels.push(
                    Label::primary((), func.func_name.span().byte_range())
                        .with_message(format!("Expect function name: {}", m.name)),
                );
            }
            // check mode
            let mode = if m.mode == "update" {
                "update"
            } else {
                "query"
            };
            if func.mode != mode {
                labels.push(
                    Label::primary((), func.mode.span().byte_range())
                        .with_message(format!("Expect mode: {}", mode)),
                );
            }
            if m.mode == "composite_query" && func.composite.is_none() {
                labels.push(
                    Label::primary((), func.mode.span().byte_range())
                        .with_message("Expect attribute: composite = true"),
                );
            }
            // check rename attribute
            if m.original_name != m.name && func.export_name.is_none() {
                // no need to check func.export_name != m.original_name, since we already found the function
                labels.push(
                    Label::primary((), func.mode.span().byte_range()).with_message(format!(
                        "Expect attribute: name = \"{}\"",
                        m.original_name.escape_debug()
                    )),
                );
            }
            // check args
            let args = m.args.iter().map(|x| x.1.clone()).collect::<Vec<_>>();
            let (mut labs, need_pp) = check_args(&func.args, &args, &func.args_span);
            if need_pp {
                let pp = pp_args(&m.args);
                labs.push(
                    Label::secondary((), func.args_span.clone())
                        .with_message(format!("Suggestion: {}", pp)),
                );
            }
            labels.extend(labs);
            let (mut labs, need_pp) = check_args(&func.rets, &m.rets, &func.rets_span);
            if need_pp {
                let mut pp = pp_rets(&m.rets);
                if pp.is_empty() {
                    pp = "remove the return type".to_string();
                }
                labs.push(
                    Label::secondary((), func.rets_span.clone())
                        .with_message(format!("Suggestion: {}", pp)),
                );
            }
            labels.extend(labs);
            if !labels.is_empty() {
                labels.push(Label::secondary((), func.fn_span.clone()));
            }
        } else {
            if let Some(func) = ids.remove(&m.original_name) {
                let (_, meta) = func.export_name.as_ref().unwrap();
                labels.push(
                    Label::primary((), meta.span().byte_range())
                        .with_message("You may want to remove the name attribute"),
                );
                labels.push(
                    Label::secondary((), func.func_name.span().byte_range())
                        .with_message("This function name matches the Candid method name"),
                );
            } else {
                notes.push(format!(
                    "Method \"{}\" is missing from Rust code. Use this signature to get started:\n{}",
                    m.original_name, pp_func(m),
                ));
            }
        }
        if labels.is_empty() && notes.is_empty() {
            continue;
        }
        res.push(diag.with_labels(labels).with_notes(notes));
    }
    if let Some(init_args) = &candid.init_args {
        let diags = check_init_args(init_args, &mut ids);
        res.extend(diags);
    }
    for (_, func) in ids {
        let span = func.mode.span().byte_range().start..func.fn_span.end;
        let label = Label::secondary((), span);
        let diag = Diagnostic::warning()
            .with_message(format!(
                "Function {} doesn't appear in Candid file",
                func.func_name
            ))
            .with_labels(vec![label]);
        res.push(diag);
    }
    res
}
fn check_init_args(
    candid: &[(String, String)],
    rust: &mut BTreeMap<String, &CDKMethod>,
) -> Vec<Diagnostic<()>> {
    let diag = Diagnostic::error().with_message("Checking init args");
    let mut notes = Vec::new();
    let mut labels = Vec::new();
    if let Some((name, func)) = rust.iter().find(|(_, m)| m.mode == "init") {
        let args = candid.iter().map(|x| x.1.clone()).collect::<Vec<_>>();
        let (mut labs, need_pp) = check_args(&func.args, &args, &func.args_span);
        if need_pp {
            let pp = pp_args(candid);
            labs.push(
                Label::secondary((), func.args_span.clone())
                    .with_message(format!("Suggestion: {}", pp)),
            );
        }
        labels.extend(labs);
        rust.remove(&name.clone());
    } else {
        notes.push(format!(
            "Init args is missing from Rust code. Use this signature to get started:\n{}",
            pp_init_args(candid)
        ));
    }
    if notes.is_empty() && labels.is_empty() {
        Vec::new()
    } else {
        vec![diag.with_notes(notes).with_labels(labels)]
    }
}
fn check_args(
    rust: &[syn::Type],
    candid: &[String],
    span: &Range<usize>,
) -> (Vec<Label<()>>, bool) {
    let mut labels = Vec::new();
    if rust.len() != candid.len() {
        labels.push(Label::primary((), span.clone()).with_message("Argument count mismatch"));
        return (labels, true);
    }
    let args = rust.iter().zip(candid.iter());
    for (rust_arg, candid_arg) in args {
        let parsed_candid_arg: syn::Type = syn::parse_str(candid_arg).unwrap();
        if parsed_candid_arg != *rust_arg {
            labels.push(
                Label::primary((), rust_arg.span().byte_range())
                    .with_message(format!("Expect type: {}", candid_arg)),
            );
        }
    }
    (labels, false)
}
fn pp_args(args: &[(String, String)]) -> String {
    let body = args
        .iter()
        .map(|(id, ty)| format!("{id}: {ty}"))
        .collect::<Vec<_>>()
        .join(", ");
    format!("({body})")
}
fn pp_rets(rets: &[String]) -> String {
    match rets.len() {
        0 => "".to_string(),
        1 => format!("-> {}", rets[0]),
        _ => format!("-> ({})", rets.join(", ")),
    }
}
fn pp_init_args(args: &[(String, String)]) -> String {
    format!("#[init]\nfn init{}", pp_args(args))
}
fn pp_attr(m: &Method) -> String {
    let mode = if m.mode == "update" {
        "update"
    } else {
        "query"
    };
    let mut attr = Vec::new();
    if m.mode == "composite_query" {
        attr.push("composite = true".to_string());
    }
    if m.original_name != m.name {
        attr.push(format!("name = \"{}\"", m.original_name.escape_debug()));
    }
    let attr = if attr.is_empty() {
        String::new()
    } else {
        format!("({})", attr.join(", "))
    };
    format!("#[{mode}{attr}]")
}
fn pp_func(m: &Method) -> String {
    format!(
        "{}\nfn {}{} {}",
        pp_attr(m),
        m.name,
        pp_args(&m.args),
        pp_rets(&m.rets)
    )
}
struct CDKMethod {
    func_name: syn::Ident,
    export_name: Option<(String, syn::Meta)>,
    composite: Option<syn::Meta>,
    mode: syn::Ident,
    args: Vec<syn::Type>,
    rets: Vec<syn::Type>,
    fn_span: Range<usize>,
    args_span: Range<usize>,
    rets_span: Range<usize>,
}
fn get_cdk_function(attrs: &[Attribute], sig: &Signature) -> Option<CDKMethod> {
    use syn::parse::Parser;
    let func_name = sig.ident.clone();
    let mut mode = None;
    let mut export_name = None;
    let mut composite = None;
    let mut fn_span = None;
    for attr in attrs {
        let attr_name = &attr.meta.path().segments.last().unwrap().ident;
        if attr_name != "update" && attr_name != "query" && attr_name != "init" {
            continue;
        }
        mode = Some(attr_name.clone());
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
    let args_span = sig.paren_token.span;
    let args_span = args_span.open().byte_range().start..args_span.close().byte_range().end;
    let mut rets_span = sig.output.span().byte_range();
    if rets_span.end == 0 {
        rets_span = args_span.end..args_span.end;
    }
    mode.map(|mode| CDKMethod {
        func_name,
        export_name,
        composite,
        args,
        rets,
        mode,
        fn_span: fn_span.unwrap(),
        args_span,
        rets_span,
    })
}
impl CDKMethod {
    fn debug_print(&self, source: &str) {
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
