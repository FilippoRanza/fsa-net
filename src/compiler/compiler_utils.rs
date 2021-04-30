use fsa_net_parser::syntax_tree::{Block, Network};

pub fn is_network<'b, 'a: 'b>(blk: &'b Block<'a>) -> Option<&'b Network<'a>> {
    match blk {
        Block::Network(net) => Some(net),
        Block::Request(_) => None,
    }
}
