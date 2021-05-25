
use crate::command;
use crate::network;

use super::full_space;
use super::linspace;
use super::NetworkResult;

pub fn run(net: &network::Network, reqs: &command::Requests) -> Vec<NetworkResult> {
    reqs.commands.iter().map(|req| run_request(net, req)).collect()
}

fn run_request(net: &network::Network, req: &command::Command) -> NetworkResult  {
    match req {
        command::Command::FullSpace => full_space::compute_full_space(net).into(),
        command::Command::Linspace(obs_labels) => linspace::compute_linear_space(net, obs_labels).into(),
        command::Command::Diagnosis(_obs_labels) => unimplemented!(),
    }
}

