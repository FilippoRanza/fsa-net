use std::collections::{HashMap, VecDeque};

#[derive(Default)]
pub struct GraphBuilder<'a> {
    nodes: Vec<Node<'a>>,
    edges: Vec<(&'a str, &'a str)>,
    node_indexs: HashMap<&'a str, usize>,
    begin: &'a str,
}

impl<'a> GraphBuilder<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_begin(mut self, begin: &'a str) -> Self {
        self.begin = begin;
        self
    }

    pub fn add_node(mut self, name: &'a str) -> Self {
        let node = Node::new(name);
        self.nodes.push(node);
        self.node_indexs.insert(name, self.nodes.len() - 1);

        self
    }

    pub fn add_link(mut self, from: &'a str, to: &'a str) -> Self {
        self.edges.push((from, to));
        self
    }

    pub fn build_graph(self) -> Graph<'a> {
        let root = self.node_indexs[self.begin];
        let adjacent_list = self.make_adjacent_list();
        Graph {
            nodes: self.nodes,
            adjacent_list,
            root,
        }
    }

    fn make_adjacent_list(&self) -> Vec<Vec<usize>> {
        let mut output: Vec<Vec<usize>> = (0..self.nodes.len()).map(|_| vec![]).collect();
        for edge in &self.edges {
            let (from, to) = self.get_indexes(edge);
            output[from].push(to);
        }
        output
    }

    fn get_indexes(&self, edge: &(&'a str, &'a str)) -> (usize, usize) {
        let (from, to) = edge;
        let from = self.node_indexs[from];
        let to = self.node_indexs[to];
        (from, to)
    }
}

pub struct Graph<'a> {
    nodes: Vec<Node<'a>>,
    adjacent_list: Vec<Vec<usize>>,
    root: usize,
}

impl<'a> Graph<'a> {
    pub fn breadth_first_search(mut self) -> Result<(), super::GraphError<'a>> {
        let mut queue = VecDeque::new();
        self.nodes[self.root].status = NodeStatus::Discovered;
        let mut missing_nodes = self.nodes.len() - 1; // root is already seen
        queue.push_back(self.root);
        while let Some(head) = queue.pop_front() {
            for adj in &self.adjacent_list[head] {
                let node = &mut self.nodes[*adj];
                if let NodeStatus::NonDiscovered = node.status {
                    node.status = NodeStatus::Discovered;
                    queue.push_back(*adj);
                    missing_nodes -= 1;
                }
            }
        }

        if missing_nodes == 0 {
            Ok(())
        } else {
            let undiscovered = self.collect_undiscovered();
            Err(undiscovered)
        }
    }

    fn collect_undiscovered(&self) -> Vec<&'a str> {
        self.nodes
            .iter()
            .filter(|n| !n.status.is_ok())
            .map(|n| n.name)
            .collect()
    }
}

struct Node<'a> {
    name: &'a str,
    status: NodeStatus,
}

impl<'a> Node<'a> {
    fn new(name: &'a str) -> Self {
        Self {
            name,
            status: NodeStatus::default(),
        }
    }
}

enum NodeStatus {
    Discovered,
    NonDiscovered,
}

impl NodeStatus {
    fn is_ok(&self) -> bool {
        match self {
            Self::Discovered => true,
            Self::NonDiscovered => false,
        }
    }
}

impl Default for NodeStatus {
    fn default() -> Self {
        Self::NonDiscovered
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn test_connected_graph() {
        let graph = GraphBuilder::new()
            .add_link("a", "a")
            .add_node("a")
            .add_link("b", "c")
            .add_node("b")
            .add_link("a", "b")
            .add_link("a", "c")
            .add_node("d")
            .add_link("c", "d")
            .set_begin("a")
            .add_link("d", "a")
            .add_node("c")
            .add_link("d", "b")
            .build_graph();

        assert!(graph.breadth_first_search().is_ok())
    }

    #[test]
    fn test_unconnected_graph() {
        let graph = GraphBuilder::new()
            .add_node("a")
            .add_node("b")
            .add_node("c")
            .add_node("d")
            .add_node("e")
            .add_link("a", "b")
            .add_link("a", "c")
            .add_link("a", "a")
            .add_link("c", "d")
            .add_link("b", "c")
            .add_link("d", "a")
            .add_link("d", "b")
            .set_begin("a")
            .build_graph();

        let res = graph.breadth_first_search();
        assert_eq!(res, Result::Err(vec!["e"]));
    }
}
