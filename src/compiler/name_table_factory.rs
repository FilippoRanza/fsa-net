use super::name_table::*;

use fsa_net_parser::syntax_tree::*;
use fsa_net_parser::Code;

pub fn build_name_table<'a>(code: &Code<'a>) -> Result<GlobalNameTable<'a>, NameError<'a>> {
    let name_table = GlobalNameTable::new();
    let name_table = code.iter().try_fold(name_table, |nt, curr| match curr {
        Block::Network(net) => collect_network(nt, net),
        Block::Request(_req) => Ok(nt),
    })?;

    name_table.validate()
}

fn collect_network<'a>(nt: GlobalNameTable<'a>, net: &Network<'a>) -> GlobalNameResult<'a> {
    let nt = nt.add_network(net.name, net.get_location())?;
    let nt = net.params.iter().try_fold(nt, collect_net_param)?;
    Ok(nt.exit_network())
}

fn collect_net_param<'a>(
    nt: GlobalNameTable<'a>,
    param: &NetworkParameter<'a>,
) -> GlobalNameResult<'a> {
    match param {
        NetworkParameter::Automata(automata) => collect_automata(nt, automata),
        NetworkParameter::Events(events) => events
            .iter()
            .try_fold(nt, |nt, ev| nt.add_event(ev, (0, 0))),
        NetworkParameter::ObserveLabels(labels) => labels
            .iter()
            .try_fold(nt, |nt, lbl| nt.add_rel_label(lbl, (0, 0))),
        NetworkParameter::RelevanceLabels(labels) => labels
            .iter()
            .try_fold(nt, |nt, lbl| nt.add_obs_label(lbl, (0, 0))),
        NetworkParameter::Link(link) => nt.add_link(link.name, link.get_location()),
    }
}

fn collect_automata<'a>(nt: GlobalNameTable<'a>, automata: &Automata<'a>) -> GlobalNameResult<'a> {
    let nt = nt.add_automata(automata.name, automata.get_location())?;
    let nt = automata
        .params
        .iter()
        .try_fold(nt, collect_automata_param)?;
    Ok(nt.exit_automata())
}

fn collect_automata_param<'a>(
    nt: GlobalNameTable<'a>,
    param: &AutomataParameter<'a>,
) -> GlobalNameResult<'a> {
    match param {
        AutomataParameter::StateDecl(state) => match state {
            StateDeclaration::Begin(state) => nt.add_begin(state, (0, 0)),
            StateDeclaration::State(state) => nt.add_state(state, (0, 0)),
        },
        AutomataParameter::Transition(trans) => nt.add_transition(trans.name, trans.get_location()),
    }
}

#[cfg(test)]
mod test {}
