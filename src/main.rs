use fsa_net_parser;
use std::path;
use structopt::StructOpt;

mod command;
mod compiler;
mod engine;
mod graph;
mod input_output;
mod network;
mod state_table;
mod utils;

#[derive(StructOpt)]
struct Arguments {
    file: Option<path::PathBuf>,
}

fn run_request(cmds: Vec<compiler::CompileResult>) {
    for cmd in &cmds {
        engine::run(&cmd.net, &cmd.req)
    }
}

fn main() {
    let args = Arguments::from_args();
    let src_code = input_output::get_fsa_code(&args.file).unwrap();
    let code = fsa_net_parser::parse(&src_code).unwrap();
    let compile_result = compiler::compile(&code).unwrap();
    run_request(compile_result);
}
