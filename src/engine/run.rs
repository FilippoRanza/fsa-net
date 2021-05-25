
use crate::command;
use crate::network;

use super::full_space;

pub fn run(net: &network::Network, reqs: &command::Requests) {
    for req in &reqs.commands {
        run_request(net, req);
    }
}

fn run_request(net: &network::Network, req: &command::Command) {
    match req {
        command::Command::FullSpace => full_space::compute_full_space(net),
        command::Command::Linspace(_obs_labels) => unimplemented!(),
        command::Command::Diagnosis(_obs_labels) => unimplemented!(),
    };
}

