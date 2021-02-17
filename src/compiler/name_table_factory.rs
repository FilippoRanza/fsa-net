
use super::name_table::{InsertResult, NameTable};

use fsa_net_parser::Code;
use fsa_net_parser::syntax_tree::*;

pub fn build_name_table<'a>(code: &Code<'a>) -> InsertResult<'a> {
    let name_table = NameTable::new();
    code.iter().try_fold(name_table,  collect_block_names)
}


fn collect_block_names<'a>(name_table: NameTable<'a>, block: &Block<'a>) -> InsertResult<'a> {
    match block {
        Block::Network(network) => collect_network_names(name_table, network),
        Block::Request(request) => collect_request_names(name_table, request)
    }
}

fn collect_network_names<'a>(name_table: NameTable<'a>, network: &Network<'a>) -> InsertResult<'a> {
    let name_table = name_table.insert_network(network.name, network.get_location())?;
    network.params.iter().try_fold(name_table, collect_network_params_names)
    
}

fn collect_network_params_names<'a>(name_table: NameTable<'a>, param: &NetworkParameter<'a>) -> InsertResult<'a> {
    match param {
        NetworkParameter::Automata(automata) =>  collect_automata_names(name_table, automata),
        NetworkParameter::Events(events) => collect_events_names(name_table, events),
        NetworkParameter::Link(links) => collect_links_names(name_table, links),
        NetworkParameter::ObserveLabels(labels) => collect_obs_labels(name_table, labels),
        NetworkParameter::RelevanceLabels(labels) => collect_rel_labels(name_table, labels)
    }
}

fn collect_automata_names<'a>(name_table: NameTable<'a>, automata: &Automata<'a>) -> InsertResult<'a> {
    let name_table = name_table.insert_automata(automata.name, automata.get_location())?;
    automata.params.iter().try_fold(name_table, collect_automata_params)
}

fn collect_automata_params<'a>(name_table: NameTable<'a>, param: &AutomataParameter<'a>) -> InsertResult<'a> {
    match param {
        AutomataParameter::StateDecl(state_decl) => collect_state_names(name_table, state_decl),
        AutomataParameter::Transition(trans_decl) => collect_transition_names(name_table, trans_decl)
    }
}

fn collect_state_names<'a>(name_table: NameTable<'a>, state_decl: &StateDeclaration<'a>) -> InsertResult<'a> {
    match state_decl {
        StateDeclaration::Begin(name) => name_table.insert_state(name, (0, 0)),
        StateDeclaration::State(name) => name_table.insert_state(name, (0, 0))
    }
}
 
fn collect_transition_names<'a>(name_table: NameTable<'a>, trans_decl: &TransitionDeclaration<'a>) -> InsertResult<'a> {
    name_table.insert_transition(trans_decl.name, trans_decl.get_location())
}

fn collect_events_names<'a>(name_table: NameTable<'a>, events: &Vec::<&'a str>) -> InsertResult<'a> {
    events.iter().try_fold(name_table, |nt, ev| nt.insert_event(ev, (0, 0)))
}

fn collect_links_names<'a>(name_table: NameTable<'a>, link: &Link<'a>) -> InsertResult<'a> {
    name_table.insert_link(link.name, link.get_location())
}

fn collect_obs_labels<'a>(name_table: NameTable<'a>, labels: &Vec::<&'a str>) -> InsertResult<'a> {
    labels.iter().try_fold(name_table, |nt, lbl| nt.insert_obs_label(lbl, (0, 0)))
}

fn collect_rel_labels<'a>(name_table: NameTable<'a>, labels: &Vec::<&'a str>) -> InsertResult<'a> {
    labels.iter().try_fold(name_table, |nt, lbl| nt.insert_rel_label(lbl, (0, 0)))
}


fn collect_request_names<'a>(name_table: NameTable<'a>, request: &Request<'a>) -> InsertResult<'a> {
    Ok(name_table)
}
