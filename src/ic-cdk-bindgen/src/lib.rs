use candid::Principal;
use candid_parser::pretty_check_file;
use std::env;
use std::fs;
use std::io::Write;
use std::path::PathBuf;

mod code_generator;
mod error;

pub use error::IcCdkBindgenError;

#[derive(Debug, Clone)]
pub struct Builder {
    pub(crate) canister_name: String,
    pub(crate) canister_id: Option<Principal>,
    pub(crate) candid_path: Option<PathBuf>,
    pub(crate) out_dir: Option<PathBuf>,
}

impl Builder {
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

    pub fn canister_id<S>(&mut self, canister_id: S) -> &mut Self
    where
        S: Into<Principal>,
    {
        self.canister_id = Some(canister_id.into());
        self
    }

    pub fn candid_path<S>(&mut self, path: S) -> &mut Self
    where
        S: Into<PathBuf>,
    {
        self.candid_path = Some(path.into());
        self
    }

    pub fn out_dir<S>(&mut self, path: S) -> &mut Self
    where
        S: Into<PathBuf>,
    {
        self.out_dir = Some(path.into());
        self
    }
}

impl Builder {
    pub fn generate_consumer(&self) -> Result<(), IcCdkBindgenError> {
        let (candid_path_env, canister_id_env) =
            resolve_candid_path_and_canister_id(&self.canister_name);
        let candid_path = self.candid_path.clone().unwrap_or(candid_path_env);
        let canister_id = self.canister_id.unwrap_or(canister_id_env);

        let mut binding = code_generator::Config::new();
        binding
            // User will depend on candid crate directly
            .set_candid_crate("candid".to_string())
            .set_canister_id(canister_id)
            .set_service_name(self.canister_name.to_string())
            .set_target(code_generator::Target::Consumer);

        let (env, actor) = pretty_check_file(&candid_path).expect("Cannot parse candid file");
        let content = code_generator::compile(&binding, &env, &actor);
        let out_dir = self.out_dir.clone().map(Ok).unwrap_or_else(|| {
            env::var_os("OUT_DIR")
                .ok_or_else(|| {
                    IcCdkBindgenError::Custom("OUT_DIR environment variable is not set".to_string())
                })
                .map(PathBuf::from)
        })?;
        let consumer_dir = out_dir.join("consumer");
        fs::create_dir_all(&consumer_dir)?;

        let generated_path = consumer_dir.join(format!("{}.rs", &self.canister_name));
        let mut file = fs::File::create(generated_path)?;
        file.write_all(content.as_bytes())?;
        Ok(())
    }

    pub fn generate_provider(&self) -> Result<(), IcCdkBindgenError> {
        unimplemented!()
    }

    pub fn generate_type(&self) -> Result<(), IcCdkBindgenError> {
        unimplemented!()
    }
}

/// Resolve the candid path and canister id from environment variables.
///
/// The name and format of the environment variables are standardized:
/// https://github.com/dfinity/sdk/blob/master/docs/cli-reference/dfx-envars.md#canister_id_canistername
///
/// We previously used environment variables like`CANISTER_CANDID_PATH_<canister_name>` without to_uppercase.
/// That is deprecated. To keep backward compatibility, we also check for the old format.
/// Just in case the user run `ic-cdk-bindgen` outside `dfx`.
/// If the old format is found, we print a warning to the user.
/// dfx v0.13.0 only provides the old format, which can be used to check the warning logic.
/// TODO: remove the support for the old format, in the next major release (v0.2) of `ic-cdk-bindgen`.
fn resolve_candid_path_and_canister_id(canister_name: &str) -> (PathBuf, Principal) {
    fn warning_deprecated_env(deprecated_name: &str, new_name: &str) {
        println!("cargo:warning=The environment variable {} is deprecated. Please set {} instead. Upgrading dfx may fix this issue.", deprecated_name, new_name);
    }

    let canister_name = canister_name.replace('-', "_");
    let canister_name_upper = canister_name.to_uppercase();

    let candid_path_var_name = format!("CANISTER_CANDID_PATH_{}", canister_name_upper);
    let candid_path_var_name_legacy = format!("CANISTER_CANDID_PATH_{}", canister_name);
    println!("cargo:rerun-if-env-changed={candid_path_var_name}");
    println!("cargo:rerun-if-env-changed={candid_path_var_name_legacy}");

    let candid_path_str = if let Ok(candid_path_str) = env::var(&candid_path_var_name) {
        candid_path_str
    } else if let Ok(candid_path_str) = env::var(&candid_path_var_name_legacy) {
        warning_deprecated_env(&candid_path_var_name_legacy, &candid_path_var_name);
        candid_path_str
    } else {
        panic!(
            "Cannot find environment variable: {}",
            &candid_path_var_name
        );
    };
    let candid_path = PathBuf::from(candid_path_str);

    let canister_id_var_name = format!("CANISTER_ID_{}", canister_name_upper);
    let canister_id_var_name_legacy = format!("CANISTER_ID_{}", canister_name);
    println!("cargo:rerun-if-env-changed={canister_id_var_name}");
    println!("cargo:rerun-if-env-changed={canister_id_var_name_legacy}");
    let canister_id_str = if let Ok(canister_id_str) = env::var(&canister_id_var_name) {
        canister_id_str
    } else if let Ok(canister_id_str) = env::var(&canister_id_var_name_legacy) {
        warning_deprecated_env(&canister_id_var_name_legacy, &canister_id_var_name);
        canister_id_str
    } else {
        panic!(
            "Cannot find environment variable: {}",
            &canister_id_var_name
        );
    };
    let canister_id = Principal::from_text(&canister_id_str)
        .unwrap_or_else(|_| panic!("Invalid principal: {}", &canister_id_str));

    (candid_path, canister_id)
}
