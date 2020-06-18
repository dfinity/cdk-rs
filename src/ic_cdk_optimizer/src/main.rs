use clap::Clap;
use humansize::{file_size_opts, FileSize};
use std::io::Read;
use std::path::PathBuf;
use wabt::{wasm2wat, wat2wasm};

#[derive(Clap, Debug)]
#[clap()]
struct CommandLineOpts {
    /// Input file to optimize. By default will use STDIN.
    input: Option<PathBuf>,

    /// Output file. Required.
    #[clap(short)]
    output: PathBuf,
}

fn main() {
    let opts: CommandLineOpts = CommandLineOpts::parse();

    let content = if let Some(i) = opts.input {
        std::fs::read(&i).expect("Could not read the file.")
    } else {
        let mut buff = Vec::new();
        std::io::stdin()
            .read_to_end(&mut buff)
            .expect("Could not read STDIN.");
        buff
    };

    eprintln!(
        "Original:          {:>8}",
        content.len().file_size(file_size_opts::BINARY).unwrap()
    );

    let wat = wasm2wat(&content).expect("Invalid WASM:");
    let wasm_back = wat2wasm(&wat).expect("Unexpected error:");

    eprintln!(
        "Stripping symbols: {} ({:3.1}% smaller)",
        wasm_back.len().file_size(file_size_opts::BINARY).unwrap(),
        (1.0 - ((wasm_back.len() as f64) / (content.len() as f64))) * 100.0
    );

    std::fs::write(opts.output, wasm_back).expect("Could not write output file.");

    // eprintln!("{:?}", opts);
}
