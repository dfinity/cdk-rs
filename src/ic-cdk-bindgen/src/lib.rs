use candid::Principal;
use candid_parser::pretty_check_file;
use std::env;
use std::fs;
use std::io::Write;
use std::path::PathBuf;

mod code_generator;
mod error;

use code_generator::Target;
pub use error::IcCdkBindgenError;

type Result<T> = std::result::Result<T, IcCdkBindgenError>;

#[derive(Debug, Clone)]
pub struct Builder {
    pub(crate) canister_name: String,
    pub(crate) canister_id: Option<Principal>,
    pub(crate) candid_path: Option<PathBuf>,
    pub(crate) out_dir: Option<PathBuf>,
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
            out_dir: None,
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

    /// Set the output directory.
    ///
    /// If not set, the output directory will be resolved from the environment variable `OUT_DIR` set by cargo.
    ///
    /// The generated files will be placed in a subdirectory of the output directory based on the target:
    /// - `/consumer/` for the [consumer](Self::generate_consumer) target
    /// - `/provider/` for the [provider](Self::generate_provider) target
    /// - `/type/` for the [type](Self::generate_type) target
    pub fn out_dir<S>(&mut self, path: S) -> &mut Self
    where
        S: Into<PathBuf>,
    {
        self.out_dir = Some(path.into());
        self
    }
}

// Code generation.
impl Builder {
    fn generate(&self, target: Target) -> Result<()> {
        let mut binding = code_generator::Config::new();
        if target == Target::Consumer {
            let canister_id = if let Some(p) = &self.canister_id {
                *p
            } else {
                canister_id_from_env(&self.canister_name)?
            };
            binding.set_canister_id(canister_id);
        }
        binding
            .set_service_name(self.canister_name.to_string())
            .set_target(target);

        let candid_path = if let Some(p) = &self.candid_path {
            p.clone()
        } else {
            candid_path_from_env(&self.canister_name)?
        };

        let (env, actor) = pretty_check_file(&candid_path).expect("Cannot parse candid file");
        let content = code_generator::compile(&binding, &env, &actor);

        let out_dir = if let Some(p) = &self.out_dir {
            p.clone()
        } else {
            out_dir_from_env()?
        };
        let sub_dir_str = match target {
            Target::Consumer => "consumer",
            Target::Provider => "provider",
            Target::Type => "type",
        };
        let sub_dir = out_dir.join(sub_dir_str);
        fs::create_dir_all(&sub_dir)?;

        let generated_path = sub_dir.join(format!("{}.rs", &self.canister_name));
        let mut file = fs::File::create(generated_path)?;
        file.write_all(content.as_bytes())?;
        Ok(())
    }

    /// Generate the Rust bindings for consumer (inter-canister calls).
    pub fn generate_consumer(&self) -> Result<()> {
        self.generate(Target::Consumer)
    }

    /// Generate the Rust bindings for provider (implement canister entry-points).
    pub fn generate_provider(&self) -> Result<()> {
        self.generate(Target::Provider)
    }

    /// Generate the Rust bindings for type only (types used in the candid file).
    pub fn generate_type(&self) -> Result<()> {
        self.generate(Target::Type)
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

// https://github.com/dfinity/sdk/blob/master/docs/cli-reference/dfx-envars.mdx#canister_id_canistername
fn canister_id_from_env(canister_name: &str) -> Result<Principal> {
    let canister_name_upper = canister_name.replace('-', "_").to_uppercase();
    let canister_id_var_name = format!("CANISTER_ID_{}", canister_name_upper);
    let canister_id_str = var_from_env(&canister_id_var_name)?;
    println!("cargo:rerun-if-env-changed={canister_id_var_name}");
    Ok(Principal::from_text(canister_id_str)?)
}

fn out_dir_from_env() -> Result<PathBuf> {
    let out_dir_str = var_from_env("OUT_DIR")?;
    Ok(PathBuf::from(out_dir_str))
}

fn var_from_env(var: &str) -> Result<String> {
    env::var(var).map_err(|e| IcCdkBindgenError::EnvVarNotFound {
        var: var.to_string(),
        source: e,
    })
}
