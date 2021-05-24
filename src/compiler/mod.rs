mod automata_connection;
mod compile_network;
mod compiler;
mod compiler_utils;
mod error;
mod link_connection;
mod name_table;

use super::network;

#[derive(Debug)]
pub struct CompileResult {
    pub net: network::Network,
}

pub use compiler::compile;
