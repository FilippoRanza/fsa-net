use super::super::name_table::GlobalNameTable;
use super::super::CompileResult;
use super::result_builder::{ItemType, ResultBuilder};

use super::compile_network;
use super::compile_requests;

use fsa_net_parser::syntax_tree;
use fsa_net_parser::Code;

pub fn compile_networks(code: &Code, table: &GlobalNameTable) -> Vec<CompileResult> {
    code.iter()
        .map(|blk| compile_block(blk, table))
        .fold(ResultBuilder::new(), |builder, (name, item)| {
            builder.insert_node(name, item)
        })
        .build_result()
}

fn compile_block<'a>(
    block: &'a syntax_tree::Block,
    table: &GlobalNameTable,
) -> (&'a str, ItemType) {
    match block {
        syntax_tree::Block::Network(net) => (
            net.name,
            compile_network::compile_network(net, table).into(),
        ),
        syntax_tree::Block::Request(req) => (
            req.name,
            compile_requests::compile_requests(req, table).into(),
        ),
    }
}
