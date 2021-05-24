use super::compiler_utils;
use super::name_table::GlobalNameTable;
use super::CompileResult;

use crate::network;
use crate::utils::zeros;

use fsa_net_parser::syntax_tree;
use fsa_net_parser::Code;

pub fn compile_networks(code: &Code, table: &GlobalNameTable) -> Vec<CompileResult> {
    code.iter()
        .filter_map(compiler_utils::is_network)
        .map(|net| compile_network(net, table))
        .collect()
}

fn compile_network(net: &syntax_tree::Network, table: &GlobalNameTable) -> CompileResult {
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
    let network = network::Network::new(automata_list, link_list);
    CompileResult { net: network }
}

fn compile_automata(
    auto_decl: &syntax_tree::Automata,
    table: &GlobalNameTable,
    net_name: &str,
) -> network::Automata {
    let mut builder = GraphBuilder::new();
    let mut begin = 0;
    for decl in &auto_decl.params {
        match &decl.param {
            syntax_tree::AutomataParameter::StateDecl(state) => {
                if let syntax_tree::StateDeclaration::Begin(name) = state {
                    begin = table.get_automata_name_index(net_name, auto_decl.name, name);
                } 
            }
            syntax_tree::AutomataParameter::Transition(trans) => compile_transition(trans, table, net_name, auto_decl.name, &mut builder)
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
    builder: &mut GraphBuilder,
) {
    let src_state = table.get_automata_name_index(net_name, auto_name, trans.source);
    let dst_state = table.get_automata_name_index(net_name, auto_name, trans.destination);

    let out_trans = network::Transition::new();

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

struct GraphBuilder {
    node_count: usize,
    arc_list: Vec<(usize, usize, network::Transition)>,
}

impl GraphBuilder {
    fn new() -> Self {
        Self {
            node_count: 0,
            arc_list: Vec::new(),
        }
    }

    fn add_arc(&mut self, src: usize, dst: usize, trans: network::Transition) {
        self.node_count = self.get_node_count(src, dst);
        let tmp = (src, dst, trans);
        self.arc_list.push(tmp);
    }

    fn build_graph(self) -> Vec<Vec<network::Adjacent>> {
        let mut output: Vec<Vec<network::Adjacent>> = zeros(self.node_count);

        for (src, dst, trans) in self.arc_list.into_iter() {
            let adj = network::Adjacent::new(dst, trans);
            output[src].push(adj);
        }

        output
    }

    fn get_node_count(&self, src: usize, dst: usize) -> usize {
        let max = if src > dst { src } else { dst };

        if max >= self.node_count {
            max + 1
        } else {
            self.node_count
        }
    }
}


