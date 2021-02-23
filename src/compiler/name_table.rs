use super::Location;
use std::collections::HashMap;


use macro_derive::IntoNameError;
use crate::new_name_error;

type Loc = (usize, usize);

#[derive(Debug)]
pub struct GlobalNameTable<'a> {
    names: HashMap<&'a str, GlobalName<'a>>,
    status: CollectionStatus<'a>,
}

impl<'a> GlobalNameTable<'a> {
    fn new() -> Self {
        Self {
            names: HashMap::new(),
            status: CollectionStatus::Global,
        }
    }

    pub fn add_network(self, name: &'a str, loc: Loc) -> GlobalNameResult<'a> {
        if let Some(prev) = self.names.get(name) {
            let err = GlobalNameError::new(name, GlobalClassName::Network, prev.loc, loc);
            Err(err)?
        } else {
            Ok(self.insert_new_network(name, loc))
        }
    }

    pub fn add_automata(self, name: &'a str, loc: Loc) -> GlobalNameResult<'a> {
        if let CollectionStatus::Network(net_name) = self.status {
            self.insert_new_automata(name, net_name, loc)
        } else {
            panic!("Call add_automata in state: {:?}", self.status)
        }
    }

    pub fn add_link(self, name: &'a str, loc: Loc) -> GlobalNameResult<'a> {
        self.insert_network_name(name, NetworkName::Link, loc)
    }

    pub fn add_rel_label(self, name: &'a str, loc: Loc) -> GlobalNameResult<'a> {
        self.insert_network_name(name, NetworkName::RelLabel, loc)
    }

    pub fn add_obs_label(self, name: &'a str, loc: Loc) -> GlobalNameResult<'a> {
        self.insert_network_name(name, NetworkName::ObsLabel, loc)
    }

    pub fn add_event(self, name: &'a str, loc: Loc) -> GlobalNameResult<'a> {
        self.insert_network_name(name, NetworkName::Event, loc)
    }

    pub fn add_begin(self, name: &'a str, loc: Loc) -> GlobalNameResult<'a> {
        self.insert_automata_name(name, loc, AutomataName::Begin)
    }

    pub fn add_state(self, name: &'a str, loc: Loc) -> GlobalNameResult<'a> {
        self.insert_automata_name(name, loc, AutomataName::State)
    }

    pub fn add_transition(self, name: &'a str, loc: Loc) -> GlobalNameResult<'a> {
        self.insert_automata_name(name, loc, AutomataName::Transition)
    }

    pub fn exit_automata(mut self) -> Self {
        if let CollectionStatus::Automata { net, automata: _ } = self.status {
            self.status = CollectionStatus::Network(net);
            self
        } else {
            panic!()
        }
    }

    pub fn exit_network(mut self) -> Self {
        self.status = CollectionStatus::Global;
        self
    }

    pub fn validate(self) -> GlobalNameResult<'a> {
        for (_, table) in self.names.iter() {
            for (_, item) in table.names.iter() {
                match item {
                    NetworkName::Automata(automata) => automata.validate()?,
                    _ => {}
                }
            }
        }

        Ok(self)
    }

    fn insert_automata_name(
        mut self,
        name: &'a str,
        loc: Loc,
        automata_cls: AutomataName,
    ) -> GlobalNameResult<'a> {
        if let CollectionStatus::Automata { net, automata } = self.status {
            self.check_name(name, loc, &automata_cls)?;
            let net_table = self.names.get_mut(net).unwrap();
            let automata_table = net_table.names.get_mut(automata).unwrap();
            if let NetworkName::Automata(ref mut automata_table) = automata_table {
                automata_table.insert_name(name, loc, automata_cls);
                Ok(self)
            } else {
                panic!("`{}` should point to an automata", automata);
            }
        } else {
            panic!("call insert_automata_name outside automata")
        }
    }

    fn insert_new_network(mut self, name: &'a str, loc: Loc) -> Self {
        self.status = CollectionStatus::Network(name);
        self.names.insert(name, GlobalName::new(loc));
        self
    }

    fn insert_new_automata(
        mut self,
        automata_name: &'a str,
        net_name: &'a str,
        loc: Loc,
    ) -> GlobalNameResult<'a> {
        self.check_name(automata_name, loc, NameClass::Automata)?;
        let net_table = self.names.get_mut(net_name).unwrap();
        let automata_info = NetworkName::Automata(Automata::new());
        net_table.names.insert(automata_name, automata_info);
        self.status = CollectionStatus::Automata {
            net: net_name,
            automata: automata_name,
        };
        Ok(self)
    }

    fn insert_network_name(
        mut self,
        name: &'a str,
        class: NetworkName<'a>,
        loc: Loc,
    ) -> GlobalNameResult<'a> {
        self.check_name(name, loc, &class)?;
        if let CollectionStatus::Network(net_name) = self.status {
            let net_table = self.names.get_mut(net_name).unwrap();
            net_table.names.insert(name, class);
            Ok(self)
        } else {
            panic!("Call add_automata in state: {:?}", self.status)
        }
    }

    fn check_name<T: Into<NameClass>>(
        &self,
        name: &'a str,
        loc: Loc,
        curr_class: T,
    ) -> Result<(), NameError<'a>> {
        let curr_class = curr_class.into();
        match self.status {
            CollectionStatus::Global => self.check_global_name(name, loc),
            CollectionStatus::Network(net) => self.check_network_name(name, net, loc, curr_class),
            CollectionStatus::Automata { net, automata } => {
                self.check_automata_name(name, net, automata, loc, curr_class)
            }
        }
    }

    fn check_global_name(&self, name: &'a str, loc: Loc) -> Result<(), NameError<'a>> {
        if let Some(prev) = self.names.get(name) {
            let err = GlobalNameError::new(name, GlobalClassName::Network, prev.loc, loc);
            Err(NameError::GlobalNameError(err))
        } else {
            Ok(())
        }
    }

    fn check_network_name(
        &self,
        name: &'a str,
        net_name: &'a str,
        loc: Loc,
        curr_class: NameClass,
    ) -> Result<(), NameError<'a>> {
        if net_name == name {
            new_name_error! {name, NameClass::Network, curr_class, (0, 0), loc}
        } else {
            let net_table = self.names.get(net_name).unwrap();
            if let Some(prev) = net_table.names.get(name) {
                let name_cls = NameClass::from_network_name(&prev);
                new_name_error! {name, name_cls, curr_class, (0, 0), loc}
            } else {
                Ok(())
            }
        }
    }

    fn check_automata_name(
        &self,
        name: &'a str,
        net_name: &'a str,
        automata_name: &'a str,
        loc: Loc,
        curr_class: NameClass,
    ) -> Result<(), NameError<'a>> {
        self.check_network_name(name, net_name, loc, curr_class)?;
        let net_table = self.names.get(net_name).unwrap();
        let automata_table = net_table.names.get(automata_name).unwrap();
        if let NetworkName::Automata(automata) = automata_table {
            if let Some(prev) = automata.names.get(name) {
                let cls = NameClass::from_automata_name(&prev.class);
                new_name_error! {name, cls, curr_class, prev.loc, loc}
            } else {
                Ok(())
            }
        } else {
            panic!("`{}` should index an automata table", automata_name);
        }
    }
}

pub type GlobalNameResult<'a> = Result<GlobalNameTable<'a>, NameError<'a>>;

#[derive(Debug)]
enum CollectionStatus<'a> {
    Network(&'a str),
    Automata { net: &'a str, automata: &'a str },
    Global,
}

#[derive(Debug)]
struct GlobalName<'a> {
    loc: Loc,
    names: HashMap<&'a str, NetworkName<'a>>,
}

impl<'a> GlobalName<'a> {
    fn new(loc: Loc) -> Self {
        Self {
            loc,
            names: HashMap::new(),
        }
    }
}

#[derive(Debug)]
enum NetworkName<'a> {
    Link,
    Event,
    ObsLabel,
    RelLabel,
    Automata(Automata<'a>),
}

#[derive(Debug)]
struct Automata<'a> {
    names: HashMap<&'a str, AutomataInfo>,
}

impl<'a> Automata<'a> {
    fn new() -> Self {
        Self {
            names: HashMap::new(),
        }
    }

