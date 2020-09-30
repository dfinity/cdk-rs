use clap::{ArgMatches, Clap, FromArgMatches, IntoApp};
use std::io::{Read, Write};
use std::path::PathBuf;
use wabt::{wasm2wat, wat2wasm};

pub type PassResult = Result<Vec<u8>, Box<dyn std::error::Error>>;

pub trait OptimizationPass {
    fn args(&self) -> clap::App {
        clap::App::new(&self.short_name())
    }
    fn short_name(&self) -> String;
    fn description(&self) -> String;
    fn opt(&self, wasm: &[u8], args: &ArgMatches) -> PassResult;
}

struct RemoveDebugSymbolsPass {}

impl OptimizationPass for RemoveDebugSymbolsPass {
    fn short_name(&self) -> String {
        String::from("strip_data")
    }

    fn description(&self) -> String {
        String::from("Stripping Unused Data Segments")
    }

    fn opt(&self, wasm: &[u8], _: &ArgMatches) -> PassResult {
        let wat = wasm2wat(&wasm)?;
        Ok(wat2wasm(&wat)?)
    }
}

struct WasmOptPass {}

#[derive(Clap, Debug)]
struct WasmOptPassArgs {
    /// The path to the `wasm-opt` binary. If not specified, will skip the optimization.
    #[clap(long)]
    wasm_opt_path: Option<PathBuf>,
}

impl OptimizationPass for WasmOptPass {
    fn args(&self) -> clap::App {
        WasmOptPassArgs::into_app()
    }

    fn short_name(&self) -> String {
        String::from("strip_data")
    }

    fn description(&self) -> String {
        String::from("Executing wasm-opt -Oz")
    }

    fn opt(&self, wasm: &[u8], matches: &ArgMatches) -> PassResult {
        let opts = WasmOptPassArgs::from_arg_matches(&matches);
        if let Some(wasm_opt_path) = opts.wasm_opt_path {
            // Write this to disk. wasm-opt only works with disk.
            let file_in = tempfile::NamedTempFile::new()?;
            file_in.as_file().write_all(&wasm)?;

            let path_out = tempfile::NamedTempFile::new()?.into_temp_path();

            let _ = std::process::Command::new(wasm_opt_path)
                .arg(file_in.path())
                .arg("-Oz")
                .arg("--output")
                .arg(path_out.as_os_str())
                .output()?;

            let mut file_out = std::fs::File::open(path_out)?;
            let metadata = file_out.metadata()?;
            let mut new_wasm = vec![0; metadata.len() as usize];
            file_out.read(&mut new_wasm)?;

            return Ok(new_wasm);
        } else {
            eprintln!("Skipping pass...");
        }
        Ok(wasm.to_vec())
    }
}

pub fn create() -> Vec<Box<dyn OptimizationPass>> {
    vec![
        Box::new(RemoveDebugSymbolsPass {}),
        Box::new(WasmOptPass {}),
    ]
}
