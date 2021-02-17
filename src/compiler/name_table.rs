use super::Location;
use std::collections::HashMap;

#[derive(Clone, Copy, Debug, PartialEq)]
enum NameClass {
    Network,
    Automata,
    Link,
    Event,
    ObsLabel,
    RelLabel,
    State,
    Transition,
    Request,
}

struct NameTableInfo {
    loc: (usize, usize),
    class: NameClass,
}

impl NameTableInfo {
    fn new(loc: (usize, usize), class: NameClass) -> Self {
        Self { loc, class }
    }
}

#[derive(Debug)]
pub struct RidefinitionError<'a> {
    name: &'a str,
    orig_loc: Location,
    ridef_loc: Location,
    orig_class: NameClass,
    ridef_class: NameClass,
}

impl<'a> RidefinitionError<'a> {
    fn new(
        name: &'a str,
        orig_loc: Location,
        ridef_loc: Location,
        orig_class: NameClass,
        ridef_class: NameClass,
    ) -> Self {
        Self {
            name,
            orig_class,
            orig_loc,
            ridef_class,
            ridef_loc,
        }
    }
}

pub type InsertResult<'a> = Result<(), RidefinitionError<'a>>;

pub struct NameTable<'a> {
    names: HashMap<&'a str, NameTableInfo>,
}

impl<'a> NameTable<'a> {
    pub fn new() -> Self {
        Self {
            names: HashMap::new(),
        }
    }

    pub fn insert_automata(&mut self, name: &'a str, loc: (usize, usize)) -> InsertResult<'a> {
        self.insert_name(name, NameClass::Automata, loc)
    }

    pub fn insert_network(&mut self, name: &'a str, loc: (usize, usize)) -> InsertResult<'a> {
        self.insert_name(name, NameClass::Network, loc)
    }

    pub fn insert_link(&mut self, name: &'a str, loc: (usize, usize)) -> InsertResult<'a> {
        self.insert_name(name, NameClass::Link, loc)
    }

    pub fn insert_event(&mut self, name: &'a str, loc: (usize, usize)) -> InsertResult<'a> {
        self.insert_name(name, NameClass::Event, loc)
    }

    pub fn insert_obs_label(&mut self, name: &'a str, loc: (usize, usize)) -> InsertResult<'a> {
        self.insert_name(name, NameClass::ObsLabel, loc)
    }

    pub fn insert_rel_label(&mut self, name: &'a str, loc: (usize, usize)) -> InsertResult<'a> {
        self.insert_name(name, NameClass::RelLabel, loc)
    }

    pub fn insert_state(&mut self, name: &'a str, loc: (usize, usize)) -> InsertResult<'a> {
        self.insert_name(name, NameClass::State, loc)
    }

    pub fn insert_transition(&mut self, name: &'a str, loc: (usize, usize)) -> InsertResult<'a> {
        self.insert_name(name, NameClass::Transition, loc)
    }

    pub fn insert_request(&mut self, name: &'a str, loc: (usize, usize)) -> InsertResult<'a> {
        self.insert_name(name, NameClass::Request, loc)
    }

    fn insert_name(
        &mut self,
        name: &'a str,
        class: NameClass,
        loc: (usize, usize),
    ) -> InsertResult<'a> {
        if let Some(prev_def) = self.names.get(name) {
            let err = self.make_error(prev_def, name, class, loc);
            Err(err)
        } else {
            self.insert_key_value(name, loc, class);
            Ok(())
        }
    }

    fn make_error(
        &self,
        prev_def: &NameTableInfo,
        name: &'a str,
        class: NameClass,
        loc: (usize, usize),
    ) -> RidefinitionError<'a> {
        let new_loc = Location::from_tuple(loc);
        let prev_loc = Location::from_tuple(prev_def.loc);

        RidefinitionError::new(name, prev_loc, new_loc, prev_def.class, class)
    }

    fn insert_key_value(&mut self, name: &'a str, loc: (usize, usize), class: NameClass) {
        let value = NameTableInfo::new(loc, class);
        self.names.insert(name, value);
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn test_ridefinition() {
        let mut name_table = NameTable::new();

        name_table.insert_automata("testname", (0, 5)).unwrap();
        name_table.insert_event("hill", (5, 6)).unwrap();
        name_table.insert_link("superlink", (7, 8)).unwrap();
        name_table.insert_network("sun", (10, 12)).unwrap();
        name_table.insert_obs_label("labelA", (45, 142)).unwrap();

        let result = name_table.insert_transition("testname", (340, 543));
        match result {
            Ok(_) => panic!("This call should return an error on 'testname'"),
            Err(err) => {
                assert_eq!(err.name, "testname");
                assert_eq!(err.orig_class, NameClass::Automata);
                assert_eq!(err.ridef_class, NameClass::Transition);
                is_same_location(&err.orig_loc, 0, 5);
                is_same_location(&err.ridef_loc, 340, 543);
            }
        }
    }

    fn is_same_location(loc: &Location, begin: usize, end: usize) {
        assert_eq!(loc.begin, begin);
        assert_eq!(loc.end, end);
    }

    #[test]
    fn test_unique_names() {
        let mut name_table = NameTable::new();

        name_table.insert_automata("testname", (0, 5)).unwrap();
        name_table.insert_event("hill", (5, 6)).unwrap();
        name_table.insert_link("superlink", (7, 8)).unwrap();
        name_table.insert_network("sun", (10, 12)).unwrap();
        name_table.insert_obs_label("labelA", (45, 142)).unwrap();
        name_table.insert_rel_label("labelB", (416, 543)).unwrap();
        name_table.insert_state("status", (600, 601)).unwrap();
        name_table.insert_transition("trans", (786, 809)).unwrap();
    }
}
