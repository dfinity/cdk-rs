use clap::Parser;
use humansize::{file_size_opts, FileSize};
use std::io::Read;
use std::path::PathBuf;

mod passes;

#[derive(Parser, Debug)]
#[clap(version)]
struct CommandLineOpts {
    /// Input file to optimize. By default will use STDIN.
    input: Option<PathBuf>,

    /// Output file. Required.
    #[clap(short, long)]
    output: PathBuf,
}

fn main() {
    let passes = passes::create();
    let opts = CommandLineOpts::parse();
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

    let original_wasm_size = content.len();
    let mut wasm_size = content.len();
    let mut wasm_back = content;

    for pass in passes {
        eprintln!("{}...", pass.description());
        let new_wasm = pass.opt(&wasm_back).expect("Pass failed:");
        if new_wasm.len() < wasm_back.len() {
            wasm_back = new_wasm;
            eprintln!(
                "    Size:          {:>8} ({:3.1}% smaller)",
                wasm_back.len().file_size(file_size_opts::BINARY).unwrap(),
                (1.0 - ((wasm_back.len() as f64) / (wasm_size as f64))) * 100.0
            );
        } else {
            eprintln!("Pass did not result in smaller WASM... Skipping.");
        }
        wasm_size = wasm_back.len();
    }

    eprintln!(
        "\nFinal Size: {} ({:3.1}% smaller)",
        wasm_back.len().file_size(file_size_opts::BINARY).unwrap(),
        (1.0 - ((wasm_back.len() as f64) / (original_wasm_size as f64))) * 100.0
    );

    std::fs::write(opts.output, wasm_back).expect("Could not write output file.");
}
