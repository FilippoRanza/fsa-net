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
) -> Vec<NetworkResult> {
    reqs.commands
        .iter()
        .map(|req| run_request(net, req, conf))
        .collect()
}

fn run_request(
    net: &network::Network,
    req: &command::Command,
    conf: &super::EngineConfig,
) -> NetworkResult {
    match req {
        command::Command::FullSpace => full_space::compute_full_space(net, conf).into(),
        command::Command::Linspace(obs_labels) => {
            linspace::compute_linear_space(net, obs_labels, conf).into()
        }
        command::Command::Diagnosis(obs_labels) => run_diagnosis(net, conf, obs_labels),
    }
}

fn run_diagnosis(
    net: &network::Network,
    conf: &super::EngineConfig,
    obs_labels: &[usize],
) -> NetworkResult {
    let tmp = linspace::compute_linear_space(net, obs_labels, conf);
    if tmp.complete {
        diagnosis::diagnosis(&tmp.graph, conf)
    } else {
        diagnosis::fail_diagnosis()
    }
    .into()
}
