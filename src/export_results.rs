use crate::compiler::{NetNames, NetworkIndexTable};
use crate::engine::{FullSpaceResult, LinSpaceResult, NetworkResult};
use crate::network;
use crate::graph::NodeKind;
use serde::Serialize;
use crate::utils::zip;

pub fn export_results(results: Vec<NetworkResult>, index_table: &NetworkIndexTable) -> String {
    results
        .into_iter()
        .map(|results| export_result(results, index_table))
        .fold(String::new(), |acc, curr| acc + &curr)
}

fn export_result(result: NetworkResult, table: &NetworkIndexTable) -> String {
    match result {
        NetworkResult::FullSpace(full_space) => export_full_space(full_space, table),
        NetworkResult::Linspace(lin_space) => export_lin_space(lin_space),
    }
}

fn export_full_space(full_space: FullSpaceResult, table: &NetworkIndexTable) -> String {
    let states = export_state_list(&full_space.states, &full_space.graph.get_node_kind_list(), table);
    let exporter = ExportFullSpace::new(full_space.graph.get_adjacent_list(), states);
    serde_json::to_string(&exporter).unwrap()
}

fn export_lin_space(lin_space: LinSpaceResult) -> String {
    Default::default()
}

#[derive(Serialize)]
struct ExportFullSpace<'a> {
    adjacent: &'a Vec<Vec<usize>>,
    states: Vec<State<'a>>,
}

impl<'a> ExportFullSpace<'a> {
    fn new(adjacent: &'a Vec<Vec<usize>>, states: Vec<State<'a>>) -> Self {
        Self { adjacent, states }
    }
}

fn export_state_list<'a>(
    states: &[network::State],
    state_kinds: &[NodeKind],
    table: &'a NetworkIndexTable,
) -> Vec<State<'a>> {
    zip(states, state_kinds).map(|(s, k)| export_state(s, k, table)).collect()
}

fn export_state<'a>(net_state: &network::State, state_kind: &NodeKind, table: &'a NetworkIndexTable) -> State<'a> {
    let states = net_state
        .get_states()
        .map(|(auto, state)| table.get_automata_names(auto).get_state_name(state))
        .collect();
    let net_table = table.get_network_names();
    let links = net_state
        .get_links()
        .map(|(link, content)| {
            (
                net_table.get_link_name(link),
                export_content(content, net_table),
            )
        })
        .collect();
    let kind = state_kind.into();
    State { states, links, kind }
}

fn export_content<'a>(content: Option<usize>, table: &'a NetNames) -> Option<&'a str> {
    if let Some(content) = content {
        let content = table.get_obs_name(content);
        Some(content)
    } else {
        None
    }
}

#[derive(Serialize)]
struct State<'a> {
    states: Vec<&'a str>,
    links: Vec<(&'a str, Option<&'a str>)>,
    kind: StateKind,
}

#[derive(Serialize)]
enum StateKind {
    Simple,
    Final
}

impl From<&NodeKind> for StateKind {
    fn from(nk: &NodeKind) -> Self {
        match nk {
            NodeKind::Final => Self::Final,
            NodeKind::Simple => Self::Simple
        }
    }
}
