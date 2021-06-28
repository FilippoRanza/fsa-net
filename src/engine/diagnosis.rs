use super::EngineConfig;
use super::Regex;
use crate::enumerate;
use crate::graph;
use crate::network::TransEvent;
use crate::timer::Timer;
use crate::utils::{clear, zeros};
use std::collections::HashSet;

pub struct DiagnosisResult {
    pub matrix: Option<Regex>,
    pub complete: bool,
    pub timeout: bool,
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
    let regex = build_regex(g, conf);

    match regex {
        BuildResult::Regex(regex) => DiagnosisResult {
            matrix: regex,
            complete: true,
            timeout: false,
        },
        BuildResult::Timeout => DiagnosisResult {
            matrix: None,
            complete: false,
            timeout: true,
        },
    }
}

fn empty_diagnosis() -> DiagnosisResult {
    DiagnosisResult {
        matrix: Some(Regex::default()),
        complete: true,
        timeout: false,
    }
}

pub fn fail_diagnosis() -> DiagnosisResult {
    DiagnosisResult {
        matrix: None,
        complete: true,
        timeout: false,
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

macro_rules! next {
    ($name:ident) => {
        $name.iter().map(|n| n.next)
    };
}

fn lbl_to_regex<T: AsLabel>(t: &T) -> Regex {
    let vect = if let Some(lbl) = t.get_label() {
        vec![lbl]
    } else {
        vec![]
    };
    Regex::Value(vect)
}

fn build_regex<T: AsLabel>(g: &graph::Graph<T>, conf: &EngineConfig) -> BuildResult {
    let mut g = g.convert(lbl_to_regex).add_fake_nodes();
    let timer = conf.timer_factory.new_timer();
    let mut timeout = false;
    while continue_process(&g, &mut timeout, &timer) {
        let node_count = g.get_node_kind_list().len();
        let trans_count = build_in_out_count(g.get_adjacent_list(), node_count);
        if let Some(chain) = find_chain(g.get_adjacent_list(), &trans_count) {
            g = apply_chain(chain, g);
        } else if let Some(parallel) = find_parallel(g.get_adjacent_list()) {
            g = apply_parallel(parallel, g);
        } else {
            g = process_best_node(g, &trans_count);
        }
    }
    if timeout {
        BuildResult::Timeout
    } else {
        let output = g.remove_arc(0, 1).pop().unwrap().label;

        let regex = output.fix_empty();
        BuildResult::Regex(regex)
    }
}

enum BuildResult {
    Timeout,
    Regex(Option<Regex>),
}

fn continue_process<T>(g: &graph::Graph<T>, timeout: &mut bool, timer: &Timer) -> bool {
    if timer.timeout() {
        *timeout = true;
        false
    } else {
        g.trans_count() > 1
    }
}

fn process_best_node(mut g: graph::Graph<Regex>, count: &[TransCount]) -> graph::Graph<Regex> {
    let best = guess_best(&count);

    let adj = &g.get_adjacent_list()[best];
    let best_count = &count[best];
    let mut actions = Vec::with_capacity(best_count.incoming + best_count.outcoming);
    for src in g.find_origin(best) {
        for dst in next! {adj}.filter(|dst| *dst != best) {
            let act = (src, dst);
            actions.push(act);
        }
    }

    let auto_loop = find_auto_trans(&mut g, best);

    for (src, dst) in actions {
        let v1 = g.get_arc(src, best);
        let v2 = g.get_arc(best, dst);
        let regex = if let Some(auto_loop) = &auto_loop {
            let al = auto_loop.clone();
            vec![v1, Regex::ZeroMore(Box::new(al)), v2]
        } else {
            vec![v1, v2]
        };
        let regex = Regex::Chain(regex);
        //dbg!(&regex);
        g = g.add_arc(src, dst, regex);
    }

    let (g, _) = g.prune_nodes(&[best]);
    g
}

fn find_auto_trans(g: &mut graph::Graph<Regex>, node: usize) -> Option<Regex> {
    let adj = g.get_adjacent_list();
    let adj = &adj[node];
    let dst = next! { adj }.find(|n| *n == node);
    if let Some(node) = dst {
        let arcs = g.remove_arc(node, node);
        let regex = chain_regex(arcs);
        Some(regex)
    } else {
        None
    }
}

fn guess_best(trans_count: &[TransCount]) -> usize {
    let index = enumerate!(trans_count[1..trans_count.len() - 1])
        .max_by_key(|(_, tc)| tc.incoming + tc.outcoming)
        .unwrap()
        .0;
    index + 1
}

fn apply_chain(mut chain: Vec<usize>, mut g: graph::Graph<Regex>) -> graph::Graph<Regex> {
    let first = *chain.first().unwrap();
    let last = *chain.last().unwrap();
    let rm = g.chain_transaction(&chain);
    let last_index = chain.len() - 1;
    let ch = &mut chain[1..last_index];
    ch.sort_unstable();
    let (g, _, first, last) = g.remove_nodes(ch, first, last);
    let regex = Regex::Chain(rm);
    g.add_arc(first, last, regex)
}

fn apply_parallel(
    parallel: Vec<(usize, usize)>,
    mut g: graph::Graph<Regex>,
) -> graph::Graph<Regex> {
    for (s, d) in parallel {
        let regex = g.remove_arc(s, d);
        let regex = alt_regex(regex);
        g = g.add_arc(s, d, regex);
    }

    g
}

fn build_in_out_count<T>(g: &graph::AdjList<T>, node_count: usize) -> Vec<TransCount> {
    let mut output: Vec<TransCount> = zeros(node_count);
    for (src, adj) in enumerate! {g} {
        for n in adj.iter().map(|n| n.next) {
            output[src].outcoming += 1;
            output[n].incoming += 1;
        }
    }
    output
}

#[derive(Default, Debug, PartialEq)]
struct TransCount {
    incoming: usize,
    outcoming: usize,
}

fn find_chain<T>(g: &graph::AdjList<T>, count: &[TransCount]) -> Option<Vec<usize>> {
    let candidate = count
        .iter()
        .enumerate()
        .filter_map(|(i, tc)| {
            if tc.incoming == tc.outcoming && tc.incoming == 1 {
                Some(i)
            } else {
                None
            }
        })
        .collect();

    let out = (0..g.len())
        .map(|i| try_path(i, g, &candidate))
        .filter_map(|o| o)
        .max_by_key(|v| v.len());

    match out {
        Some(v) if v.len() > 1 => Some(v),
        _ => None,
    }
}

fn try_path<T>(
    curr: usize,
    g: &graph::AdjList<T>,
    candidate: &HashSet<usize>,
) -> Option<Vec<usize>> {
    let best = g
        .get(curr)
        .unwrap()
        .iter()
        .filter(|n| candidate.contains(&n.next))
        .map(|n| follow_unique_path(curr, n.next, g, candidate))
        .max_by_key(|v| v.len());
    match best {
        Some(v) if v.len() > 1 => Some(v),
        _ => None,
    }
}

fn follow_unique_path<T>(
    begin: usize,
    mut curr: usize,
    g: &graph::AdjList<T>,
    candidate: &HashSet<usize>,
) -> Vec<usize> {
    let mut vect = Vec::with_capacity(g.len());
    vect.push(begin);
    while candidate.contains(&curr) {
        vect.push(curr);
        curr = g[curr][0].next;
    }
    vect.push(curr);
    vect
}

fn find_parallel<T>(g: &graph::AdjList<T>) -> Option<Vec<(usize, usize)>> {
    let mut trans_count: Vec<bool> = zeros(g.len());
    let mut output = Vec::with_capacity(g.len());
    for (i, next) in enumerate! {g} {
        trans_count = clear(trans_count);
        if next.len() < 2 {
            continue;
        }
        for n in next.iter().map(|n| n.next) {
            if trans_count[n] {
                output.push((i, n));
                break;
            } else {
                trans_count[n] = true;
            }
        }
    }
    if output.len() > 0 {
        Some(output)
    } else {
        None
    }
}

fn alt_regex<I>(reg_iter: I) -> Regex
where
    I: IntoIterator<Item = graph::Arc<Regex>>,
{
    Regex::Alternative(vectorize_regex(reg_iter))
}

fn chain_regex<I>(reg_iter: I) -> Regex
where
    I: IntoIterator<Item = graph::Arc<Regex>>,
{
    Regex::Chain(vectorize_regex(reg_iter))
}

fn vectorize_regex<I>(reg_iter: I) -> Vec<Regex>
where
    I: IntoIterator<Item = graph::Arc<Regex>>,
{
    reg_iter.into_iter().map(|a| a.label).collect()
}

#[cfg(test)]
mod test {

    use super::super::GraphMode;
    use super::*;
    use crate::timer;

    use crate::graph::GraphBuilder;

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
        let regex = diagnosis(&graph, &config).matrix.unwrap();
        let expected = Regex::Chain(vec![Regex::Optional(Box::new(Regex::Chain(vec![
            Regex::Chain(vec![
                Regex::Value(vec![0]),
                Regex::Optional(Box::new(Regex::Chain(vec![Regex::Chain(vec![
                    Regex::Value(vec![1]),
                    Regex::Optional(Box::new(Regex::Chain(vec![Regex::Chain(vec![
                        Regex::Value(vec![0]),
                    ])]))),
                ])]))),
            ]),
        ])))]);

        assert_eq!(expected, regex);
    }

    #[test]
    fn test_chain() {
        let graph = build_sample_graph();
        let count = build_in_out_count(graph.get_adjacent_list(), graph.get_adjacent_list().len());

        let chain = find_chain(graph.get_adjacent_list(), &count).unwrap();

        let expect = [0, 4, 2, 6, 7];
        assert_eq!(chain, &expect);
    }

    #[test]
    fn test_transition_count() {
        let graph = build_sample_graph();
        let graph = graph.add_fake_nodes();
        let count = build_in_out_count(graph.get_adjacent_list(), graph.get_adjacent_list().len());
        let expected = [
            TransCount::new(0, 1),
            TransCount::new(1, 2),
            TransCount::new(1, 2),
            TransCount::new(1, 1),
            TransCount::new(4, 1),
            TransCount::new(1, 1),
            TransCount::new(2, 3),
            TransCount::new(1, 1),
            TransCount::new(5, 0),
            TransCount::new(1, 5),
            TransCount::new(0, 0),
        ];
        assert_eq!(count, &expected);
    }

    #[test]
    fn test_find_parallel() {
        let graph = build_sample_graph().add_fake_nodes();
        let parallel = find_parallel(graph.get_adjacent_list()).unwrap();
        let expected = [(6, 4), (9, 8)];
        assert_eq!(parallel.len(), expected.len());
        for ex in &expected {
            assert!(parallel.contains(ex), "{:?}\n{:?}", parallel, ex);
        }
    }

    fn build_sample_graph() -> graph::Graph<()> {
        let mut builder = GraphBuilder::new();

        for i in 0..=8 {
            builder.add_simple_node(i);
        }

        let arcs = [
            (0, 1),
            (0, 4),
            (1, 5),
            (1, 8),
            (2, 6),
            (3, 7),
            (4, 2),
            (5, 3),
            (5, 3),
            (5, 3),
            (6, 7),
            (8, 3),
            (8, 5),
            (8, 7),
            (8, 7),
            (8, 7),
        ];
        for (s, d) in &arcs {
            builder.add_arc(*s, *d, ());
        }

        builder.build_graph()
    }
    impl TransCount {
        fn new(incoming: usize, outcoming: usize) -> Self {
            Self {
                incoming,
                outcoming,
            }
        }
    }
}
