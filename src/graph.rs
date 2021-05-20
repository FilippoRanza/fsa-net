use std::collections::HashMap;

#[derive(Debug)]
pub struct Graph<T> {
    nodes: Vec<T>,
    adjacent: Vec<Vec<usize>>,
}

impl<T> Graph<T> {
    fn export_graphviz(self) -> String {
        String::new()
    }
}

pub struct GraphBuilder<T> {
    nodes_index: HashMap<T, usize>,
    nodes_list: Vec<Vec<usize>>,
}

impl<T> GraphBuilder<T>
where
    T: Eq + std::hash::Hash,
{
    pub fn new() -> Self {
        Self {
            nodes_index: HashMap::new(),
            nodes_list: Vec::new(),
        }
    }

    pub fn add_node(&mut self, node: T) {
        if let None = self.nodes_index.get(&node) {
            let index = self.nodes_index.len();
            self.nodes_index.insert(node, index);
            self.nodes_list.push(Vec::new());
        }
    }

    pub fn add_arc(&mut self, src: &T, dst: &T) {
        let src = self.get_index(src);
        let dst = self.get_index(dst);
        self.nodes_list[src].push(dst);
    }

    pub fn get_index(&mut self, n: &T) -> usize {
        if let Some(i) = self.nodes_index.get(n) {
            *i
        } else {
            panic!("Call get_index on non existing node")
        }
    }

    pub fn build_graph(self) -> Graph<T> {
        let mut node_list: Vec<(T, usize)> = self.nodes_index.into_iter().collect();
        node_list.sort_by_key(|(_, i)| *i);
        let node_list = node_list.into_iter().map(|(n, _)| n).collect();
        Graph {
            nodes: node_list,
            adjacent: self.nodes_list,
        }
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn test_graph_builder() {
        let mut builder = GraphBuilder::new();

        let nodes = ['a', 'b', 'c', 'd', 'e'];
        for node in &nodes {
            builder.add_node(*node);
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
            builder.add_arc(s, d);
        }

        let graph = builder.build_graph();

        for (i, n) in nodes.iter().enumerate() {
            assert_eq!(graph.nodes[i], *n);
        }
        assert_eq!(graph.nodes.len(), nodes.len());
        assert_eq!(graph.adjacent.len(), nodes.len());

        assert_eq!(graph.adjacent[0], vec![2, 3, 1]);
        assert_eq!(graph.adjacent[1], vec![]);
        assert_eq!(graph.adjacent[2], vec![1, 3]);
        assert_eq!(graph.adjacent[3], vec![1, 4]);
        assert_eq!(graph.adjacent[4], vec![]);
    }
}
