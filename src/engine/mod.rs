mod engine_utils;
mod full_space;
mod linspace;
mod run;

pub use full_space::FullSpaceResult;
pub use linspace::LinSpaceResult;
pub use run::run;

use crate::graph;
use crate::state_table;

pub enum NetworkResult {
    FullSpace(full_space::FullSpaceResult),
    Linspace(linspace::LinSpaceResult),
}

pub enum GraphMode {
    Prune,
    Full
}

impl GraphMode {
    pub fn from_flag(full: bool) -> Self {
        if full {
            Self::Full
        } else {
            Self::Prune
        }
    }

    fn build_graph<T>(&self, builder: graph::GraphBuilder, table: state_table::StateTable<T>) -> (graph::Graph, Vec<T>) 
    where T: Eq + std::hash::Hash
    {
        let stat_list = table.to_state_list();
        match self {
            Self::Full => (builder.build_graph(), stat_list),
            Self::Prune => builder.build_graph().prune(stat_list)
        }
    }
}
