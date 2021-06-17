use super::super::name_table::GlobalNameTable;

use crate::command::{Command, DiagnosisCommand, Requests};

use fsa_net_parser::syntax_tree;

pub fn compile_requests(req: &syntax_tree::Request, table: &GlobalNameTable) -> Requests {
    let commands = req
        .list
        .iter()
        .map(|r| compile_request(r, req.name, table))
        .collect();
    Requests::new(commands)
}

fn compile_request(
    cmd: &syntax_tree::CommandDecl,
    req_name: &str,
    table: &GlobalNameTable,
) -> Command {
    match &cmd.cmd {
        syntax_tree::Command::Space => Command::FullSpace,
        syntax_tree::Command::Linspace(labels) => {
            Command::Linspace(compile_linspace(labels, req_name, table))
        }
        syntax_tree::Command::Diagnosis(labels) => {
            Command::Diagnosis(compile_diagnosis(labels, req_name, table))
        }
    }
}

fn compile_linspace(
    labels: &syntax_tree::LinspaceCommand,
    req_name: &str,
    table: &GlobalNameTable,
) -> (Vec<usize>, Option<usize>) {
    (
        map_obs_label(&labels.name_list, req_name, table),
        convert_file_index(&labels.save_file, req_name, table),
    )
}

fn compile_diagnosis(
    label: &syntax_tree::DiagnosisCommand,
    req_name: &str,
    table: &GlobalNameTable,
) -> DiagnosisCommand {
    match label {
        syntax_tree::DiagnosisCommand::Fresh(lbls) => {
            DiagnosisCommand::Fresh(map_obs_label(&lbls.name_list, req_name, table))
        }
        syntax_tree::DiagnosisCommand::Load(file) => {
            DiagnosisCommand::Load(table.get_file_index(req_name, file.file))
        }
    }
}

fn compile_fresh_diagnosis(
    label: &syntax_tree::DiagnosisCommand,
    req_name: &str,
    table: &GlobalNameTable,
) -> Vec<usize> {
    map_obs_label(&vec![""], req_name, table)
}

fn map_obs_label(labels: &[&str], req_name: &str, table: &GlobalNameTable) -> Vec<usize> {
    labels
        .iter()
        .map(|lbl| table.get_network_name_index(req_name, lbl))
        .collect()
}

fn convert_file_index(
    file: &Option<&str>,
    net_name: &str,
    table: &GlobalNameTable,
) -> Option<usize> {
    if let Some(file) = file {
        Some(table.get_file_index(net_name, file))
    } else {
        None
    }
}
