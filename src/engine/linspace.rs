use crate::graph;
use crate::network;
use crate::state_table;

use std::collections::VecDeque;


pub struct LinSpaceResult {
    pub graph: graph::Graph
}


pub fn compute_linear_space(net: &network::Network, obs_labels: &[usize]) -> LinSpaceResult {
    let mut builder = graph::GraphBuilder::new();
    let mut table = state_table::StateTable::new();
    let mut stack = VecDeque::new();
    let begin_state = net.get_initial_state();
    let begin_index = table.insert_state(begin_state);
    stack.push_front(begin_index);
    while let Some(state_index) = stack.pop_front() {
        let curr_state = table.get_object(state_index);
        if curr_state.is_final() {
            builder.add_final_node();
        } else {
            builder.add_simple_node();
        }

        let next_state = net.step_one(curr_state);
        for (_, next_state) in next_state.into_iter() {
            let next_index = if !table.is_present(&next_state) {
                let tmp_index = table.insert_state(next_state);
                stack.push_front(tmp_index);
                tmp_index
            } else {
                table.get_index(&next_state).unwrap()
            };

            builder.add_arc(state_index, next_index);
        }
    }

    let graph = builder.build_graph();
    LinSpaceResult {
        graph
    }
}


impl Into<super::NetworkResult> for LinSpaceResult {
    fn into(self) -> super::NetworkResult {
        super::NetworkResult::Linspace(self)
    }
}

