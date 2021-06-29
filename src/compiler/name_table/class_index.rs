use ahash::AHashMap;

#[derive(Debug)]
pub struct ClassIndex<T> {
    counter: AHashMap<T, usize>,
}

impl<T> ClassIndex<T>
where
    T: Eq + std::hash::Hash,
{
    pub fn new() -> Self {
        Self {
            counter: AHashMap::new(),
        }
    }

    pub fn get_count(&mut self, cls: T) -> usize {
        let curr = if let Some(curr) = self.counter.get(&cls) {
            *curr
        } else {
            0
        };
        self.counter.insert(cls, curr + 1);
        curr
    }
}

#[cfg(test)]
mod test {

    use super::super::name_class::NameClass;
    use super::*;

    #[test]
    fn test_counter() {
        let mut counter = ClassIndex::new();

        let result = [
            (0, NameClass::Transition),
            (0, NameClass::ObsLabel),
            (1, NameClass::Transition),
            (0, NameClass::State),
            (1, NameClass::State),
            (2, NameClass::Transition),
        ];

        for (count, cls) in result.iter() {
            let c = counter.get_count(*cls);
            assert_eq!(c, *count);
        }
    }
}
