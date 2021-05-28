mod automata_connection;
mod compiler;
mod compiler_utils;
mod error;
mod index_name_table;
mod link_connection;
mod name_table;
mod net_compiler;

use super::command;
use super::network;

#[derive(Debug)]
pub struct CompileResult<'a> {
    pub compile_network: Vec<CompileNetwork>,
    pub index_table: GlobalIndexTable<'a>,
}

#[derive(Debug)]
pub struct CompileNetwork {
    pub net: network::Network,
    pub req: command::Requests,
}

pub use compiler::compile;
pub use index_name_table::GlobalIndexTable;
