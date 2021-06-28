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
}

impl EngineConfig {
    pub fn new(mode: GraphMode, timer: timer::TimerFactory) -> Self {
        Self {
            mode,
            timer_factory: timer,
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

#[derive(Clone, PartialEq, Debug)]
pub enum Regex {
    Alternative(Vec<Regex>),
    ZeroMore(Box<Regex>),
    Optional(Box<Regex>),
    Chain(Vec<Regex>),
    Value(Vec<usize>),
}

impl Default for Regex {
    fn default() -> Self {
        Self::Value(vec![])
    }
}

impl Regex {
    fn is_empty(&self) -> bool {
        match self {
            Self::Alternative(re) | Self::Chain(re) => {
                re.iter().map(|r| r.is_empty()).find(|e| !e).is_some()
            }
            Self::Optional(opt) => opt.is_empty(),
            Self::Value(val) => val.len() == 0,
            Self::ZeroMore(zm) => zm.is_empty(),
        }
    }

    fn fix_empty(self) -> Option<Self> {
        let out = match self {
            Self::Alternative(alt) => {
                let len = alt.len();
                let alt: Vec<Regex> = alt.into_iter().filter_map(|re| re.fix_empty()).collect();
                match alt.len() {
                    0 => None,
                    1 => {
                        if len != alt.len() {
                            Some(Self::Optional(Box::new(Self::Chain(alt))))
                        } else {
                            Some(Self::Chain(alt))
                        }
                    }
                    _ => {
                        if len != alt.len() {
                            Some(Self::Optional(Box::new(Self::Alternative(alt))))
                        } else {
                            Some(Self::Alternative(alt))
                        }
                    }
                }
            }
            Self::Chain(re) => {
                let chain: Vec<Regex> = re.into_iter().filter_map(|r| r.fix_empty()).collect();
                if chain.len() > 0 {
                    Some(Self::Chain(chain))
                } else {
                    None
                }
            }
            Self::Optional(opt) => {
                if opt.is_empty() {
                    None
                } else {
                    let opt = opt.fix_empty();
                    if let Some(opt) = opt {
                        Some(Self::Optional(Box::new(opt)))
                    } else {
                        None
                    }
                }
            }
            Self::Value(val) => {
                if val.len() == 0 {
                    None
                } else {
                    Some(Self::Value(val))
                }
            }
            Self::ZeroMore(zm) => {
                if zm.is_empty() {
                    None
                } else {
                    let zm = zm.fix_empty();
                    if let Some(zm) = zm {
                        Some(Self::ZeroMore(Box::new(zm)))
                    } else {
                        None
                    }
                }
            }
        };
        out
    }
}
