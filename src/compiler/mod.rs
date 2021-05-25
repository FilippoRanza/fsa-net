mod automata_connection;
mod compiler;
mod compiler_utils;
mod error;
mod link_connection;
mod name_table;
mod net_compiler;

use super::network;
use super::command;

#[derive(Debug)]
pub struct CompileResult {
    pub net: network::Network,
    pub req: command::Requests
}

pub use compiler::compile;
