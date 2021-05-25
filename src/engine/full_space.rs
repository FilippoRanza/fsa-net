use crate::graph;
use crate::network;
use crate::state_table;

use std::collections::VecDeque;

pub fn compute_full_space(net: &network::Network) -> graph::Graph {
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

    builder.build_graph()
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn test_linspace() {
        let trans_a_a = network::Transition::new()
            .set_input(network::Event::new(0, 0))
            .add_output(network::Event::new(1, 1));
        let trans_b_a = network::Transition::new().add_output(network::Event::new(1, 1));

        let auto_a = network::Automata::new(
            0,
            0,
            vec![
                vec![network::Adjacent::new(1, trans_a_a)],
                vec![network::Adjacent::new(0, trans_b_a)],
            ],
        );

        let trans_a_b = network::Transition::new().add_output(network::Event::new(0, 0));
        let trans_b_b = network::Transition::new().set_input(network::Event::new(1, 1));
        let trans_c_b = network::Transition::new().set_input(network::Event::new(1, 1));

        let auto_b = network::Automata::new(
            0,
            1,
            vec![
                vec![network::Adjacent::new(1, trans_a_b)],
                vec![
                    network::Adjacent::new(0, trans_b_b),
                    network::Adjacent::new(1, trans_c_b),
                ],
            ],
        );

        let net = network::Network::new(
            vec![auto_a, auto_b],
            vec![network::Link::new(1, 0), network::Link::new(0, 1)],
        );

        let graph = compute_full_space(&net);

        let adjacent_list = graph.get_adjacent_list();
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
