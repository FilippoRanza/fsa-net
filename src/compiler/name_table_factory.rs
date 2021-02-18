use super::name_table::{GlobalNameTable, NameError};

use fsa_net_parser::syntax_tree::*;
use fsa_net_parser::Code;

pub fn build_name_table<'a>(code: &Code<'a>) -> Result<GlobalNameTable<'a>, NameError<'a>> {
    let name_table = GlobalNameTable::new();
    let name_table = code.iter().try_fold(name_table, |nt, curr| match curr {
        Block::Network(net) => nt.insert_network(net.name, net.get_location()),
        Block::Request(req) => nt.insert_request(req.name, req.get_location()),
    })?;
    Ok(name_table)
}



