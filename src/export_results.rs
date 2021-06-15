use crate::compiler::{NetNames, NetworkIndexTable};
use crate::engine::{FullSpaceResult, LinSpaceResult, NetworkResult};
use crate::graph;
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
    let adj = export_adjacent_matrix(full_space.graph.get_adjacent_list(), table);
    ExportFullSpace::new(adj, states, full_space.complete)
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

    let adjacent = export_adjacent_matrix(lin_space.graph.get_adjacent_list(), table);
    ExportLinSpace {
        adjacent,
        states,
        complete: lin_space.complete,
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
    adjacent: Vec<Vec<Arc<'a>>>,
    states: Vec<State<'a>>,
    complete: bool,
}

#[derive(Serialize)]
struct ExportLinSpace<'a> {
    adjacent: Vec<Vec<Arc<'a>>>,
    states: Vec<State<'a>>,
    complete: bool,
}

impl<'a> ExportFullSpace<'a> {
    fn new(adjacent: Vec<Vec<Arc<'a>>>, states: Vec<State<'a>>, complete: bool) -> Self {
        Self {
            adjacent,
            states,
            complete,
        }
    }
}

fn export_state_list<'a>(
    states: &[network::State],
    state_kinds: &[graph::NodeKind],
    table: &'a NetworkIndexTable,
) -> Vec<State<'a>> {
    zip(states, state_kinds)
        .map(|(s, k)| export_state(s, k, table))
        .collect()
}

fn export_state<'a>(
    net_state: &network::State,
    state_kind: &graph::NodeKind,
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

impl From<&graph::NodeKind> for StateKind {
    fn from(nk: &graph::NodeKind) -> Self {
        match nk {
            graph::NodeKind::Final => Self::Final,
            graph::NodeKind::Simple => Self::Simple,
        }
    }
}

#[derive(Serialize)]
struct Arc<'a> {
    next: usize,
    ev: TransEvent<'a>
}

#[derive(Serialize)]
struct TransEvent<'a> {
    src: &'a str,
    name: &'a str,
    rel: Option<&'a str>,
    obs: Option<&'a str>,
}

impl<'a> TransEvent<'a> {
    fn new(te: &network::TransEvent, index_table: &'a NetworkIndexTable<'a>) -> Self {
        let map = index_table.get_network_names();
        let obs = map_option(&te.obs, |v| map.get_obs_name(v));
        let rel = map_option(&te.rel, |v| map.get_rel_name(v));

        let map = index_table.get_automata_names(te.auto);
        let src = map.get_name();
        let name = map.get_transition_name(te.trans);

        Self {
            obs,
            rel,
            name,
            src,
        }
    }
}

fn map_option<'a, F>(op: &Option<usize>, map: F) -> Option<&'a str>
where
    F: Fn(usize) -> &'a str,
{
    if let Some(v) = op {
        Some(map(*v))
    } else {
        None
    }
}

fn export_adjacent_matrix<'a>(
    adj: &graph::AdjList<network::TransEvent>,
    index_table: &'a NetworkIndexTable<'a>,
) -> Vec<Vec<Arc<'a>>> {
    adj.iter()
        .map(|l| export_adjacent_list(l, index_table))
        .collect()
}

fn export_adjacent_list<'a>(
    adj: &[graph::Arc<network::TransEvent>],
    index_table: &'a NetworkIndexTable<'a>,
) -> Vec<Arc<'a>> {
    adj.iter()
        .map(|n| convert_arc(n, index_table))
        .collect()
}

fn convert_arc<'a>(arc: &graph::Arc<network::TransEvent>, index_table: &'a NetworkIndexTable<'a>) -> Arc<'a> {
    let ev = TransEvent::new(&arc.label, index_table);
    Arc {
        next: arc.next,
        ev
    }
}
