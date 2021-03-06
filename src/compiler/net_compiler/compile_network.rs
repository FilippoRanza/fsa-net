use super::super::name_table::GlobalNameTable;
use super::graph_builder;

use crate::network;

use fsa_net_parser::syntax_tree;

pub fn compile_network(net: &syntax_tree::Network, table: &GlobalNameTable) -> network::Network {
    let mut automata_list = Vec::new();
    let mut link_list = Vec::new();

    for entry in &net.params {
        match &entry.param {
            syntax_tree::NetworkParameter::Automata(automata) => {
                let tmp = compile_automata(automata, table, net.name);
                automata_list.push(tmp);
            }
            syntax_tree::NetworkParameter::Link(link) => {
                let tmp = compile_link(link, table, net.name);
                link_list.push(tmp);
            }
            _ => {}
        }
    }
    network::Network::new(automata_list, link_list)
}

fn compile_automata(
    auto_decl: &syntax_tree::Automata,
    table: &GlobalNameTable,
    net_name: &str,
) -> network::Automata {
    let mut builder = graph_builder::GraphBuilder::new();
    let mut begin = 0;
    for decl in &auto_decl.params {
        match &decl.param {
            syntax_tree::AutomataParameter::StateDecl(state) => {
                if let syntax_tree::StateDeclaration::Begin(name) = state {
                    begin = table.get_automata_name_index(net_name, auto_decl.name, name);
                }
            }
            syntax_tree::AutomataParameter::Transition(trans) => {
                compile_transition(trans, table, net_name, auto_decl.name, &mut builder)
            }
        }
    }

    let adj_list = builder.build_graph();
    let automata_index = table.get_network_name_index(net_name, auto_decl.name);
    network::Automata::new(begin, automata_index, adj_list)
}

fn compile_transition(
    trans: &syntax_tree::TransitionDeclaration,
    table: &GlobalNameTable,
    net_name: &str,
    auto_name: &str,
    builder: &mut graph_builder::GraphBuilder,
) {
    let src_state = table.get_automata_name_index(net_name, auto_name, trans.source);
    let dst_state = table.get_automata_name_index(net_name, auto_name, trans.destination);
    let owner = table.get_network_name_index(net_name, auto_name);
    let index = table.get_automata_name_index(net_name, auto_name, trans.name);
    let out_trans = network::Transition::new(owner, index);

    let out_trans = if let Some(input) = &trans.input {
        let event = compile_event(input, table, net_name);
        out_trans.set_input(event)
    } else {
        out_trans
    };

    let out_trans = if let Some(output) = &trans.output {
        output
            .iter()
            .map(|ev| compile_event(ev, table, net_name))
            .fold(out_trans, |ot, ev| ot.add_output(ev))
    } else {
        out_trans
    };

    let out_trans = if let Some(obs) = &trans.obs_label {
        let index = table.get_network_name_index(net_name, obs);
        out_trans.set_observability(index)
    } else {
        out_trans
    };

    let out_trans = if let Some(rel) = &trans.rel_label {
        let index = table.get_network_name_index(net_name, rel);
        out_trans.set_relevance(index)
    } else {
        out_trans
    };

    builder.add_arc(src_state, dst_state, out_trans);
}

fn compile_event(
    event: &syntax_tree::Event,
    table: &GlobalNameTable,
    net_name: &str,
) -> network::Event {
    let link = table.get_network_name_index(net_name, event.link);
    let event = table.get_network_name_index(net_name, event.name);
    network::Event::new(event, link)
}

fn compile_link(
    decl: &syntax_tree::Link,
    table: &GlobalNameTable,
    net_name: &str,
) -> network::Link {
    let src = table.get_network_name_index(net_name, decl.source);
    let dst = table.get_network_name_index(net_name, decl.destination);
    network::Link::new(src, dst)
}
