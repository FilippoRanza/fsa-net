use crate::utils::{auto_sort, zeros};
use serde::{Deserialize, Serialize};

pub type AdjList<T> = Vec<Vec<Arc<T>>>;

#[derive(Debug, Serialize, Deserialize)]
pub struct Arc<T> {
    pub label: T,
    pub next: usize,
}

impl<T> Arc<T> {
    fn new(next: usize, label: T) -> Self {
        Self { next, label }
    }

    fn remap(mut self, mapper: &[Option<usize>], collect: &mut Vec<Arc<T>>) -> Option<Self> {
        let map = mapper[self.next];
        if let Some(next) = map {
            self.next = next;
            Some(self)
        } else {
            collect.push(self);
            None
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Graph<T> {
    nodes: Vec<NodeKind>,
    adjacent: AdjList<T>,
}

impl<'a, T> Graph<T> {
    pub fn get_adjacent_list(&'a self) -> &'a AdjList<T> {
        &self.adjacent
    }

    pub fn get_node_kind_list(&self) -> &Vec<NodeKind> {
        &self.nodes
    }

    pub fn prune<K>(self, states: Vec<K>) -> (Self, Vec<K>) {
        let prune = prune_list(&self.adjacent, &self.nodes);
        let output_graph = self.prune_nodes(&prune).0;
        let states = filter_by_index(states, &prune);
        (output_graph, states)
    }

    pub fn chain_transaction(&mut self, nodes: &[usize]) -> Vec<T> {
        let mut output = Vec::with_capacity(nodes.len() - 1);
        let src = nodes[0];
        let dst = nodes[1];
        let v = &mut self.adjacent[src];
        let index = find_by_next(v, dst).unwrap();
        let next = v.remove(index).label;
        output.push(next);
        for src in &nodes[1..nodes.len() - 1] {
            let v = &mut self.adjacent[*src];
            let next = v.pop().unwrap().label;
            output.push(next);
        }

        output
    }

    pub fn remove_nodes(
        self,
        prune_list: &[usize],
        src: usize,
        dst: usize,
    ) -> (Self, Vec<Arc<T>>, usize, usize) {
        let nodes = filter_by_index(self.nodes, &prune_list);
        let remapper = IndexRemap::new(self.adjacent.len(), prune_list);
        let (adjacent, remove) = remapper.remap_indexes(self.adjacent);
        let adjacent = filter_by_index(adjacent, &prune_list);
        let src = remapper.remap_node(src).unwrap();
        let dst = remapper.remap_node(dst).unwrap();

        (Self { adjacent, nodes }, remove, src, dst)
    }

    pub fn prune_nodes(self, prune_list: &[usize]) -> (Self, Vec<Arc<T>>) {
        let nodes = filter_by_index(self.nodes, &prune_list);
        let remapper = IndexRemap::new(self.adjacent.len(), prune_list);
        let (adjacent, remove) = remapper.remap_indexes(self.adjacent);
        let adjacent = filter_by_index(adjacent, &prune_list);
        (Self { adjacent, nodes }, remove)
    }

    pub fn add_arc(mut self, src: usize, dst: usize, value: T) -> Self {
        let arc = Arc::new(dst, value);
        self.adjacent[src].push(arc);
        self
    }

    pub fn remove_arc(&mut self, src: usize, dst: usize) -> Vec<Arc<T>> {
        let v = &mut self.adjacent[src];
        let mut output = Vec::with_capacity(v.len());
        while let Some(i) = find_by_next(&v, dst) {
            let arc = v.remove(i);
            output.push(arc);
        }
        output
    }

    pub fn convert<F, K>(&self, f: F) -> Graph<K>
    where
        F: Fn(&T) -> K,
    {
        let adj = self
            .adjacent
            .iter()
            .map(|vec| vec.iter().map(|a| Arc::new(a.next, f(&a.label))).collect())
            .collect();
        Graph {
            nodes: self.nodes.clone(),
            adjacent: adj,
        }
    }

    pub fn find_origin(&'a self, node: usize) -> impl Iterator<Item = usize> + 'a {
        self.adjacent
            .iter()
            .enumerate()
            .filter(move |(_, adj)| find_by_next(adj, node).is_some())
            .map(|(i, _)| i)
            .filter(move |i| *i != node)
    }

    pub fn trans_count(&self) -> usize {
        self.adjacent.iter().map(|adj| adj.len()).sum()
    }
}

impl<T> Graph<T>
where
    T: Default,
{
    pub fn add_fake_nodes(self) -> Self {
        let mut adjacent = Vec::with_capacity(self.adjacent.len() + 2);
        adjacent.push(vec![Arc::new(1, T::default())]);
        let final_node = self.adjacent.len() + 1;
        for (i, adj) in self.adjacent.into_iter().enumerate() {
            let mut adj = adj;
            for n in adj.iter_mut() {
                n.next += 1;
            }
            if let NodeKind::Final = self.nodes[i] {
                let arc = Arc::new(final_node, T::default());
                adj.push(arc);
            }
            adjacent.push(adj);
        }
        adjacent.push(vec![]);

        let mut nodes: Vec<NodeKind> = (0..adjacent.len() - 1).map(|_| NodeKind::Simple).collect();
        nodes.push(NodeKind::Final);

        Self { adjacent, nodes }
    }
}

impl<T> Graph<T>
where
    T: Clone,
{
    pub fn get_arc(&self, src: usize, dst: usize) -> T {
        let v = &self.adjacent[src];
        if let Some(i) = find_by_next(&v, dst) {
            v[i].label.clone()
        } else {
            panic!()
        }
    }
}

impl<'a, T> Graph<T>
where
    T: Serialize + Deserialize<'a>,
{
    pub fn save(self) -> String {
        serde_json::to_string(&self).unwrap()
    }

    pub fn load(s: &'a str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(s)
    }
}

struct IndexRemap {
    remap: Vec<Option<usize>>,
}

impl IndexRemap {
    fn new(count: usize, prune: &[usize]) -> Self {
        let remap = build_index_remap(count, prune);
        Self { remap }
    }

    fn remap_indexes<T>(&self, adj: AdjList<T>) -> (AdjList<T>, Vec<Arc<T>>) {
        let mut remove_arcs = Vec::new();
        (
            adj.into_iter()
                .map(|v| {
                    let (adj, mut remove) = remap_adjacent(v, &self.remap);
                    remove_arcs.append(&mut remove);
                    adj
                })
                .collect(),
            remove_arcs,
        )
    }

    fn remap_node(&self, node: usize) -> Option<usize> {
        self.remap[node]
    }
}

fn remap_adjacent<T>(iter: Vec<Arc<T>>, remap: &[Option<usize>]) -> (Vec<Arc<T>>, Vec<Arc<T>>) {
    let mut vect = Vec::new();
    (
        iter.into_iter()
            .filter_map(|i| i.remap(remap, &mut vect))
            .collect(),
        vect,
    )
}

fn build_index_remap(len: usize, prune: &[usize]) -> Vec<Option<usize>> {
    let mut curr = 0;
    let mut filter = FilterByIndex::new(prune);
    (0..len)
        .map(|i| (i, i))
        .map(|i| filter.should_remove(i))
        .map(|i| {
            if let Some(i) = i {
                Some(i - curr)
            } else {
                curr += 1;
                None
            }
        })
        .collect()
}

fn filter_by_index<I>(iter: I, indexes: &[usize]) -> Vec<I::Item>
where
    I: IntoIterator,
{
    let mut filter = FilterByIndex::new(indexes);
    iter.into_iter()
        .enumerate()
        .filter_map(|i| filter.should_remove(i))
        .collect()
}

struct FilterByIndex<'a> {
    index: usize,
    buff: &'a [usize],
}

impl<'a> FilterByIndex<'a> {
    fn new(buff: &'a [usize]) -> Self {
        FilterByIndex { index: 0, buff }
    }

    fn should_remove<T>(&mut self, elem: (usize, T)) -> Option<T> {
        let (index, item) = elem;
        if self.index >= self.buff.len() {
            Some(item)
        } else if index == self.buff[self.index] {
            self.index += 1;
            None
        } else {
            Some(item)
        }
    }
}

pub struct GraphBuilder<T> {
    nodes_list: Vec<(usize, usize, T)>,
    node_kind: Vec<(NodeKind, usize)>,
}

impl<T> GraphBuilder<T> {
    pub fn new() -> Self {
        Self {
            nodes_list: Vec::new(),
            node_kind: Vec::new(),
        }
    }

    pub fn add_final_node(&mut self, index: usize) {
        self.add_node(index, NodeKind::Final);
    }

    pub fn add_simple_node(&mut self, index: usize) {
        self.add_node(index, NodeKind::Simple);
    }

    fn add_node(&mut self, index: usize, kind: NodeKind) {
        self.node_kind.push((kind, index));
    }

    pub fn add_arc(&mut self, src: usize, dst: usize, t: T) {
        self.nodes_list.push((src, dst, t));
    }

    pub fn build_graph(self) -> Graph<T> {
        let node_count = self.node_kind.len();
        let mut adjacent: AdjList<T> = zeros(node_count);
        for (s, d, t) in self.nodes_list.into_iter() {
            if s < node_count && d < node_count {
                let arc = Arc::new(d, t);
                adjacent[s].push(arc);
            }
        }

        let nodes = auto_sort(&mut self.node_kind.into_iter());
        Graph { nodes, adjacent }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub enum NodeKind {
    Simple,
    Final,
}

fn prune_list<T>(adj: &AdjList<T>, kind_list: &[NodeKind]) -> Vec<usize> {
    let mut reach: Vec<bool> = kind_list
        .iter()
        .map(|k| match k {
            NodeKind::Final => true,
            NodeKind::Simple => false,
        })
        .collect();
    let mut seen = reach.clone();

    for node in 0..adj.len() {
        if !reach[node] {
            make_prune_list(node, adj, &mut seen, &mut reach);
            for s in seen.iter_mut() {
                *s = false;
            }
        }
    }

    reach
        .into_iter()
        .enumerate()
        .filter_map(|(i, s)| if s { None } else { Some(i) })
        .collect()
}

fn make_prune_list<T>(
    curr: usize,
    adj: &AdjList<T>,
    seen: &mut Vec<bool>,
    reach: &mut Vec<bool>,
) -> bool {
    if seen[curr] {
        reach[curr]
    } else if reach[curr] {
        seen[curr] = true;
        true
    } else {
        seen[curr] = true;
        let next = &adj[curr];
        let next = next
            .iter()
            .find(|curr| make_prune_list(curr.next, adj, seen, reach));
        let stat = next.is_some();
        reach[curr] = stat;
        stat
    }
}

fn find_by_next<T>(v: &[Arc<T>], val: usize) -> Option<usize> {
    v.into_iter()
        .enumerate()
        .find(|(_, n)| n.next == val)
        .map(|(i, _)| i)
}

#[cfg(test)]
mod test {

    use super::*;
    use crate::enumerate;

    #[test]
    fn test_graph_builder() {
        let mut builder = GraphBuilder::new();

        let nodes = ['a', 'b', 'c', 'd', 'e'];
        for (i, _) in nodes.iter().enumerate() {
            builder.add_simple_node(i);
        }

        let arcs = [
            ('a', 'c'),
            ('a', 'd'),
            ('a', 'b'),
            ('c', 'b'),
            ('c', 'd'),
            ('d', 'b'),
            ('d', 'e'),
        ];
        for (s, d) in &arcs {
            // terrible :-(
            let s = nodes.iter().position(|n| n == s).unwrap();
            let d = nodes.iter().position(|n| n == d).unwrap();
            builder.add_arc(s, d, ());
        }

        let graph = builder.build_graph();

        for (i, _) in nodes.iter().enumerate() {
            match graph.nodes[i] {
                NodeKind::Simple => {}
                NodeKind::Final => panic!("NodeKind should be simple"),
            }
        }
        assert_eq!(graph.nodes.len(), nodes.len());
        assert_eq!(graph.adjacent.len(), nodes.len());

        assert_eq!(graph.adjacent[0], vec![2, 3, 1]);
        assert_eq!(graph.adjacent[1], Vec::<usize>::new());
        assert_eq!(graph.adjacent[2], vec![1, 3]);
        assert_eq!(graph.adjacent[3], vec![1, 4]);
        assert_eq!(graph.adjacent[4], Vec::<usize>::new());
    }

    #[test]
    fn test_prune_list() {
        let graph = build_test_graph();
        let prune = prune_list(&graph.adjacent, &graph.nodes);
        assert_eq!(prune, vec![1, 7, 10, 12, 13, 14, 15]);
    }
    #[test]
    fn test_prune_list_2() {
        let mut builder = GraphBuilder::new();

        let node_type = [
            true, false, false, true, false, true, true, false, false, false, false, false, false,
            false, false,
        ];
        assert_eq!(node_type.len(), 15);

        for (i, nt) in node_type.iter().enumerate() {
            if *nt {
                builder.add_final_node(i);
            } else {
                builder.add_simple_node(i);
            }
        }

        let arcs = [
            (0, 1),
            (1, 2),
            (2, 3),
            (2, 6),
            (3, 4),
            (4, 5),
            (4, 0),
            (6, 7),
            (6, 12),
            (7, 8),
            (8, 9),
            (8, 1),
            (9, 10),
            (10, 11),
            (11, 12),
            (12, 8),
            (11, 13),
            (13, 14),
        ];

        for (s, d) in &arcs {
            builder.add_arc(*s, *d, ());
        }

        let graph = builder.build_graph();
        let prune = prune_list(&graph.adjacent, &graph.nodes);
        assert_eq!(prune, vec![13, 14], "{:?}", prune);
    }

    #[test]
    fn test_graph_prune_list() {
        let (graph, _) = build_test_graph().prune(vec![0]);
        let expected_nodes = vec![
            NodeKind::Final,
            NodeKind::Simple,
            NodeKind::Simple,
            NodeKind::Final,
            NodeKind::Simple,
            NodeKind::Simple,
            NodeKind::Simple,
            NodeKind::Simple,
            NodeKind::Simple,
        ];
        assert_eq!(graph.nodes, expected_nodes);

        let expected_adjacent = vec![
            vec![1, 2, 6],
            vec![3],
            vec![5],
            vec![],
            vec![3],
            vec![4],
            vec![7],
            vec![8],
            vec![0],
        ];
        assert_eq!(graph.adjacent, expected_adjacent);

        let prune = prune_list(&graph.adjacent, &graph.nodes);
        assert_eq!(prune.len(), 0);
    }

    #[test]
    fn test_filter_by_index() {
        let items = vec!['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i'];
        let remove_index = [2, 4, 5, 7];
        let filter_items = filter_by_index(items, &remove_index);
        assert_eq!(filter_items, &['a', 'b', 'd', 'g', 'i']);
    }

    #[test]
    fn test_build_index_map() {
        let remove_index = [2, 4, 5, 7];
        let len = 9;
        let index_map = build_index_remap(len, &remove_index);
        assert_eq!(index_map.len(), len);
        for i in &remove_index {
            assert!(index_map[*i].is_none());
        }

        let expect_index_map = vec![
            Some(0),
            Some(1),
            None,
            Some(2),
            None,
            None,
            Some(3),
            None,
            Some(4),
        ];
        assert_eq!(expect_index_map, index_map);
    }

    #[test]
    fn test_build_incomplete_graph() {
        let mut builder = GraphBuilder::new();
        for i in 0..3 {
            builder.add_node(i, NodeKind::Final);
        }

        let arcs = [(0, 1), (1, 0), (1, 2), (2, 1), (2, 3), (3, 5)];
        for (s, d) in &arcs {
            builder.add_arc(*s, *d, ());
        }

        let graph = builder.build_graph();
        assert_eq!(
            graph.nodes,
            vec![NodeKind::Final, NodeKind::Final, NodeKind::Final]
        );
        assert_eq!(graph.adjacent, vec![vec![1], vec![0, 2], vec![1]]);
    }

    fn build_test_graph() -> Graph<()> {
        let node_type = [
            true, false, false, false, true, false, false, false, false, false, false, false,
            false, false, false, false,
        ];
        let mut builder = GraphBuilder::new();
        for (i, nt) in enumerate! {node_type} {
            if *nt {
                builder.add_final_node(i);
            } else {
                builder.add_simple_node(i);
            }
        }

        let arcs = [
            (0, 1),
            (0, 2),
            (0, 3),
            (3, 6),
            (6, 5),
            (5, 4),
            (2, 4),
            (6, 7),
            (0, 8),
            (8, 9),
            (9, 11),
            (9, 10),
            (11, 0),
            (0, 12),
            (12, 13),
            (13, 14),
            (14, 15),
            (13, 15),
            (15, 12),
        ];

        for (s, d) in &arcs {
            builder.add_arc(*s, *d, ());
        }

        builder.build_graph()
    }

    impl<T> PartialEq<usize> for Arc<T> {
        fn eq(&self, n: &usize) -> bool {
            self.next == *n
        }
    }

    impl<T> PartialEq<Arc<T>> for Arc<T> {
        fn eq(&self, n: &Arc<T>) -> bool {
            self.next == n.next
        }
    }

    #[test]
    fn test_fake_nodes() {
        let node_type = [true, false, false, true, false, true];
        let mut builder = GraphBuilder::new();
        for (i, nt) in enumerate! {node_type} {
            if *nt {
                builder.add_final_node(i);
            } else {
                builder.add_simple_node(i);
            }
        }

        let arcs = [(0, 1), (0, 4), (1, 2), (2, 1), (2, 3), (4, 3), (4, 5)];

        for (s, d) in &arcs {
            builder.add_arc(*s, *d, ());
        }

        let g = builder.build_graph();
        let nodes = g.adjacent.len();
        let g = g.add_fake_nodes();

        assert_eq!(g.adjacent.len(), g.nodes.len());
        assert_eq!(g.adjacent.len(), nodes + 2);

        let node_type = [false, false, false, false, false, false, false, true];
        let mut builder = GraphBuilder::new();
        for (i, nt) in enumerate! {node_type} {
            if *nt {
                builder.add_final_node(i);
            } else {
                builder.add_simple_node(i);
            }
        }

        let arcs = [
            (0, 1),
            (1, 2),
            (2, 3),
            (3, 2),
            (3, 4),
            (1, 5),
            (5, 4),
            (5, 6),
            (1, 7),
            (6, 7),
            (4, 7),
        ];

        for (s, d) in &arcs {
            builder.add_arc(*s, *d, ());
        }

        let expected = builder.build_graph();

        assert_eq!(expected.adjacent, g.adjacent);
        assert_eq!(expected.nodes, g.nodes);
    }
}
