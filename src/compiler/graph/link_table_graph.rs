use std::collections::{HashMap, VecDeque};

#[derive(Default)]
pub struct GraphBuilder<'a> {
    nodes: Vec<Node<'a>>,
    adjacent_list: Vec<Vec<usize>>,
    node_indexs: HashMap<&'a str, usize>,
    begin: &'a str,
}

impl<'a> GraphBuilder<'a> {
    fn new() -> Self {
        Self::default()
    }

    fn set_begin(mut self, begin: &'a str) -> Self {
        self.begin = begin;
        self
    }

    fn add_node(mut self, name: &'a str) -> Self {
        let node = Node::new(name);
        self.nodes.push(node);
        self.adjacent_list.push(vec![]);
        self.node_indexs.insert(name, self.nodes.len() - 1);

        self
    }

    fn add_link(mut self, from: &'a str, to: &'a str) -> Self {
        let src_node = self.node_indexs[from];
        let dst_node = self.node_indexs[to];
        self.adjacent_list[src_node].push(dst_node);
        self
    }

    fn build_graph(self) -> Graph<'a> {
        let root = self.node_indexs[self.begin];
        Graph {
            nodes: self.nodes,
            adjacent_list: self.adjacent_list,
            root,
        }
    }
}

pub struct Graph<'a> {
    nodes: Vec<Node<'a>>,
    adjacent_list: Vec<Vec<usize>>,
    root: usize,
}

impl<'a> Graph<'a> {
    fn breadth_first_search(mut self) -> bool {
        let mut queue = VecDeque::new();
        self.nodes.get_mut(self.root).unwrap().status = NodeStatus::Discovered;
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

        missing_nodes == 0
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
            .add_node("a")
            .add_node("b")
            .add_node("c")
            .add_node("d")
            .add_link("a", "b")
            .add_link("a", "c")
            .add_link("a", "a")
            .add_link("c", "d")
            .add_link("b", "c")
            .add_link("d", "a")
            .add_link("d", "b")
            .set_begin("a")
            .build_graph();

        assert!(graph.breadth_first_search())
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

        assert!(!graph.breadth_first_search())
    }
}
