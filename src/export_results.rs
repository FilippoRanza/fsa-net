use crate::compiler::{NetNames, NetworkIndexTable};
use crate::engine::{FullSpaceResult, LinSpaceResult, NetworkResult};
use crate::graph::NodeKind;
use crate::network;
use crate::utils::zip;
use serde::Serialize;

pub enum JsonFormat {
    Compact,
    Pretty,
}

impl JsonFormat {
    pub fn new(pretty: bool) -> Self {
        if pretty {
            Self::Pretty
        } else {
            Self::Compact
        }
    }
}

pub fn export_results(
    results: Vec<NetworkResult>,
    index_table: &NetworkIndexTable,
    fmt: &JsonFormat,
) -> String {
    let exports = results
        .iter()
        .map(|results| export_result(results, index_table))
        .collect();
    let name = index_table.get_name();
    ExportResult { name, exports }.to_json(fmt)
}

#[derive(Serialize)]
pub struct ExportResult<'a> {
    name: &'a str,
    exports: Vec<Export<'a>>,
}

impl<'a> ExportResult<'a> {
    fn to_json(&self, fmt: &JsonFormat) -> String {
        let f = match fmt {
            JsonFormat::Compact => serde_json::to_string,
            JsonFormat::Pretty => serde_json::to_string_pretty,
        };
        f(self).unwrap()
    }
}

fn export_result<'a>(result: &'a NetworkResult, table: &'a NetworkIndexTable<'a>) -> Export<'a> {
    match result {
        NetworkResult::FullSpace(full_space) => export_full_space(&full_space, table).into(),
        NetworkResult::Linspace(lin_space) => export_lin_space(&lin_space, table).into(),
    }
}

fn export_full_space<'a>(
    full_space: &'a FullSpaceResult,
    table: &'a NetworkIndexTable<'a>,
) -> ExportFullSpace<'a> {
    let states = export_state_list(
        &full_space.states,
        &full_space.graph.get_node_kind_list(),
        table,
    );
    ExportFullSpace::new(full_space.graph.get_adjacent_list(), states)
}

fn export_lin_space<'a>(
    lin_space: &'a LinSpaceResult,
    table: &'a NetworkIndexTable<'a>,
) -> ExportLinSpace<'a> {
    let states = export_state_list(
        &lin_space.states,
        &lin_space.graph.get_node_kind_list(),
        table,
    );
    ExportLinSpace {
        adjacent: lin_space.graph.get_adjacent_list(),
        states,
    }
}

#[derive(Serialize)]
enum Export<'a> {
    FullSpace(ExportFullSpace<'a>),
    LinSpace(ExportLinSpace<'a>),
}

impl<'a> From<ExportFullSpace<'a>> for Export<'a> {
    fn from(res: ExportFullSpace<'a>) -> Self {
        Self::FullSpace(res)
    }
}

impl<'a> From<ExportLinSpace<'a>> for Export<'a> {
    fn from(res: ExportLinSpace<'a>) -> Self {
        Self::LinSpace(res)
    }
}

#[derive(Serialize)]
struct ExportFullSpace<'a> {
    adjacent: &'a Vec<Vec<usize>>,
    states: Vec<State<'a>>,
}

#[derive(Serialize)]
struct ExportLinSpace<'a> {
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
    zip(states, state_kinds)
        .map(|(s, k)| export_state(s, k, table))
        .collect()
}

fn export_state<'a>(
    net_state: &network::State,
    state_kind: &NodeKind,
    table: &'a NetworkIndexTable,
) -> State<'a> {
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
    State {
        states,
        links,
        kind,
    }
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
    Final,
}

impl From<&NodeKind> for StateKind {
    fn from(nk: &NodeKind) -> Self {
        match nk {
            NodeKind::Final => Self::Final,
            NodeKind::Simple => Self::Simple,
        }
    }
}
