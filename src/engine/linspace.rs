use crate::graph;
use crate::network;
use crate::state_table;

use std::collections::{HashMap, VecDeque};

use super::engine_utils::get_next_index;
use super::EngineConfig;

pub struct LinSpaceResult {
    pub graph: graph::Graph,
    pub states: Vec<network::State>,
    pub complete: bool,
}

pub fn compute_linear_space(
    net: &network::Network,
    obs_labels: &[usize],
    conf: &EngineConfig,
) -> LinSpaceResult {
    let mut builder = graph::GraphBuilder::new();
    let mut table = state_table::StateTable::new();
    let mut stack = VecDeque::new();
    let mut label_table = IndexTable::new();
    let begin_state = net.get_initial_state();
    let begin_index = table.insert_state(begin_state);
    label_table.insert_begin_state(begin_index);
    stack.push_front(begin_index);
    let timer = conf.timer_factory.new_timer();
    while let Some(state_index) = stack.pop_front() {
        if timer.timeout() {
            break;
        }
        let curr_state = table.get_object(state_index);
        if curr_state.is_final() && curr_state.get_index() == obs_labels.len() {
            builder.add_final_node(state_index);
        } else {
            builder.add_simple_node(state_index);
        }
        let next_state = net.step_one(curr_state);
        for (event, next_state) in next_state.into_iter() {
            let obs_index = label_table.get_index(state_index);
            if let Some(obs) = event.obs {
                if obs_index < obs_labels.len() && obs == obs_labels[obs_index] {
                    let next_state = next_state.set_index(obs_index + 1);
                    let next_index = get_next_index(next_state, &mut table, &mut stack);
                    label_table.insert_next_index_state(state_index, next_index);
                    builder.add_arc(state_index, next_index);
                }
            } else {
                let next_state = next_state.set_index(obs_index);
                let next_index = get_next_index(next_state, &mut table, &mut stack);
                builder.add_arc(state_index, next_index);
                label_table.copy_state_index(state_index, next_index);
            }
        }
    }

    let (graph, states) = conf.mode.build_graph(builder, table);
    LinSpaceResult {
        graph,
        states,
        complete: stack.is_empty(),
    }
}

impl Into<super::NetworkResult> for LinSpaceResult {
    fn into(self) -> super::NetworkResult {
        super::NetworkResult::Linspace(self)
    }
}

struct IndexTable {
    table: HashMap<usize, usize>,
}

impl IndexTable {
    fn new() -> Self {
        Self {
            table: HashMap::new(),
        }
    }

    fn insert_begin_state(&mut self, state: usize) {
        self.table.insert(state, 0);
    }

    fn insert_next_index_state(&mut self, prev: usize, state: usize) {
        let index = self.get_index(prev);
        self.table.insert(state, index + 1);
    }

    fn copy_state_index(&mut self, prev: usize, curr: usize) {
        let index = self.get_index(prev);
        self.table.insert(curr, index);
    }

    fn get_index(&self, state: usize) -> usize {
        *self.table.get(&state).unwrap()
    }
}

#[cfg(test)]
mod test {

    use super::super::EngineConfig;
    use super::super::GraphMode;
    use super::*;
    use crate::compiler::compile;
    use crate::graph::NodeKind;
    use crate::timer;
    use crate::utils::zip;
    use fsa_net_parser::parse;
    use test_utils::load_code_from_file;

    #[test]
    fn test_linspace() {
        let src_code = load_code_from_file("simple-network");
        let code = parse(&src_code).expect("`simple-network` should be syntactically correct");
        let comp_res = compile(&code).expect("`simple-network` should be semantically correct");
        let net = &comp_res.compile_network[0].net;

        let obs_labels = [1, 0];
        let config = EngineConfig::new(GraphMode::Full, timer::TimerFactory::from_value(None));

        let linspace = compute_linear_space(&net, &obs_labels, &config);
        let graph = &linspace.graph;
        let adjacent = graph.get_adjacent_list();

        assert_eq!(adjacent.len(), 9, "adjacent list: {:?}", adjacent);
        let expec_adjacent = vec![
            vec![1],
            vec![2],
            vec![3, 4],
            vec![8],
            vec![5],
            vec![6, 7],
            vec![],
            vec![],
            vec![],
        ];
        assert_eq!(adjacent, &expec_adjacent);

        let expec_index = [0, 1, 2, 2, 2, 2, 2, 2, 2];

        let states = &linspace.states;
        for (state, index) in zip(&states, &expec_index) {
            assert_eq!(state.get_index(), *index, "{:?}", state);
        }

        let expect_kind = vec![
            NodeKind::Simple,
            NodeKind::Simple,
            NodeKind::Simple,
            NodeKind::Final,
            NodeKind::Final,
            NodeKind::Simple,
            NodeKind::Final,
            NodeKind::Final,
            NodeKind::Simple,
        ];

        let node_kind = graph.get_node_kind_list();
        assert_eq!(node_kind, &expect_kind);
    }
}
