mod automata_connection;
mod compiler;
mod compiler_utils;
mod error;
mod link_connection;
mod name_table;
mod net_compiler;

use super::command;
use super::network;

#[derive(Debug)]
pub struct CompileResult {
    pub net: network::Network,
    pub req: command::Requests,
}

pub use compiler::compile;
