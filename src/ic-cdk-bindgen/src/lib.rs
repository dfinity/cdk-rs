use candid::Principal;
use candid_parser::pretty_check_file;
use std::env;
use std::fs;
use std::io::Write;
use std::path::PathBuf;

mod code_generator;

#[derive(Clone)]
pub struct Config {
    pub canister_name: String,
    pub candid_path: PathBuf,
    pub skip_existing_files: bool,
    pub binding: code_generator::Config,
}

impl Config {
    pub fn new(canister_name: &str) -> Self {
        let (candid_path, canister_id) = resolve_candid_path_and_canister_id(canister_name);
        let mut binding = code_generator::Config::new();
        binding
            // User will depend on candid crate directly
            .set_candid_crate("candid".to_string())
            .set_canister_id(canister_id)
            .set_service_name(canister_name.to_string())
            .set_target(code_generator::Target::CanisterCall);

        Config {
            canister_name: canister_name.to_string(),
            candid_path,
            skip_existing_files: false,
            binding,
        }
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

#[derive(Default)]
pub struct Builder {
    configs: Vec<Config>,
}

impl Builder {
    pub fn new() -> Self {
        Builder {
            configs: Vec::new(),
        }
    }
    pub fn add(&mut self, config: Config) -> &mut Self {
        self.configs.push(config);
        self
    }
    pub fn build(self, out_path: Option<PathBuf>) {
        let out_path = out_path.unwrap_or_else(|| {
            let manifest_dir =
                PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("Cannot find manifest dir"));
            manifest_dir.join("src").join("declarations")
        });
        fs::create_dir_all(&out_path).unwrap();
        for conf in self.configs.iter() {
            let (env, actor) =
                pretty_check_file(&conf.candid_path).expect("Cannot parse candid file");
            let content = code_generator::compile(&conf.binding, &env, &actor);
            let generated_path = out_path.join(format!("{}.rs", conf.canister_name));
            if !(conf.skip_existing_files && generated_path.exists()) {
                fs::write(generated_path, content).expect("Cannot store generated binding");
            }
        }
        let mut module = fs::File::create(out_path.join("mod.rs")).unwrap();
        module.write_all(b"#![allow(unused_imports)]\n").unwrap();
        module
            .write_all(b"#![allow(non_upper_case_globals)]\n")
            .unwrap();
        module.write_all(b"#![allow(non_snake_case)]\n").unwrap();
        for conf in self.configs.iter() {
            module.write_all(b"#[rustfmt::skip]\n").unwrap(); // so that we get a better diff
            let line = format!("pub mod {};\n", conf.canister_name);
            module.write_all(line.as_bytes()).unwrap();
        }
    }
}
