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
mod timer;
mod utils;

#[derive(StructOpt)]
struct Arguments {
    input: Option<path::PathBuf>,
    output: Option<path::PathBuf>,
    #[structopt(flatten)]
    conf: EngineConfig,
}

#[derive(StructOpt)]
struct EngineConfig {
    #[structopt(short="-p", long="--pretty", parse(from_flag = export_results::JsonFormat::new))]
    format: export_results::JsonFormat,
    #[structopt(short="-f", long="--full", parse(from_flag = engine::GraphMode::from_flag))]
    prune: engine::GraphMode,
    #[structopt(short="-t", long="--time-limit",parse(try_from_str = timer::parse_time_spec))]
    time_limit: Option<u64>,
}

fn run_request(comp_res: compiler::CompileResult, conf: EngineConfig) -> String {
    let timer_factory = timer::TimerFactory::from_value(conf.time_limit);
    let engine_config = engine::EngineConfig::new(conf.prune, timer_factory);
    let mut results = Vec::with_capacity(comp_res.compile_network.len());
    for (i, cmd) in comp_res.compile_network.iter().enumerate() {
        let net_table = comp_res.index_table.get_network_table(i);
        let res = engine::run(
            &cmd.net,
            &cmd.req,
            &engine_config,
            net_table.get_files_names(),
        );
        results.push(res);
    }
    let full_res: Vec<_> = results
        .iter()
        .enumerate()
        .map(|(i, r)| export_results::export_results(r, comp_res.index_table.get_network_table(i)))
        .collect();

    match conf.format {
        export_results::JsonFormat::Pretty => serde_json::to_string_pretty(&full_res),
        export_results::JsonFormat::Compact => serde_json::to_string(&full_res),
    }
    .unwrap()
}

fn main() {
    let args = Arguments::from_args();
    let src_code = input_output::get_fsa_code(&args.input).unwrap();
    let code = fsa_net_parser::parse(&src_code).unwrap();
    let compile_result = compiler::compile(&code).unwrap();
    let result = run_request(compile_result, args.conf);
    input_output::write_results(result, args.output).unwrap();
}
