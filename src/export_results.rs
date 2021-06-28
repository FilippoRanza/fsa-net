use crate::compiler::{NetNames, NetworkIndexTable};
use crate::engine::{DiagnosisResult, FullSpaceResult, LinSpaceResult, NetworkResult, Regex};
use crate::graph;
use crate::network;
use crate::utils::zip;
use serde::Serialize;

type NRes = Result<NetworkResult, Box<dyn std::error::Error>>;

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
    results: Vec<NRes>,
    index_table: &NetworkIndexTable,
    fmt: &JsonFormat,
) -> String {
    let exports = results
        .iter()
        .map(|results| export_result(results, index_table))
        .collect();
    let name = index_table.get_name();
    FullResult { name, exports }.to_json(fmt)
}

#[derive(Serialize)]
pub struct FullResult<'a> {
    name: &'a str,
    exports: Vec<ExportResult<'a>>,
}

impl<'a> FullResult<'a> {
    fn to_json(&self, fmt: &JsonFormat) -> String {
        let f = match fmt {
            JsonFormat::Compact => serde_json::to_string,
            JsonFormat::Pretty => serde_json::to_string_pretty,
        };
        f(self).unwrap()
    }
}

fn export_result<'a>(result: &'a NRes, table: &'a NetworkIndexTable<'a>) -> ExportResult<'a> {
    match result {
        Ok(result) => ExportResult::Success(match result {
            NetworkResult::FullSpace(full_space) => export_full_space(&full_space, table).into(),
            NetworkResult::Linspace(lin_space) => export_lin_space(&lin_space, table).into(),
            NetworkResult::Diagnosis(diagnosis) => export_diagnosis(diagnosis, table).into(),
        }),
        Err(err) => {
            let msg = format!("{}", err);
            ExportResult::Error(msg)
        }
    }
}

fn export_diagnosis(diag: &DiagnosisResult, table: &NetworkIndexTable) -> ExportDiagnosis {
    if let Some(regex) = &diag.matrix {
        let regex = export_regex(regex, table.get_network_names());
        ExportDiagnosis { regex: Some(regex), complete: diag.complete, timeout: diag.timeout }
    } else {
        ExportDiagnosis { regex: None, complete: diag.complete,  timeout: diag.timeout  }
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
struct ExportDiagnosis {
    regex: Option<String>,
    complete: bool,
    timeout: bool
}

#[derive(Serialize)]
enum ExportResult<'a> {
    Success(Export<'a>),
    Error(String),
}

#[derive(Serialize)]
enum Export<'a> {
    FullSpace(ExportFullSpace<'a>),
    LinSpace(ExportLinSpace<'a>),
    Diagnosis(ExportDiagnosis),
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

impl<'a> From<ExportDiagnosis> for Export<'a> {
    fn from(res: ExportDiagnosis) -> Self {
        Self::Diagnosis(res)
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
    states: Vec<IndexedState<'a>>,
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

fn export_state_list<'a, T>(
    states: &[network::State],
    state_kinds: &[graph::NodeKind],
    table: &'a NetworkIndexTable,
) -> Vec<T>
where
    T: Convert<'a>,
{
    zip(states, state_kinds)
        .map(|(s, k)| T::convert(s, table, k))
        .collect()
}

fn export_content<'a>(content: Option<usize>, table: &'a NetNames) -> Option<&'a str> {
    if let Some(content) = content {
        let content = table.get_ev_name(content);
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

impl<'a> Convert<'a> for State<'a> {
    fn convert(
        state: &network::State,
        index: &'a NetworkIndexTable<'a>,
        kind: &graph::NodeKind,
    ) -> Self {
        let states = state
            .get_states()
            .map(|(auto, state)| index.get_automata_names(auto).get_state_name(state))
            .collect();
        let net_table = index.get_network_names();
        let links = state
            .get_links()
            .map(|(link, content)| {
                (
                    net_table.get_link_name(link),
                    export_content(content, net_table),
                )
            })
            .collect();
        let kind = kind.into();
        State {
            states,
            links,
            kind,
        }
    }
}

#[derive(Serialize)]
struct IndexedState<'a> {
    state: State<'a>,
    index: usize,
}

impl<'a> Convert<'a> for IndexedState<'a> {
    fn convert(
        state: &network::State,
        index: &'a NetworkIndexTable<'a>,
        kind: &graph::NodeKind,
    ) -> Self {
        let out_state = State::convert(state, index, kind);
        Self {
            state: out_state,
            index: state.get_index(),
        }
    }
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
    ev: TransEvent<'a>,
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
    adj.iter().map(|n| convert_arc(n, index_table)).collect()
}

fn convert_arc<'a>(
    arc: &graph::Arc<network::TransEvent>,
    index_table: &'a NetworkIndexTable<'a>,
) -> Arc<'a> {
    let ev = TransEvent::new(&arc.label, index_table);
    Arc { next: arc.next, ev }
}

trait Convert<'a> {
    fn convert(
        state: &network::State,
        index: &'a NetworkIndexTable<'a>,
        kind: &graph::NodeKind,
    ) -> Self;
}

fn export_regex(regex: &Regex, table: &NetNames) -> String {
    match regex {
        Regex::Alternative(alt) => {
            let alt =
                alt.iter()
                    .map(|a| export_regex(a, table))
                    .fold(String::new(), |acc, curr| {
                        if acc.len() == 0 {
                            format!("({})", curr)
                        } else {
                            format!("{}|({})", acc, curr)
                        }
                    });
            format!("({})", alt)
        }
        Regex::Chain(chain) => chain.iter().map(|c| export_regex(c, table)).collect(),
        Regex::Optional(opt) => format!("({})?", export_regex(opt, table)),
        Regex::Value(val) => join_values(val, table),
        Regex::ZeroMore(rep) => format!("({})*", export_regex(rep, table)),
    }
}

fn join_values(vals: &[usize], table: &NetNames) -> String {
    vals.iter().map(|r| table.get_rel_name(*r)).collect()
}
