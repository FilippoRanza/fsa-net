use super::super::name_table::GlobalNameTable;

use crate::command::{Command, Requests};


use fsa_net_parser::syntax_tree;



pub fn compile_requests(req: &syntax_tree::Request, table: &GlobalNameTable) -> Requests {
    let commands = req.list.iter().map(|r| compile_request(r, table)).collect();
    Requests::new(commands)
}

fn compile_request(cmd: &syntax_tree::CommandDecl, table: &GlobalNameTable) -> Command {
    match &cmd.cmd {
        syntax_tree::Command::Space => Command::FullSpace,
        syntax_tree::Command::Linspace(labels) => Command::Linspace(compile_linspace(labels, table)),
        syntax_tree::Command::Diagnosis(labels) => Command::Diagnosis(compile_diagnosis(labels, table))
    }
}

fn compile_linspace(labels: &syntax_tree::LinspaceCommand, table: &GlobalNameTable) -> Vec<usize> {
    vec![]
}

fn compile_diagnosis(label: &syntax_tree::DiagnosisCommand, table: &GlobalNameTable) -> Vec<usize> {
    vec![]
}