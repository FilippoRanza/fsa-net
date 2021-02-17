
use super::name_table;

use fsa_net_parser::{Code, syntax_tree};

pub fn build_name_table<'a>(code: &Code<'a>) -> Result<name_table::NameTable<'a>, name_table::RidefinitionError<'a>> {
    let mut name_table = name_table::NameTable::new();
    for block in code {
        check_block_names(block, &mut name_table)?;
    }
    Ok(name_table)
}

fn check_block_names<'a>(block: &syntax_tree::Block<'a>, name_table: &mut name_table::NameTable<'a>) -> name_table::InsertResult<'a> {
    match block {
        syntax_tree::Block::Network(network) => check_network_names(network, name_table),
        syntax_tree::Block::Request(request) => check_request_names(request, name_table)
    }
}

fn check_network_names<'a>(network: &syntax_tree::Network<'a>, name_table: &mut name_table::NameTable<'a>) -> name_table::InsertResult<'a> {
    name_table.insert_network(network.name,network.get_location())?;

    for param in &network.params {

    }
    Ok(())
}

fn check_request_names<'a>(request: &syntax_tree::Request<'a>, name_table: &mut name_table::NameTable<'a>) -> name_table::InsertResult<'a> {
    Ok(())
}





