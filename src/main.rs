use fsa_net_parser;
use std::path;
use structopt::StructOpt;

mod command;
mod compiler;
mod engine;
mod export_results;
mod graph;
mod input_output;
mod network;
mod state_table;
mod utils;

#[derive(StructOpt)]
struct Arguments {
    file: Option<path::PathBuf>,
    #[structopt(short="-p", long="--pretty", parse(from_flag = export_results::JsonFormat::new))]
    format: export_results::JsonFormat,
}

fn run_request(comp_res: compiler::CompileResult, format: export_results::JsonFormat) {
    for (i, cmd) in comp_res.compile_network.iter().enumerate() {
        let res = engine::run(&cmd.net, &cmd.req);
        let net_table = comp_res.index_table.get_network_table(i);
        let res = export_results::export_results(res, net_table, &format);
        println!("{}", res);
    }
}

fn main() {
    let args = Arguments::from_args();
    let src_code = input_output::get_fsa_code(&args.file).unwrap();
    let code = fsa_net_parser::parse(&src_code).unwrap();
    let compile_result = compiler::compile(&code).unwrap();
    run_request(compile_result, args.format);
}
