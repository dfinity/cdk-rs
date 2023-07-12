use candid::{bindings::rust, pretty_check_file, Principal};
use std::env;
use std::fs;
use std::io::Write;
use std::path::PathBuf;

#[derive(Clone)]
pub struct Config {
    pub canister_name: String,
    pub candid_path: PathBuf,
    pub skip_existing_files: bool,
    pub binding: rust::Config,
}
impl Config {
    pub fn new(canister_name: &str) -> Self {
        let candid_path_var_name = format!("CANISTER_CANDID_PATH_{}", canister_name);
        let candid_path =
            PathBuf::from(env::var(candid_path_var_name).expect("Cannot find candid path"));
        let canister_id_var_name = format!("CANISTER_ID_{}", canister_name);
        let canister_id =
            Principal::from_text(env::var(canister_id_var_name).expect("Cannot find canister id"))
                .unwrap();
        Config {
            canister_name: canister_name.to_string(),
            candid_path,
            skip_existing_files: false,
            binding: rust::Config {
                // User will depend on candid crate directly
                candid_crate: "candid".to_string(),
                type_attributes: "".to_string(),
                canister_id: Some(canister_id),
                service_name: canister_name.to_string(),
                target: rust::Target::CanisterCall,
            },
        }
    }
}

pub struct Builder {
    configs: Vec<Config>,
}
impl Builder {
    pub fn new() -> Self {
        Builder {
            configs: Vec::new(),
        }
    }
    pub fn add<'a>(&'a mut self, config: Config) -> &'a mut Self {
        self.configs.push(config);
        self
    }
    pub fn build(self, out_path: Option<PathBuf>) {
        let out_path = out_path.unwrap_or_else(|| {
            let manifest_dir =
                PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("Cannot find manifest dir"));
            // TODO dfx provide out_path
            let out_path = manifest_dir.join("src").join("declarations");
            out_path
        });
        fs::create_dir_all(&out_path).unwrap();
        for conf in self.configs.iter() {
            let (env, actor) =
                pretty_check_file(&conf.candid_path).expect("Cannot parse candid file");
            let content = rust::compile(&conf.binding, &env, &actor);
            let generated_path = out_path.join(&format!("{}.rs", conf.canister_name));
            if !(conf.skip_existing_files && generated_path.exists()) {
                fs::write(generated_path, content).expect("Cannot store generated binding");
            }
        }
        let mut module = fs::File::create(out_path.join("mod.rs")).unwrap();
        module.write(b"#![allow(unused_imports)]\n").unwrap();
        module.write(b"#![allow(non_upper_case_globals)]\n").unwrap();
        module.write(b"#![allow(non_snake_case)]\n").unwrap();
        for conf in self.configs.iter() {
            module.write(b"#[rustfmt::skip]\n").unwrap(); // so that we get a better diff
            let line = format!("pub mod {};\n", conf.canister_name);
            module.write(line.as_bytes()).unwrap();
        }
        module.flush().unwrap();
    }
}
