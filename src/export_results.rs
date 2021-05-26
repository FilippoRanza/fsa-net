use crate::engine::{FullSpaceResult, NetworkResult};
use serde::Serialize;

pub fn export_results(results: Vec<NetworkResult>) -> String {
    results
        .into_iter()
        .map(export_result)
        .fold(String::new(), |acc, curr| acc + &curr)
}

fn export_result(result: NetworkResult) -> String {
    match result {
        NetworkResult::FullSpace(full_space) => export_full_space(full_space),
        NetworkResult::Linspace(_) => unimplemented!(),
    }
}

fn export_full_space(full_space: FullSpaceResult) -> String {
    let exporter = ExportFullSpace::new(full_space.graph.get_adjacent_list());
    serde_json::to_string(&exporter).unwrap()
}

#[derive(Serialize)]
struct ExportFullSpace<'a> {
    adjacent: &'a Vec<Vec<usize>>,
}

impl<'a> ExportFullSpace<'a> {
    fn new(adjacent: &'a Vec<Vec<usize>>) -> Self {
        Self { adjacent }
    }
}
