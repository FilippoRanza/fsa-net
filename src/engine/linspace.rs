use crate::graph;
use crate::network;

fn compute_linspace(net: &network::Network) -> graph::Graph<network::State> {
    let mut builder = graph::GraphBuilder::new();

    builder.build_graph()
}
