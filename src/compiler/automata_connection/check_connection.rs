use super::GraphBuilder;
use fsa_net_parser::syntax_tree::*;
use fsa_net_parser::Code;

type GraphResult<'a> = Result<(), super::GraphError<'a>>;
pub fn check_connection<'a>(code: &Code<'a>) -> GraphResult<'a> {
    code.iter().try_fold((), |_, curr| match curr {
        Block::Network(net) => check_network(net),
        Block::Request(_) => Ok(()),
    })
}

fn check_network<'a>(network: &Network<'a>) -> GraphResult<'a> {
    network
        .params
        .iter()
        .try_fold((), |_, param| check_parameters(param))
}

fn check_parameters<'a>(param: &NetworkParameterDecl<'a>) -> GraphResult<'a> {
    match &param.param {
        NetworkParameter::Automata(automata) => check_automata(automata),
        _ => Ok(()),
    }
}

fn check_automata<'a>(automata: &Automata<'a>) -> GraphResult<'a> {
    let graph_builder = automata
        .params
        .iter()
        .fold(GraphBuilder::new(), chain_graph_builder);
    graph_builder.build_graph().breadth_first_search()
}

fn chain_graph_builder<'a>(
    builder: GraphBuilder<'a>,
    param: &AutomataParameterDecl<'a>,
) -> GraphBuilder<'a> {
    match &param.param {
        AutomataParameter::StateDecl(state) => add_node(builder, state),
        AutomataParameter::Transition(trans) => builder.add_link(trans.source, trans.destination),
    }
}

fn add_node<'a>(builder: GraphBuilder<'a>, state: &StateDeclaration<'a>) -> GraphBuilder<'a> {
    match state {
        StateDeclaration::Begin(name) => builder.add_node(name).set_begin(name),
        StateDeclaration::State(name) => builder.add_node(name),
    }
}
