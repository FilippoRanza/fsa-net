mod diagnosis;
mod engine_utils;
mod full_space;
mod linspace;
mod run;

pub use diagnosis::DiagnosisResult;
pub use full_space::FullSpaceResult;
pub use linspace::LinSpaceResult;
pub use run::run;

use crate::graph;
use crate::state_table;
use crate::timer;

pub enum NetworkResult {
    FullSpace(full_space::FullSpaceResult),
    Linspace(linspace::LinSpaceResult),
    Diagnosis(diagnosis::DiagnosisResult),
}

pub struct EngineConfig {
    mode: GraphMode,
    timer_factory: timer::TimerFactory,
    deduplicate: bool,
}

impl EngineConfig {
    pub fn new(mode: GraphMode, timer: timer::TimerFactory, deduplicate: bool) -> Self {
        Self {
            mode,
            timer_factory: timer,
            deduplicate,
        }
    }
}

pub enum GraphMode {
    Prune,
    Full,
}

impl GraphMode {
    pub fn from_flag(full: bool) -> Self {
        if full {
            Self::Full
        } else {
            Self::Prune
        }
    }

    fn build_graph<T, K>(
        &self,
        builder: graph::GraphBuilder<K>,
        table: state_table::StateTable<T>,
    ) -> (graph::Graph<K>, Vec<T>)
    where
        T: Eq + std::hash::Hash,
    {
        let stat_list = table.to_state_list();
        match self {
            Self::Full => (builder.build_graph(), stat_list),
            Self::Prune => builder.build_graph().prune(stat_list),
        }
    }
}

pub enum Regex {
    Alternative(Vec<Regex>),
    ZeroMore(Box<Regex>),
    Optional(Box<Regex>),
    Value(Vec<usize>),
}
