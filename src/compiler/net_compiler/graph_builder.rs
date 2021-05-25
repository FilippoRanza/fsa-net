use crate::network;
use crate::utils::zeros;

pub struct GraphBuilder {
    node_count: usize,
    arc_list: Vec<(usize, usize, network::Transition)>,
}

impl GraphBuilder {
    pub fn new() -> Self {
        Self {
            node_count: 0,
            arc_list: Vec::new(),
        }
    }

    pub fn add_arc(&mut self, src: usize, dst: usize, trans: network::Transition) {
        self.node_count = self.get_node_count(src, dst);
        let tmp = (src, dst, trans);
        self.arc_list.push(tmp);
    }

    pub fn build_graph(self) -> Vec<Vec<network::Adjacent>> {
        let mut output: Vec<Vec<network::Adjacent>> = zeros(self.node_count);

        for (src, dst, trans) in self.arc_list.into_iter() {
            let adj = network::Adjacent::new(dst, trans);
            output[src].push(adj);
        }

        output
    }

    fn get_node_count(&self, src: usize, dst: usize) -> usize {
        let max = if src > dst { src } else { dst };

        if max >= self.node_count {
            max + 1
        } else {
            self.node_count
        }
    }
}
