use fsa_net_parser;
use std::path;
use structopt::StructOpt;

mod compiler;
mod input_output;

#[derive(StructOpt)]
struct Arguments {
    file: Option<path::PathBuf>,
}

fn main() {
    let args = Arguments::from_args();
    let src_code = input_output::get_fsa_code(&args.file).unwrap();
    let code = fsa_net_parser::parse(&src_code).unwrap();
    let _ = compiler::compile(&code).unwrap();
}
