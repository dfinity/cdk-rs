use crate::passes::{OptimizationPass, PassResult};
use binaryen::{CodegenConfig, Module};
use clap::{ArgMatches, Clap, IntoApp};

pub struct BinaryenPass {}

impl BinaryenPass {}

#[derive(Clap, Debug)]
pub struct BinaryenPassArgs {}

impl OptimizationPass for BinaryenPass {
    fn args(&self) -> clap::App {
        BinaryenPassArgs::into_app()
    }

    fn short_name(&self) -> String {
        String::from("binaryen")
    }

    fn description(&self) -> String {
        String::from("Execute a binaryen optimization pass on your WASM.")
    }

    fn opt(&self, wasm: &[u8], _matches: &ArgMatches) -> PassResult {
        let mut module =
            Module::read(wasm).map_err(|_| String::from("Could not load module..."))?;

        module.optimize(&CodegenConfig {
            debug_info: false,
            optimization_level: 2,
            shrink_level: 2,
        });

        Ok(module.write())
    }
}
