use crate::utils::{auto_sort, zeros};

type AdjList = Vec<Vec<usize>>;

#[derive(Debug)]
pub struct Graph {
    nodes: Vec<NodeKind>,
    adjacent: AdjList,
}

impl<'a> Graph {
    pub fn get_adjacent_list(&'a self) -> &'a AdjList {
        &self.adjacent
    }

    pub fn get_node_kind_list(&self) -> &Vec<NodeKind> {
        &self.nodes
    }

    pub fn prune<T>(self, states: Vec<T>) -> (Self, Vec<T>) {
        let prune = prune_list(&self.adjacent, &self.nodes);
        let nodes = filter_by_index(self.nodes, &prune);
        let adjacent = remap_indexes(self.adjacent, &prune);
        let adjacent = filter_by_index(adjacent, &prune);
        let states = filter_by_index(states, &prune);
        (Self { adjacent, nodes }, states)
    }
}

fn remap_indexes(adj: AdjList, prune: &[usize]) -> AdjList {
    let index_map = build_index_remap(adj.len(), prune);
    adj.into_iter()
        .map(|v| remap_adjacent(v, &index_map))
        .collect()
}

fn remap_adjacent(iter: Vec<usize>, remap: &[Option<usize>]) -> Vec<usize> {
    iter.into_iter().filter_map(|i| remap[i]).collect()
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

pub struct GraphBuilder {
    nodes_list: Vec<(usize, usize)>,
    node_kind: Vec<(NodeKind, usize)>,
}

impl GraphBuilder {
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

    pub fn add_arc(&mut self, src: usize, dst: usize) {
        self.nodes_list.push((src, dst));
    }

    pub fn build_graph(self) -> Graph {
        let node_count = self.node_kind.len();
        let mut adjacent: AdjList = zeros(node_count);
        for (s, d) in self.nodes_list.into_iter() {
            if s < node_count && d < node_count {
                adjacent[s].push(d);
            }
        }

        let nodes = auto_sort(&mut self.node_kind.into_iter());
        Graph { nodes, adjacent }
    }
}

#[derive(Debug, PartialEq)]
pub enum NodeKind {
    Simple,
    Final,
}

fn prune_list(adj: &AdjList, kind_list: &[NodeKind]) -> Vec<usize> {
    let mut reach: Vec<bool> = kind_list
        .iter()
        .map(|k| match k {
            NodeKind::Final => true,
            NodeKind::Simple => false,
        })
        .collect();
    let mut seen = reach.clone();

    for node in 0..adj.len() {
        if !seen[node] {
            make_prune_list(node, adj, &mut seen, &mut reach);
        }
    }

    reach
        .into_iter()
        .enumerate()
        .filter_map(|(i, s)| if s { None } else { Some(i) })
        .collect()
}

fn make_prune_list(
    curr: usize,
    adj: &AdjList,
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
            .find(|curr| make_prune_list(**curr, adj, seen, reach));
        let stat = next.is_some();
        reach[curr] = stat;
        stat
    }
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
            builder.add_arc(s, d);
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
        let mut builder =GraphBuilder::new();
        for i in 0..3 {
            builder.add_node(i, NodeKind::Final);
        }

        let arcs = [
            (0, 1),
            (1, 0),
            (1, 2),
            (2, 1),
            (2, 3),
            (3, 5)
        ];
        for (s, d) in &arcs {
            builder.add_arc(*s, *d);
        }

        let graph = builder.build_graph();
        assert_eq!(graph.nodes, vec![NodeKind::Final,NodeKind::Final, NodeKind::Final]);
        assert_eq!(graph.adjacent, vec![
            vec![1],
            vec![0, 2],
            vec![1]
        ]);

    }

    fn build_test_graph() -> Graph {
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
            builder.add_arc(*s, *d);
        }

        builder.build_graph()
    }
}