    fn insert_name(&mut self, name: &'a str, loc: Loc, class: AutomataName) {
        let info = AutomataInfo { loc, class };
        self.names.insert(name, info);
    }

    fn validate(&self) -> Result<(), BeginStateError<'a>> {
        let begin_states: Vec<&'a str> = self
            .names
            .iter()
            .filter_map(|(name, info)| match info.class {
                AutomataName::Begin => Some(*name),
                _ => None,
            })
            .collect();
        match begin_states.len() {
            0 => Err(BeginStateError::NoBeginState),
            1 => Ok(()),
            _ => Err(BeginStateError::MultipleBeginState(begin_states)),
        }
    }
}

#[derive(Debug)]
enum AutomataName {
    State,
    Begin,
    Transition,
}

#[derive(Debug)]
struct AutomataInfo {
    loc: Loc,
    class: AutomataName,
}

enum NameStatus {
    Defined,
    Undefined,
}

#[derive(Debug)]
pub enum NameError<'a> {
    GlobalNameError(GlobalNameError<'a>),
    UndefinedNetwork(UndefinedNetwork<'a>),
    NameRidefinitionError(NameRidefinitionError<'a>),
    BeginStateError(BeginStateError<'a>),
}

#[derive(Debug, PartialEq)]
pub enum GlobalClassName {
    Network,
    Request,
}

#[derive(Debug, IntoNameError)]
pub struct GlobalNameError<'a> {
    pub name: &'a str,
    pub class: GlobalClassName,
    pub orig_loc: Location,
    pub new_loc: Location,
}

impl<'a> GlobalNameError<'a> {
    fn new(name: &'a str, class: GlobalClassName, orig_loc: Loc, new_loc: Loc) -> Self {
        Self {
            name,
            class,
            orig_loc: orig_loc.into(),
            new_loc: new_loc.into(),
        }
    }
}

#[derive(Debug, IntoNameError)]
pub struct UndefinedNetwork<'a> {
    pub names: Vec<(&'a str, Loc)>,
}

#[derive(Debug)]
enum NetworkDefinitionState {
    RequestDefined,
    NetworkDefined,
    FullDefined,
}

#[derive(Debug, IntoNameError)]
pub struct NameRidefinitionError<'a> {
    name: &'a str,
    orig_loc: Loc,
    ridef_loc: Loc,
    orig_class: NameClass,
    ridef_class: NameClass,
}

#[derive(Debug, IntoNameError)]
pub enum BeginStateError<'a> {
    NoBeginState,
    MultipleBeginState(Vec<&'a str>),
}



#[derive(Debug, PartialEq, Copy, Clone)]
pub enum NameClass {
    Network,
    Automata,
    Link,
    Event,
    ObsLabel,
    RelLabel,
    State,
    Transition,
}

impl NameClass {
    fn from_network_name(cls: &NetworkName) -> Self {
        match cls {
            NetworkName::Automata(_) => Self::Automata,
            NetworkName::Event => Self::Event,
            NetworkName::Link => Self::Link,
            NetworkName::ObsLabel => Self::ObsLabel,
            NetworkName::RelLabel => Self::RelLabel,
        }
    }

    fn from_automata_name(cls: &AutomataName) -> Self {
        match cls {
            AutomataName::Begin | AutomataName::State => Self::State,
            AutomataName::Transition => Self::Transition,
        }
    }
}

impl<'a> From<&NetworkName<'a>> for NameClass {
    fn from(cls: &NetworkName) -> Self {
        match cls {
            NetworkName::Automata(_) => Self::Automata,
            NetworkName::Event => Self::Event,
            NetworkName::Link => Self::Link,
            NetworkName::ObsLabel => Self::ObsLabel,
            NetworkName::RelLabel => Self::RelLabel,
        }
    }
}

impl From<&AutomataName> for NameClass {
    fn from(cls: &AutomataName) -> Self {
        match cls {
            AutomataName::Begin | AutomataName::State => Self::State,
            AutomataName::Transition => Self::Transition,
        }
    }
}

#[macro_export]
macro_rules! new_name_error {
    ($name:expr, $orig_cls:expr, $ridef_cls:expr, $orig_loc:expr, $ridef_loc:expr) => {{
        let err = NameRidefinitionError {
            name: $name,
            orig_loc: $orig_loc,
            ridef_loc: $ridef_loc,
            orig_class: $orig_cls,
            ridef_class: $ridef_cls,
        };
        let name_error = NameError::NameRidefinitionError(err);
        Err(name_error)
    }};
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn test_correct_name() {
        let name_table = GlobalNameTable::new();
        let name_table = name_table.add_network("netname", (0, 1)).unwrap();
        let name_table = name_table.add_event("ev", (4, 5)).unwrap();
        let name_table = name_table.add_link("lk", (6, 7)).unwrap();
        let name_table = name_table.add_automata("at", (10, 12)).unwrap();
        let name_table = name_table.add_begin("s1", (13, 15)).unwrap();
        let name_table = name_table.add_state("s2", (45, 35)).unwrap();
        let name_table = name_table.exit_automata();
        let _ = name_table.exit_network();
    }

    #[test]
    fn test_ridefined_name() {
        let name_table = GlobalNameTable::new();
        let name_table = name_table.add_network("netname", (0, 1)).unwrap();
        let name_table = name_table.add_event("ev", (4, 5)).unwrap();
        let name_table = name_table.add_link("lk", (6, 7)).unwrap();
        let name_table = name_table.add_automata("at", (10, 12)).unwrap();
        let name_table = name_table.exit_automata();
        let err = name_table
            .add_link("at", (15, 18))
            .expect_err("`at` is defined twice");
        match err {
            NameError::NameRidefinitionError(err) => {
                assert_eq!(err.name, "at");
                assert_eq!(err.orig_class, NameClass::Automata);
                assert_eq!(err.ridef_class, NameClass::Link);
            }
            err => panic!("This should be a NameRidefinitionError: {:?} found", err),
        }
    }

    #[test]
    fn test_multiple_begin_state() {
        let name_table = GlobalNameTable::new();
        let name_table = name_table.add_network("netname", (0, 1)).unwrap();
        let name_table = name_table.add_event("ev", (4, 5)).unwrap();
        let name_table = name_table.add_link("lk", (6, 7)).unwrap();
        let name_table = name_table.add_automata("at", (10, 12)).unwrap();
        let name_table = name_table.add_begin("s0", (45, 12)).unwrap();
        let name_table = name_table.add_begin("s1", (56, 142)).unwrap();

        let name_table = name_table.exit_automata();
        let name_table = name_table.exit_network();

        let err = name_table
            .validate()
            .expect_err("There are two begin states");

        match err {
            NameError::BeginStateError(err) => match err {
                BeginStateError::MultipleBeginState(states) => {
                    assert_eq!(states.len(), 2);
                    assert!(states.contains(&"s0"));
                    assert!(states.contains(&"s1"));
                }
                _ => panic!("There are begin state ridefinition"),
            },
            err => panic!("{:?} should be a BeginStateError", err),
        }
    }

    #[test]
    fn test_no_begin_state() {
        let name_table = GlobalNameTable::new();
        let name_table = name_table.add_network("netname", (0, 1)).unwrap();
        let name_table = name_table.add_event("ev", (4, 5)).unwrap();
        let name_table = name_table.add_link("lk", (6, 7)).unwrap();
        let name_table = name_table.add_automata("at", (10, 12)).unwrap();

        let name_table = name_table.exit_automata();
        let name_table = name_table.exit_network();

        let err = name_table
            .validate()
            .expect_err("There aren't begin states");

        match err {
            NameError::BeginStateError(err) => match err {
                BeginStateError::NoBeginState => {}
                _ => panic!("There aren't begin state ridefinition"),
            },
            err => panic!("{:?} should be a BeginStateError", err),
        }
    }


    #[test]
    fn test_state_ridefinition() {
        let name_table = GlobalNameTable::new();
        let name_table = name_table.add_network("netname", (0, 1)).unwrap();
        let name_table = name_table.add_event("ev", (4, 5)).unwrap();
        let name_table = name_table.add_link("lk", (6, 7)).unwrap();
        let name_table = name_table.add_automata("at", (10, 12)).unwrap();
        let name_table = name_table.add_begin("s0", (45, 12)).unwrap();
        let name_table = name_table.add_state("s1", (56, 142)).unwrap();
        

        let err = name_table.add_state("s1", (67, 132)).expect_err("State s1 is defined twice");

        match err {
            NameError::NameRidefinitionError(err) => {
                assert_eq!(err.name, "s1");
                assert_eq!(err.orig_class, NameClass::State);
                assert_eq!(err.ridef_class, NameClass::State);
            },
            err => panic!("expected NameRidefinitionError: found `{:?}`", err)
        }


    }
}
