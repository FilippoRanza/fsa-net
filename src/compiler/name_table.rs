use super::Location;
use std::collections::HashMap;

enum NameClass {
    Network,
    Automata,
    Link,
    Event,
    ObsLabel,
    RelLabel,
    State,
    Transition,
}

struct NameTableInfo {
    loc: Location, 
    class: NameClass
}

impl NameTableInfo {
    fn new(loc: (usize, usize), class: NameClass) -> Self {
        let (begin, end) = loc;
        let loc = Location::new(begin, end);
        Self {
            loc, 
            class
        }
    }
}

struct RidefinitionError<'a> {
    name: &'a str,
    orig_loc: Location,
    ridef_loc: Location,
    orig_class: NameClass,
    ridef_class: NameClass
}

impl<'a> RidefinitionError<'a> {
    fn new(name: &'a str, orig_loc: Location, ridef_loc: Location, orig_class: NameClass, ridef_class: NameClass) -> Self {
        Self {
            name, orig_class, orig_loc, ridef_class, ridef_loc
        }
    }
}


struct NameTable<'a> {
    names: HashMap<&'a str, NameClass>
}

impl<'a> NameTable<'a> {
    
}


