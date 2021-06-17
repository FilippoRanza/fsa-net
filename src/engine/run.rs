use crate::command;
use crate::network;

use super::diagnosis;
use super::full_space;
use super::linspace;
use super::NetworkResult;

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
        command::Command::Linspace((obs_labels, _)) => {
            linspace::compute_linear_space(net, obs_labels, conf).into()
        }
        command::Command::Diagnosis(cmd) => run_diagnosis(net, conf, cmd, file_names),
    }
}

fn run_diagnosis(
    net: &network::Network,
    conf: &super::EngineConfig,
    cmd: &command::DiagnosisCommand,
    file_names: &Vec<&str>
) -> NetworkResult {
    match cmd {
        command::DiagnosisCommand::Fresh(obs_labels) => run_fresh_diagnosis(net, conf, obs_labels),
        command::DiagnosisCommand::Load(file) => unimplemented!(),
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
