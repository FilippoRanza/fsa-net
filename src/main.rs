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
    #[structopt(flatten)]
    conf: EngineConfig
}

#[derive(StructOpt)]
struct EngineConfig {
    #[structopt(short="-p", long="--pretty", parse(from_flag = export_results::JsonFormat::new))]
    format: export_results::JsonFormat,
    #[structopt(short="-f", long="--full", parse(from_flag = engine::GraphMode::from_flag))]
    prune: engine::GraphMode,
}

fn run_request(comp_res: compiler::CompileResult, conf: EngineConfig) {
    for (i, cmd) in comp_res.compile_network.iter().enumerate() {
        let res = engine::run(&cmd.net, &cmd.req, &conf.prune);
        let net_table = comp_res.index_table.get_network_table(i);
        let res = export_results::export_results(res, net_table, &conf.format);
        println!("{}", res);
    }
}

fn main() {
    let args = Arguments::from_args();
    let src_code = input_output::get_fsa_code(&args.file).unwrap();
    let code = fsa_net_parser::parse(&src_code).unwrap();
    let compile_result = compiler::compile(&code).unwrap();
    run_request(compile_result, args.conf);
}
