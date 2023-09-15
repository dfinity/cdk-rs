use std::path::Path;

use anyhow::{bail, Result};
use wasmtime::*;

static IC0: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/ic_mock.wat"));

fn generate_candid<P>(wasm_path: P) -> Result<String>
where
    P: AsRef<Path>,
{
    let mut store: Store<()> = Store::<()>::default();

    let mut linker = Linker::new(store.engine());
    let ic0_module = Module::new(store.engine(), IC0)?;
    let ic0 = linker.instantiate(&mut store, &ic0_module)?;
    linker.instance(&mut store, "ic0", ic0)?;

    let module = Module::from_file(store.engine(), wasm_path)?;
    let canister = linker.instantiate(&mut store, &module)?;

    let get_candid_pointer =
        canister.get_typed_func::<(), i32>(&mut store, "get_candid_pointer")?;
    let candid_pointer = get_candid_pointer.call(&mut store, ())?;

    let memory = canister
        .get_memory(&mut store, "memory")
        .ok_or(anyhow::format_err!("failed to find `memory` export"))?;
    let memory_buffer = memory.data(&store);

    let mut i = candid_pointer as usize;
    let mut str_vec = vec![];
    while memory_buffer[i] != 0 {
        str_vec.push(memory_buffer[i]);
        i += 1;
    }
    let s = String::from_utf8(str_vec)?;
    Ok(s)
}

fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().collect();
    if args.len() != 2 {
        // The first arg will the name of current binary.
        bail!("Expecting one argument: path to the canister WASM file");
    }
    let c = generate_candid(args.last().unwrap())?;
    println!("{c}");
    Ok(())
}
