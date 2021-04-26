use super::error;
use super::graph;
use super::name_table;

use fsa_net_parser::Code;

pub fn compile<'a>(code: &Code<'a>) -> Result<(), error::CompileError<'a>> {
    let _ = name_table::build_name_table(code)?;
    graph::check_connection(code)?;

    Ok(())
}
