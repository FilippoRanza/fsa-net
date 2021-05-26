use crate::command;
use crate::network;

use super::super::CompileResult;

use std::collections::HashMap;

pub struct ResultBuilder<'a> {
    results: HashMap<&'a str, CompileStorage>,
}

impl<'a> ResultBuilder<'a> {
    pub fn new() -> Self {
        Self {
            results: HashMap::new(),
        }
    }

    pub fn build_result(self) -> Vec<CompileResult> {
        let mut store: Vec<CompileStorage> = self.results.into_iter().map(|(_, v)| v).collect();
        store.sort_by_key(|s| s.index);
        store
            .into_iter()
            .filter_map(|s| s.get_compile_result())
            .collect()
    }

    pub fn insert_node<T>(mut self, name: &'a str, item: T) -> Self
    where
        T: Into<ItemType>,
    {
        let store = if let Some(out) = self.results.get_mut(name) {
            out
        } else {
            let index = self.results.len();
            let tmp = CompileStorage::new(index);
            self.results.insert(name, tmp);
            self.results.get_mut(name).unwrap()
        };

        match item.into() {
            ItemType::Requests(cmd) => store.req = Some(cmd),
            ItemType::Network(net) => store.net = Some(net),
        }
        self
    }
}

pub enum ItemType {
    Network(network::Network),
    Requests(command::Requests),
}

impl Into<ItemType> for network::Network {
    fn into(self) -> ItemType {
        ItemType::Network(self)
    }
}

impl Into<ItemType> for command::Requests {
    fn into(self) -> ItemType {
        ItemType::Requests(self)
    }
}

#[derive(Default)]
struct CompileStorage {
    net: Option<network::Network>,
    req: Option<command::Requests>,
    index: usize,
}

impl CompileStorage {
    fn new(index: usize) -> Self {
        CompileStorage {
            index,
            ..Default::default()
        }
    }

    fn get_compile_result(self) -> Option<CompileResult> {
        if let Some(req) = self.req {
            let output = CompileResult {
                net: self.net.unwrap(),
                req,
            };
            Some(output)
        } else {
            None
        }
    }
}
