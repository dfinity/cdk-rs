use candid::Principal;
use candid_parser::bindings::rust::{Config, ExternalConfig, emit_bindgen, output_handlebar};
use candid_parser::configs::Configs;
use candid_parser::pretty_check_file;

use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::str::FromStr;

/// Generate bindings for a callee canister, the canister ID of it is static (known at compile time).
pub fn static_callee<S>(canister_name: &str, candid_path: S, canister_id: Principal)
where
    S: Into<PathBuf>,
{
    // 1. Parse the candid file and generate the Output (the struct for bindings)
    let config = Config::new(Configs::from_str("").unwrap());
    let candid_path = candid_path.into();
    let (env, actor, prog) = pretty_check_file(&candid_path).unwrap_or_else(|e| {
        panic!(
            "failed to parse candid file ({}): {}",
            candid_path.display(),
            e
        )
    });
    // unused are not handled
    let (output, _unused) = emit_bindgen(&config, &env, &actor, &prog);

    // 2. Generate the Rust bindings using the Handlebars template
    let template = include_str!("templates/static_callee.hbs");
    let mut external = ExternalConfig::default();
    external
        .0
        .insert("canister_id".to_string(), canister_id.to_string());
    let content = output_handlebar(output, external, template);

    // 3. Write the generated Rust bindings to the output directory
    let out_dir_str = std::env::var("OUT_DIR")
        .expect("OUT_DIR should always be set when execute the build.rs script");
    let out_dir = PathBuf::from(out_dir_str);
    let generated_path = out_dir.join(format!("{}.rs", canister_name));
    let mut file = fs::File::create(&generated_path).unwrap_or_else(|e| {
        panic!(
            "failed to create the output file ({}): {}",
            generated_path.display(),
            e
        )
    });
    writeln!(file, "{content}").unwrap_or_else(|e| {
        panic!(
            "failed to write to the output file ({}): {}",
            generated_path.display(),
            e
        )
    });
}
