use super::EngineConfig;
use super::Regex;
use crate::graph;
use crate::network::TransEvent;
use crate::timer::Timer;
use crate::utils::zeros;
use std::collections::HashSet;

type Stack = Vec<usize>;
pub type Matrix = Vec<Vec<usize>>;

pub struct DiagnosisResult {
    pub matrix: Option<Matrix>,
    pub complete: bool,
}

pub fn diagnosis<T>(g: &graph::Graph<T>, conf: &EngineConfig) -> DiagnosisResult
where
    T: AsLabel,
{
    let node_count = g.get_adjacent_list().len();
    if node_count == 0 {
        empty_diagnosis()
    } else {
        real_diagnosis(g, conf)
    }
}

fn real_diagnosis<T>(g: &graph::Graph<T>, conf: &EngineConfig) -> DiagnosisResult
where
    T: AsLabel,
{
    let node_count = g.get_adjacent_list().len();
    let mut stack = Vec::with_capacity(node_count);
    let mut matrix = Vec::new();
    let timer = conf.timer_factory.new_timer();
    let mut seen = zeros(node_count);
    run_diagnosis(
        0,
        g.get_adjacent_list(),
        g.get_node_kind_list(),
        &mut seen,
        &mut stack,
        &mut matrix,
        &timer,
    );

    let matrix = if conf.deduplicate {
        let matrix: HashSet<Vec<usize>> = matrix.into_iter().collect();
        matrix.into_iter().collect()
    } else {
        matrix
    };

    DiagnosisResult {
        matrix: Some(matrix),
        complete: true,
    }
}

fn empty_diagnosis() -> DiagnosisResult {
    DiagnosisResult {
        matrix: Some(vec![]),
        complete: true,
    }
}

pub fn fail_diagnosis() -> DiagnosisResult {
    DiagnosisResult {
        matrix: None,
        complete: true,
    }
}

fn run_diagnosis<T>(
    curr: usize,
    adj: &graph::AdjList<T>,
    node_type: &[graph::NodeKind],
    seen: &mut [bool],
    stack: &mut Stack,
    mat: &mut Matrix,
    timer: &Timer,
) where
    T: AsLabel,
{
    if let graph::NodeKind::Final = node_type[curr] {
        let tmp = stack.clone();
        mat.push(tmp);
    }
    for next in &adj[curr] {
        if seen[next.next] {
            continue;
        } else {
            seen[next.next] = true;
        }
        if timer.timeout() {
            break;
        }

        let rel = next.label.get_label();
        if let Some(rel) = rel {
            stack.push(rel);
        }

        run_diagnosis(next.next, adj, node_type, seen, stack, mat, timer);

        if rel.is_some() {
            stack.pop();
        }
    }
}

impl Into<super::NetworkResult> for DiagnosisResult {
    fn into(self) -> super::NetworkResult {
        super::NetworkResult::Diagnosis(self)
    }
}

pub trait AsLabel {
    fn get_label(&self) -> Option<usize>;
}

impl AsLabel for TransEvent {
    fn get_label(&self) -> Option<usize> {
        self.rel
    }
}

impl AsLabel for Option<usize> {
    fn get_label(&self) -> Option<usize> {
        *self
    }
}

fn convert_graph<T: AsLabel>(g: &graph::Graph<T>) -> graph::Graph<Regex> {
    fn lbl_to_vect<T: AsLabel>(t: &T) -> Vec<usize> {
        if let Some(lbl) = t.get_label() {
            vec![lbl]
        } else {
            vec![]
        }
    }
    g.convert(|lbl| Regex::Value(lbl_to_vect(lbl)))
}

fn build_regex<T: AsLabel>(g: &graph::Graph<T>) -> Regex {
    let mut g = convert_graph(g);
    while g.get_adjacent_list().len() > 1 {
        if let Some(chain) = find_chain() {
            g = g.prune_nodes(&chain);
        } else if let Some(parallel) = find_parallel() {
            g = g.prune_nodes(&parallel);
        }
    }
    g.take_label().unwrap()
}

fn find_chain() -> Option<Vec<usize>> {
    None
}

fn find_parallel() -> Option<Vec<usize>> {
    None
}

#[cfg(test)]
mod test {

    use super::super::GraphMode;
    use super::*;
    use crate::timer;

    #[test]
    fn test_diagnosis() {
        let mut builder = graph::GraphBuilder::new();

        let states = [false, false, false, true, true, false, true, true];
        for (i, s) in states.iter().enumerate() {
            if *s {
                builder.add_final_node(i);
            } else {
                builder.add_simple_node(i);
            }
        }

        let arcs = [
            (0, 1, None),
            (1, 2, None),
            (2, 3, Some(0)),
            (2, 4, None),
            (3, 5, Some(1)),
            (5, 6, Some(0)),
            (5, 7, None),
        ];

        for (s, d, l) in &arcs {
            builder.add_arc(*s, *d, *l);
        }

        let graph = builder.build_graph();
        let config = EngineConfig::new(
            GraphMode::Full,
            timer::TimerFactory::from_value(None),
            false,
        );
        let mat = diagnosis(&graph, &config).matrix.unwrap();
        assert_eq!(mat.len(), 4);
        let mut mat = mat;
        mat.sort_by_key(|v| v.len());

        let expected = vec![vec![], vec![0], vec![0, 1], vec![0, 1, 0]];
        assert_eq!(expected, mat);
    }
}
