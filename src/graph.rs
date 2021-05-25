use crate::utils::zeros;

type AdjList = Vec<Vec<usize>>;

#[derive(Debug)]
pub struct Graph {
    nodes: Vec<NodeKind>,
    adjacent: AdjList,
}

impl Graph {
    pub fn export_graphviz(self) -> String {
        String::new()
    }

    pub fn get_adjacent_list(&self) -> &AdjList {
        &self.adjacent
    }

    pub fn get_node_kind_list(&self) -> &Vec<NodeKind> {
        &self.nodes
    }
}

pub struct GraphBuilder {
    nodes_list: Vec<(usize, usize)>,
    node_kind: Vec<NodeKind>,
}

impl GraphBuilder {
    pub fn new() -> Self {
        Self {
            nodes_list: Vec::new(),
            node_kind: Vec::new(),
        }
    }

    pub fn add_final_node(&mut self) {
        self.add_node(NodeKind::Final);
    }

    pub fn add_simple_node(&mut self) {
        self.add_node(NodeKind::Simple);
    }

    fn add_node(&mut self, kind: NodeKind) {
        self.node_kind.push(kind);
    }

    pub fn add_arc(&mut self, src: usize, dst: usize) {
        self.nodes_list.push((src, dst));
    }

    pub fn build_graph(self) -> Graph {
        let mut adjacent: AdjList = zeros(self.node_kind.len());
        for (s, d) in self.nodes_list.into_iter() {
            adjacent[s].push(d);
        }

        Graph {
            nodes: self.node_kind,
            adjacent,
        }
    }
}

#[derive(Debug)]
pub enum NodeKind {
    Simple,
    Final,
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn test_graph_builder() {
        let mut builder = GraphBuilder::new();

        let nodes = ['a', 'b', 'c', 'd', 'e'];
        for _ in &nodes {
            builder.add_simple_node();
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
}
