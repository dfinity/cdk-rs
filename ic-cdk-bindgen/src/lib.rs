use candid::Principal;
use candid_parser::bindings::rust::{Config, ExternalConfig, emit_bindgen, output_handlebar};
use candid_parser::configs::Configs;
use candid_parser::pretty_check_file;

use std::env;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::str::FromStr;

mod error;

pub use error::IcCdkBindgenError;

type Result<T> = std::result::Result<T, IcCdkBindgenError>;

#[derive(Debug, Clone)]
pub struct Builder {
    pub(crate) canister_name: String,
    pub(crate) canister_id: Option<Principal>,
    pub(crate) candid_path: Option<PathBuf>,
}

// Configurations.
impl Builder {
    /// Create a new builder with the given canister name.
    pub fn new<S>(canister_name: S) -> Self
    where
        S: Into<String>,
    {
        Self {
            canister_name: canister_name.into(),
            canister_id: None,
            candid_path: None,
        }
    }

    /// Set the canister id.
    ///
    /// This is only needed for the consumer target.
    ///
    /// If not set, the canister id will be resolved from the environment variable `CANISTER_ID_<CANISTER_NAME>`.
    pub fn canister_id<S>(&mut self, canister_id: S) -> &mut Self
    where
        S: Into<Principal>,
    {
        self.canister_id = Some(canister_id.into());
        self
    }

    /// Set the path to the candid file.
    ///
    /// If not set, the path will be resolved from the environment variable `CANISTER_CANDID_PATH_<CANISTER_NAME>`.
    pub fn candid_path<S>(&mut self, path: S) -> &mut Self
    where
        S: Into<PathBuf>,
    {
        self.candid_path = Some(path.into());
        self
    }
}

// Code generation.
impl Builder {
    pub fn generate(&self) -> Result<()> {
        // 1. Parse the candid file and generate the Output (the struct for bindings)
        let config = Config::new(Configs::from_str("").unwrap());
        let candid_path = if let Some(p) = &self.candid_path {
            p.clone()
        } else {
            candid_path_from_env(&self.canister_name)?
        };

        let (env, actor, prog) = pretty_check_file(&candid_path).expect("Cannot parse candid file");
        let (output, unused) = emit_bindgen(&config, &env, &actor, &prog);
        // TODO: handle unused.
        assert!(unused.is_empty());

        // 2. Generate the Rust bindings using the Handlebars template
        let template = include_str!("templates/consumer.hbs");
        let mut external = ExternalConfig::default();
        let canister_id = &self.canister_id.expect("canister_id not set");
        external
            .0
            .insert("canister_id".to_string(), canister_id.to_string());
        let content = output_handlebar(output, external, template);

        // 3. Write the generated Rust bindings to the output directory
        let out_dir_str = std::env::var("OUT_DIR")
            .expect("OUT_DIR should always be set when execute the build.rs script");
        let out_dir = PathBuf::from(out_dir_str);
        let generated_path = out_dir.join(format!("{}.rs", &self.canister_name));
        let mut file = fs::File::create(generated_path)?;
        writeln!(file, "{content}")?;
        Ok(())
    }
}

// https://github.com/dfinity/sdk/blob/master/docs/cli-reference/dfx-envars.mdx#canister_candid_path_canistername
fn candid_path_from_env(canister_name: &str) -> Result<PathBuf> {
    let canister_name_upper = canister_name.replace('-', "_").to_uppercase();
    let candid_path_var_name = format!("CANISTER_CANDID_PATH_{}", canister_name_upper);
    let candid_path_str = var_from_env(&candid_path_var_name)?;
    println!("cargo:rerun-if-env-changed={candid_path_var_name}");
    Ok(PathBuf::from(candid_path_str))
}

fn var_from_env(var: &str) -> Result<String> {
    env::var(var).map_err(|e| IcCdkBindgenError::EnvVarNotFound {
        var: var.to_string(),
        source: e,
    })
}
