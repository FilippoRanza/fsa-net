use super::EngineConfig;
use crate::graph;
use crate::network::TransEvent;
use crate::timer::Timer;

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
    run_diagnosis(
        0,
        g.get_adjacent_list(),
        g.get_node_kind_list(),
        &mut stack,
        &mut matrix,
        &timer,
    );

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
        if timer.timeout() {
            break;
        }

        let rel = next.label.get_label();
        if let Some(rel) = rel {
            stack.push(rel);
        }

        run_diagnosis(next.next, adj, node_type, stack, mat, timer);

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
        let config = EngineConfig::new(GraphMode::Full, timer::TimerFactory::from_value(None));
        let mat = diagnosis(&graph, &config).matrix.unwrap();
        assert_eq!(mat.len(), 4);
        let mut mat = mat;
        mat.sort_by_key(|v| v.len());

        let expected = vec![vec![], vec![0], vec![0, 1], vec![0, 1, 0]];
        assert_eq!(expected, mat);
    }

    impl AsLabel for Option<usize> {
        fn get_label(&self) -> Option<usize> {
            *self
        }
    }
}
