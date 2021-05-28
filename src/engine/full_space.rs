use crate::graph;
use crate::network;
use crate::state_table;

use std::collections::VecDeque;

use super::engine_utils::get_next_index;

pub struct FullSpaceResult {
    pub graph: graph::Graph,
}

pub fn compute_full_space(net: &network::Network) -> FullSpaceResult {
    let mut builder = graph::GraphBuilder::new();
    let mut table = state_table::StateTable::new();
    let mut stack = VecDeque::new();
    let begin_state = net.get_initial_state();
    let begin_index = table.insert_state(begin_state);
    stack.push_front(begin_index);
    while let Some(state_index) = stack.pop_front() {
        let curr_state = table.get_object(state_index);
        if curr_state.is_final() {
            builder.add_final_node(state_index);
        } else {
            builder.add_simple_node(state_index);
        }

        let next_state = net.step_one(curr_state);
        for (_, next_state) in next_state.into_iter() {
            let next_index = get_next_index(next_state, &mut table, &mut stack);
            builder.add_arc(state_index, next_index);
        }
    }

    let graph = builder.build_graph();
    FullSpaceResult { graph }
}

impl Into<super::NetworkResult> for FullSpaceResult {
    fn into(self) -> super::NetworkResult {
        super::NetworkResult::FullSpace(self)
    }
}

#[cfg(test)]
mod test {

    use super::*;
    use crate::compiler::compile;
    use fsa_net_parser::parse;
    use test_utils::load_code_from_file;

    #[test]
    fn test_full_space() {
        let src_code = load_code_from_file("simple-network");
        let code = parse(&src_code).expect("`simple-network` should be syntactically correct");
        let comp_res = compile(&code).expect("`simple-network` should be semantically correct");
        let net = &comp_res.compile_network[0].net;

        let result = compute_full_space(&net);

        let adjacent_list = result.graph.get_adjacent_list();
        assert_eq!(adjacent_list.len(), 15);
        let expected = vec![
            vec![1],     // 0
            vec![2],     // 1
            vec![3, 4],  // 2
            vec![7, 8],  // 3
            vec![5],     // 4
            vec![0, 6],  // 5
            vec![],      // 6
            vec![9],     // 7
            vec![9],     // 8
            vec![10, 1], // 9
            vec![11],    // 10
            vec![12],    // 11
            vec![13, 8], // 12
            vec![14],    // 13
            vec![],      // 14
        ];

        assert_eq!(adjacent_list, &expected);
    }
}
