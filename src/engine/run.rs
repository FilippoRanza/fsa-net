use crate::command;
use crate::network;

use super::diagnosis;
use super::full_space;
use super::linspace;
use super::NetworkResult;
use crate::input_output::{load_str_from_file, save_str_to_file};


use crate::graph;

pub fn run(
    net: &network::Network,
    reqs: &command::Requests,
    conf: &super::EngineConfig,
    file_names: &Vec<&str>
) -> Vec<NetworkResult> {
    reqs.commands
        .iter()
        .map(|req| run_request(net, req, conf, file_names))
        .collect()
}

fn run_request(
    net: &network::Network,
    req: &command::Command,
    conf: &super::EngineConfig,
    file_names: &Vec<&str>

) -> NetworkResult {
    match req {
        command::Command::FullSpace => full_space::compute_full_space(net, conf).into(),
        command::Command::Linspace((obs_labels, out_file)) => run_linspace(net, obs_labels, out_file, file_names, conf),
        command::Command::Diagnosis(cmd) => run_diagnosis(net, conf, cmd, file_names),
    }
}

fn run_linspace(net: &network::Network, labels: &Vec<usize>, out_file: &Option<usize>, file_names: &Vec<&str>, conf: &super::EngineConfig) -> NetworkResult {
    let lin_space = linspace::compute_linear_space(net, labels, conf);
    if let Some(file_index) = out_file {
        let file_name = file_names[*file_index];
        let graph = &lin_space.graph;
        let graph = graph.convert(network::trans_event_to_rel_label);
        let json = graph.save();    
        save_str_to_file(&json, file_name).unwrap();
    }

    lin_space.into()
}

fn run_diagnosis(
    net: &network::Network,
    conf: &super::EngineConfig,
    cmd: &command::DiagnosisCommand,
    file_names: &Vec<&str>
) -> NetworkResult {
    match cmd {
        command::DiagnosisCommand::Fresh(obs_labels) => run_fresh_diagnosis(net, conf, obs_labels),
        command::DiagnosisCommand::Load(file) => run_load_diagnosis(*file, file_names, conf),
    }
}

fn run_fresh_diagnosis(
    net: &network::Network,
    conf: &super::EngineConfig,
    obs_labels: &Vec<usize>,
) -> NetworkResult {
    let tmp = linspace::compute_linear_space(net, obs_labels, conf);
    if tmp.complete {
        diagnosis::diagnosis(&tmp.graph, conf)
    } else {
        diagnosis::fail_diagnosis()
    }
    .into()
}

fn run_load_diagnosis(out_file: usize, file_names: &Vec<&str>, conf: &super::EngineConfig) -> NetworkResult {
    let file_name = file_names[out_file];
    let data = load_str_from_file(file_name).unwrap();
    let g: graph::Graph<Option<usize>> = graph::Graph::load(&data);
    diagnosis::diagnosis(&g, conf).into()
}
