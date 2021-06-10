use indexmap::IndexMap;

#[derive(Debug)]
pub struct StateTable<T> {
    table: IndexMap<T, usize>,
}

impl<T> StateTable<T>
where
    T: Eq + std::hash::Hash,
{
    pub fn new() -> Self {
        Self {
            table: IndexMap::default(),
        }
    }

    pub fn insert_state(&mut self, s: T) -> usize {
        if let Some(i) = self.table.get(&s) {
            *i
        } else {
            let index = self.table.len();
            self.table.insert(s, index);
            index
        }
    }

    pub fn get_index(&self, s: &T) -> Option<usize> {
        if let Some(i) = self.table.get(s) {
            Some(*i)
        } else {
            None
        }
    }

    pub fn is_present(&self, s: &T) -> bool {
        self.table.get(s).is_some()
    }

    pub fn get_object(&self, index: usize) -> &T {
        if let Some((k, _)) = self.table.get_index(index) {
            k
        } else {
            panic!("Call get_object on non existing index {}", index)
        }
    }

    pub fn to_state_list(self) -> Vec<T> {
        self.table.into_iter().map(|(s, _)| s).collect()
    }
}
