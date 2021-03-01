use std::collections::HashMap;

use crate::{into_name_error, new_name_error};

use super::request_table::{Request, RequestTable};
use super::Loc;

#[derive(Debug)]
pub struct GlobalNameTable<'a> {
    networks: HashMap<&'a str, NetworkNameTable<'a>>,
    requests: HashMap<&'a str, RequestTable<'a>>,
    status: CollectionStatus<'a>,
}

impl<'a> GlobalNameTable<'a> {
    pub fn new() -> Self {
        Self {
            networks: HashMap::new(),
            requests: HashMap::new(),
            status: CollectionStatus::Global,
        }
    }

    pub fn declare_network(self, name: &'a str, loc: Loc) -> GlobalNameResult<'a> {
        self.check_name(name, loc, NameClass::Network, &NameStatus::Defined)?;
        Ok(self.insert_new_network(name, loc, NameStatus::Defined))
    }

    pub fn declare_automata(self, name: &'a str, loc: Loc) -> GlobalNameResult<'a> {
        if let CollectionStatus::Network(net_name) = self.status {
            self.insert_new_automata(name, net_name, loc, NameStatus::Defined)
        } else {
            panic!("Call add_automata in state: {:?}", self.status)
        }
    }

    pub fn declare_link(self, name: &'a str, loc: Loc) -> GlobalNameResult<'a> {
        self.insert_network_name(name, NetworkName::Link, loc, NameStatus::Defined)
    }

    pub fn declare_rel_label(self, name: &'a str, loc: Loc) -> GlobalNameResult<'a> {
        self.insert_network_name(name, NetworkName::RelLabel, loc, NameStatus::Defined)
    }

    pub fn declare_obs_label(self, name: &'a str, loc: Loc) -> GlobalNameResult<'a> {
        self.insert_network_name(name, NetworkName::ObsLabel, loc, NameStatus::Defined)
    }

    pub fn declare_event(self, name: &'a str, loc: Loc) -> GlobalNameResult<'a> {
        self.insert_network_name(name, NetworkName::Event, loc, NameStatus::Defined)
    }

    pub fn declare_begin(self, name: &'a str, loc: Loc) -> GlobalNameResult<'a> {
        self.insert_automata_name(name, loc, AutomataName::Begin, NameStatus::Defined)
    }

    pub fn declare_state(self, name: &'a str, loc: Loc) -> GlobalNameResult<'a> {
        self.insert_automata_name(name, loc, AutomataName::State, NameStatus::Defined)
    }

    pub fn declare_transition(self, name: &'a str, loc: Loc) -> GlobalNameResult<'a> {
        self.insert_automata_name(name, loc, AutomataName::Transition, NameStatus::Defined)
    }

    pub fn insert_request(mut self, name: &'a str, loc: Loc) -> GlobalNameResult<'a> {
        if let Some(prev) = self.requests.get(name) {
            new_name_error! {name, NameClass::Request, NameClass::Request, prev.get_location(), loc}
        } else {
            self.requests.insert(name, RequestTable::new(loc));
            self.status = CollectionStatus::Request(name);
            Ok(self)
        }
    }

    pub fn add_request(mut self, req: Request<'a>) -> GlobalNameResult<'a> {
        if let CollectionStatus::Request(net_name) = self.status {
            let req_table = self.requests.get_mut(net_name).unwrap();
            req_table.add_request(req);
            Ok(self)
        } else {
            panic!("call `add_request` in wrong status")
        }
    }

    pub fn exit_request(mut self) -> Self {
        self.status = CollectionStatus::Global;
        self
    }

    pub fn add_automata(mut self, name: &'a str, loc: Loc) -> GlobalNameResult<'a> {
        if let CollectionStatus::Network(net_name) = self.status {
            let net_table = self.networks.get_mut(net_name).unwrap();
            if let Some(_) = net_table.automata.get(name) {
                net_table.names.remove(name);
                Ok(self)
            } else {
                self.check_name(name, loc, NameClass::Automata, &NameStatus::Undefined)?;
                self.insert_network_name(
                    name,
                    NetworkName::UnknowAutomata,
                    loc,
                    NameStatus::Undefined,
                )
            }
        } else {
            panic!("cannot call `add_automata` outside netword")
        }
    }

    pub fn add_link(self, name: &'a str, loc: Loc) -> GlobalNameResult<'a> {
        self.insert_network_name(name, NetworkName::Link, loc, NameStatus::Undefined)
    }

    pub fn add_rel_label(self, name: &'a str, loc: Loc) -> GlobalNameResult<'a> {
        self.insert_network_name(name, NetworkName::RelLabel, loc, NameStatus::Undefined)
    }

    pub fn add_obs_label(self, name: &'a str, loc: Loc) -> GlobalNameResult<'a> {
        self.insert_network_name(name, NetworkName::ObsLabel, loc, NameStatus::Undefined)
    }

    pub fn add_event(self, name: &'a str, loc: Loc) -> GlobalNameResult<'a> {
        self.insert_network_name(name, NetworkName::Event, loc, NameStatus::Undefined)
    }

    pub fn add_state(self, name: &'a str, loc: Loc) -> GlobalNameResult<'a> {
        self.insert_automata_name(name, loc, AutomataName::State, NameStatus::Undefined)
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
        self.validate_network()?.validate_requests()
    }

    fn validate_network(self) -> GlobalNameResult<'a> {
        for (net_name, table) in self.networks.iter() {
            table.stat.validate(net_name, table.loc)?;
            for (name, item) in table.names.iter() {
                item.stat.validate(name, item.loc)?;
            }
            for automata in table.automata.values() {
                automata.validate()?;
            }
        }

        Ok(self)
    }

    fn validate_requests(self) -> GlobalNameResult<'a> {
        let names: Vec<(&'a str, Loc)> = self
            .requests
            .iter()
            .filter(|(k, _)| !self.networks.contains_key(*k))
            .map(|(k, v)| (*k, v.get_location()))
            .collect();
        if names.len() > 0 {
            let err = UndefinedNetwork { names };
            Err(NameError::UndefinedNetwork(err))
        } else {
            self.validate_request_labels()
        }
    }

    fn validate_request_labels(self) -> GlobalNameResult<'a> {
        for (net_name, req) in self.requests.iter() {
            let net_table = self.networks.get(net_name).unwrap();
            validate_labels(net_table, req.get_linspace_labels(), NameClass::ObsLabel)?;
            validate_labels(net_table, req.get_diagnosis_labels(), NameClass::RelLabel)?;
        }
        Ok(self)
    }

    fn insert_automata_name(
        mut self,
        name: &'a str,
        loc: Loc,
        automata_cls: AutomataName,
        stat: NameStatus,
    ) -> GlobalNameResult<'a> {
        if let CollectionStatus::Automata { net, automata } = self.status {
            let res = self.check_name(name, loc, &automata_cls, &stat)?;
            let net_table = self.networks.get_mut(net).unwrap();
            let automata_table = net_table.automata.get_mut(automata).unwrap();
            let (stat, class) = next_stat_and_class(res, stat, automata_cls);

            automata_table.insert_name(name, loc, class, stat);
            Ok(self)
        } else {
            panic!("call insert_automata_name outside automata")
        }
    }

    fn insert_new_network(mut self, name: &'a str, loc: Loc, stat: NameStatus) -> Self {
        self.status = CollectionStatus::Network(name);
        self.networks.insert(name, NetworkNameTable::new(loc, stat));
        self
    }

    fn insert_new_automata(
        mut self,
        automata_name: &'a str,
        net_name: &'a str,
        loc: Loc,
        stat: NameStatus,
    ) -> GlobalNameResult<'a> {
        self.check_name(automata_name, loc, NameClass::Automata, &stat)?;
        let net_table = self.networks.get_mut(net_name).unwrap();
        if let Some(_) = net_table.names.get(automata_name) {
            net_table.names.remove(automata_name);
        }
        let net_table = self.networks.get_mut(net_name).unwrap();
        let automata_table = AutomataNameTable::new(loc);
        net_table.automata.insert(automata_name, automata_table);
        self.status = CollectionStatus::Automata {
            net: net_name,
            automata: automata_name,
        };
        Ok(self)
    }

    fn insert_network_name(
        mut self,
        name: &'a str,
        class: NetworkName,
        loc: Loc,
        stat: NameStatus,
    ) -> GlobalNameResult<'a> {
        let name_stat = self.check_name(name, loc, &class, &stat)?;
        let name_stat = name_stat.get_name_status();
        if let CollectionStatus::Network(net_name) = self.status {
            let net_table = self.networks.get_mut(net_name).unwrap();
            let stat = NameStatus::next(name_stat, stat);
            let info = NetworkNameInfo { stat, class, loc };
            net_table.names.insert(name, info);
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
        stat: &NameStatus,
    ) -> Result<CheckNameResult, NameError<'a>> {
        let curr_class = curr_class.into();
        match self.status {
            CollectionStatus::Global => self.check_global_name(name, loc, curr_class),
            CollectionStatus::Network(net) => {
                self.check_network_name(name, net, loc, curr_class, stat)
            }
            CollectionStatus::Automata { net, automata } => {
                self.check_automata_name(name, net, automata, loc, curr_class, stat)
            }
            CollectionStatus::Request(_) => Ok(CheckNameResult::NameStatus(NameStatus::Unknown)),
        }
    }

    fn check_global_name(
        &self,
        name: &'a str,
        loc: Loc,
        cls: NameClass,
    ) -> Result<CheckNameResult, NameError<'a>> {
        if let Some(prev) = self.networks.get(name) {
            new_name_error! {name, NameClass::Network, cls, prev.loc, loc}
        } else {
            Ok(CheckNameResult::NameStatus(NameStatus::Unknown))
        }
    }

    fn check_network_name(
        &self,
        name: &'a str,
        net_name: &'a str,
        loc: Loc,
        curr_class: NameClass,
        stat: &NameStatus,
    ) -> Result<CheckNameResult, NameError<'a>> {
        if net_name == name {
            let prev_loc = self.get_container_location();
            new_name_error! {name, NameClass::Network, curr_class, prev_loc, loc}
        } else {
            let net_table = self.networks.get(net_name).unwrap();
            net_table.check_network_name(name, curr_class, loc, *stat)
        }
    }

    fn check_automata_name(
        &self,
        name: &'a str,
        net_name: &'a str,
        automata_name: &'a str,
        loc: Loc,
        curr_class: NameClass,
        stat: &NameStatus,
    ) -> Result<CheckNameResult, NameError<'a>> {
        if net_name == name {
            let prev_loc = self.get_container_location();
            new_name_error! {name, NameClass::Network, curr_class, prev_loc, loc}
        } else {
            let net_table = self.networks.get(net_name).unwrap();
            net_table.check_automata_name(name, automata_name, curr_class, loc, *stat)
        }
    }

    fn get_container_location(&self) -> Loc {
        match self.status {
            CollectionStatus::Automata{net, automata} => {
                let net_table = self.networks.get(net).unwrap();
                let automata_table = net_table.automata.get(automata).unwrap();
                automata_table.loc
            },
            CollectionStatus::Network(net) => {
                let net_table = self.networks.get(net).unwrap();
                net_table.loc
            },
            CollectionStatus::Request(req) => {
                let req_table = self.requests.get(req).unwrap();
                req_table.loc
            }
            CollectionStatus::Global => unreachable!(),
        }
    }
}

fn validate_labels<'b, 'a: 'b>(
    table: &NetworkNameTable<'a>,
    labels: impl Iterator<Item = &'b Vec<&'a str>>,
    class: NameClass,
) -> Result<(), NameError<'a>> {
    for lbls in labels {
        for lbl in lbls {
            if let Some(cls) = table.get_name_class(lbl) {
                if cls != class {
                    return Err(MismatchedType {
                        name: lbl,
                        orig: cls,
                        curr: class,
                    })?;
                }
            } else {
                return Err(UndefinedLabel { name: lbl, class })?;
            }
        }
    }

    Ok(())
}

fn next_stat_and_class(
    res: CheckNameResult,
    stat: NameStatus,
    cls: AutomataName,
) -> (NameStatus, AutomataName) {
    match res {
        CheckNameResult::AutomataNameStatus((curr_stat, class)) => (
            NameStatus::next(curr_stat, stat),
            AutomataName::next(class, cls),
        ),
        CheckNameResult::NameStatus(curr_stat) => (NameStatus::next(curr_stat, stat), cls),
    }
}

enum CheckNameResult {
    NameStatus(NameStatus),
    AutomataNameStatus((NameStatus, AutomataName)),
}

impl CheckNameResult {
    fn get_name_status(self) -> NameStatus {
        match self {
            Self::NameStatus(stat) => stat,
            Self::AutomataNameStatus(_) => panic!(),
        }
    }
}

pub type GlobalNameResult<'a> = Result<GlobalNameTable<'a>, NameError<'a>>;

#[derive(Debug)]
enum CollectionStatus<'a> {
    Network(&'a str),
    Request(&'a str),
    Automata { net: &'a str, automata: &'a str },
    Global,
}

#[derive(Debug)]
struct NetworkNameTable<'a> {
    loc: Loc,
    stat: NameStatus,
    names: HashMap<&'a str, NetworkNameInfo>,
    automata: HashMap<&'a str, AutomataNameTable<'a>>,
}

impl<'a> NetworkNameTable<'a> {
    fn new(loc: Loc, stat: NameStatus) -> Self {
        NetworkNameTable {
            loc,
            stat,
            names: HashMap::new(),
            automata: HashMap::new(),
        }
    }

    fn check_network_name<T: Into<NameClass>>(
        &self,
        name: &'a str,
        cls: T,
        loc: Loc,
        stat: NameStatus,
    ) -> Result<CheckNameResult, NameError<'a>> {
        let cls = cls.into();
        self.check_network_level_names(name, cls, loc, stat)?;
        self.check_automata_items(name, cls, loc)
    }

    fn check_automata_name<T: Into<NameClass>>(
        &self,
        name: &'a str,
        automata_name: &'a str,
        cls: T,
        loc: Loc,
        stat: NameStatus,
    ) -> Result<CheckNameResult, NameError<'a>> {
        let cls = cls.into();
        self.check_network_level_names(name, cls, loc, stat)?;
        self.check_automata_level_name(name, automata_name, cls, loc, stat)
    }

    fn check_automata_items(
        &self,
        name: &'a str,
        cls: NameClass,
        loc: Loc,
    ) -> Result<CheckNameResult, NameError<'a>> {
        for automata in self.automata.values() {
            if let Some(prev) = automata.names.get(name) {
                let prev_cls: NameClass = (&prev.class).into();
                return new_name_error! {name, prev_cls, cls, prev.loc, loc};
            }
        }
        Ok(CheckNameResult::NameStatus(NameStatus::Unknown))
    }

    fn check_automata_level_name(
        &self,
        name: &'a str,
        automata_name: &'a str,
        cls: NameClass,
        loc: Loc,
        stat: NameStatus,
    ) -> Result<CheckNameResult, NameError<'a>> {
        let automata_table = self.automata.get(automata_name).unwrap();
        if let Some(prev) = automata_table.names.get(name) {
            check_prev_automata_def(name, prev, cls, stat, loc)
        } else {
            Ok(CheckNameResult::NameStatus(NameStatus::Unknown))
        }
    }

    fn check_network_level_names(
        &self,
        name: &'a str,
        cls: NameClass,
        loc: Loc,
        stat: NameStatus,
    ) -> Result<(), NameError<'a>> {
        if let Some(prev) = self.names.get(name) {
            check_previous_definition(name, &prev.class, cls, prev.loc, loc, prev.stat, stat)?;
            Ok(())
        } else if let Some(prev) = self.automata.get(name) {
            check_previous_definition(
                name,
                NameClass::Automata,
                cls,
                prev.loc,
                loc,
                NameStatus::Defined,
                stat,
            )?;
            Ok(())
        } else {
            Ok(())
        }
    }

    fn get_name_class(&self, name: &str) -> Option<NameClass> {
        if let Some(def) = self.names.get(name) {
            Some((&def.class).into())
        } else if let Some(_) = self.automata.get(name) {
            Some(NameClass::Automata)
        } else {
            for automata in self.automata.values() {
                let res = automata.get_name_class(name);
                if res.is_some() {
                    return res;
                }
            }
            None
        }
    }
}

fn check_prev_automata_def<'a>(
    name: &'a str,
    def: &AutomataInfo,
    cls: NameClass,
    stat: NameStatus,
    loc: Loc,
) -> Result<CheckNameResult, NameError<'a>> {
    let curr_cls = NameClass::from_automata_name(&def.class);
    let res = NameStatus::check_status(&def.stat, &stat);
    match res {
        CheckStatus::Success => {
            let next = NameStatus::next(def.stat, stat);
            Ok(CheckNameResult::AutomataNameStatus((next, def.class)))
        }
        CheckStatus::Failure => new_name_error! {name, curr_cls, cls, def.loc, loc},
    }
}

fn check_previous_definition<'a, T: Into<NameClass>>(
    name: &'a str,
    prev_cls: T,
    curr_cls: NameClass,
    prev_loc: Loc,
    curr_loc: Loc,
    prev_stat: NameStatus,
    curr_stat: NameStatus,
) -> Result<NameStatus, NameError<'a>> {
    let prev_cls = prev_cls.into();
    if prev_cls == curr_cls {
        let res = NameStatus::check_status(&prev_stat, &curr_stat);
        match res {
            CheckStatus::Failure => new_name_error! {name, prev_cls, curr_cls, prev_loc, curr_loc},
            CheckStatus::Success => Ok(NameStatus::next(prev_stat, curr_stat)),
        }
    } else {
        new_name_error! {name, prev_cls, curr_cls, prev_loc, curr_loc}
    }
}

#[derive(Debug)]
struct NetworkNameInfo {
    stat: NameStatus,
    class: NetworkName,
    loc: Loc
}

#[derive(Debug)]
enum NetworkName {
    Link,
    Event,
    ObsLabel,
    RelLabel,
    UnknowAutomata,
}

#[derive(Debug)]
struct AutomataNameTable<'a> {
    names: HashMap<&'a str, AutomataInfo>,
    loc: Loc
}

impl<'a> AutomataNameTable<'a> {
    fn new(loc: Loc) -> Self {
        AutomataNameTable {
            names: HashMap::new(),
            loc
        }
    }

    fn get_name_class(&self, name: &str) -> Option<NameClass> {
        if let Some(def) = self.names.get(name) {
            Some((&def.class).into())
        } else {
            None
        }
    }

    fn insert_name(&mut self, name: &'a str, loc: Loc, class: AutomataName, stat: NameStatus) {
        let info = AutomataInfo { loc, class, stat };
        self.names.insert(name, info);
    }

    fn validate(&self) -> Result<(), NameError<'a>> {
        self.validate_definitions()?;
        self.validate_begin_state()?;
        Ok(())
    }

    fn validate_definitions(&self) -> Result<(), UndefinedNameError<'a>> {
        for (name, item) in self.names.iter() {
            item.stat.validate(name, item.loc)?
        }
        Ok(())
    }

    fn validate_begin_state(&self) -> Result<(), BeginStateError<'a>> {
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

#[derive(Debug, Copy, Clone)]
enum AutomataName {
    State,
    Begin,
    Transition,
}

impl AutomataName {
    fn next(prev: Self, curr: Self) -> Self {
        match (prev, curr) {
            (Self::Begin, _) => Self::Begin,
            (_, Self::Begin) => Self::Begin,
            _ => curr,
        }
    }
}

#[derive(Debug)]
struct AutomataInfo {
    loc: Loc,
    class: AutomataName,
    stat: NameStatus,
}

#[derive(Debug, Copy, Clone)]
enum NameStatus {
    Defined,
    Used,
    Undefined,
    Unknown,
}

impl NameStatus {
    fn next(curr: Self, incoming: Self) -> Self {
        match (curr, incoming) {
            (NameStatus::Unknown, inc) => inc,
            (NameStatus::Used, _) => NameStatus::Used,
            (NameStatus::Defined, NameStatus::Undefined) => NameStatus::Used,
            (NameStatus::Undefined, NameStatus::Defined) => NameStatus::Used,
            (NameStatus::Undefined, NameStatus::Undefined) => NameStatus::Undefined,
            (_, NameStatus::Used) => unreachable!(),
            (NameStatus::Defined, NameStatus::Defined) => unreachable!(),
            (_, NameStatus::Unknown) => unreachable!(),
        }
    }

    fn check_status(curr: &Self, incoming: &Self) -> CheckStatus {
        match (curr, incoming) {
            (NameStatus::Defined, NameStatus::Defined) => CheckStatus::Failure,
            _ => CheckStatus::Success,
        }
    }
}

enum CheckStatus {
    Success,
    Failure,
}

impl<'a> NameStatus {
    fn validate(&self, name: &'a str, loc: Loc) -> Result<(), UndefinedNameError<'a>> {
        match self {
            Self::Undefined => Err(UndefinedNameError { name, loc }),
            _ => Ok(()),
        }
    }
}

#[derive(Debug)]
pub enum NameError<'a> {
    UndefinedNetwork(UndefinedNetwork<'a>),
    NameRidefinitionError(NameRidefinitionError<'a>),
    BeginStateError(BeginStateError<'a>),
    UndefinedNameError(UndefinedNameError<'a>),
    UndefinedLabel(UndefinedLabel<'a>),
    MismatchedType(MismatchedType<'a>),
}

into_name_error! {UndefinedNetwork}
into_name_error! {NameRidefinitionError}
into_name_error! {BeginStateError}
into_name_error! {UndefinedNameError}
into_name_error! {UndefinedLabel}
into_name_error! {MismatchedType}

#[derive(Debug)]
pub struct UndefinedLabel<'a> {
    pub name: &'a str,
    pub class: NameClass,
}

#[derive(Debug)]
pub struct MismatchedType<'a> {
    pub name: &'a str,
    pub orig: NameClass,
    pub curr: NameClass,
}

#[derive(Debug)]
pub struct UndefinedNameError<'a> {
    pub name: &'a str,
    pub loc: Loc,
}

#[derive(Debug)]
pub struct UndefinedNetwork<'a> {
    pub names: Vec<(&'a str, Loc)>,
}

#[derive(Debug)]
pub struct NameRidefinitionError<'a> {
    pub name: &'a str,
    pub orig_loc: Loc,
    pub ridef_loc: Loc,
    pub orig_class: NameClass,
    pub ridef_class: NameClass,
}

#[derive(Debug)]
pub enum BeginStateError<'a> {
    NoBeginState,
    MultipleBeginState(Vec<&'a str>),
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum NameClass {
    Network,
    Request,
    Automata,
    Link,
    Event,
    ObsLabel,
    RelLabel,
    State,
    Transition,
}

impl NameClass {
    fn from_automata_name(cls: &AutomataName) -> Self {
        match cls {
            AutomataName::Begin | AutomataName::State => Self::State,
            AutomataName::Transition => Self::Transition,
        }
    }
}

impl From<&NetworkName> for NameClass {
    fn from(cls: &NetworkName) -> Self {
        match cls {
            NetworkName::UnknowAutomata => Self::Automata,
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


#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn test_correct_name() {
        let name_table = GlobalNameTable::new();
        let name_table = name_table.declare_network("netname", (0, 1)).unwrap();
        let name_table = name_table.declare_event("ev", (4, 5)).unwrap();
        let name_table = name_table.declare_link("lk", (6, 7)).unwrap();
        let name_table = name_table.declare_automata("at", (10, 12)).unwrap();
        let name_table = name_table.declare_begin("s1", (13, 15)).unwrap();
        let name_table = name_table.declare_state("s2", (45, 35)).unwrap();
        let name_table = name_table.exit_automata();
        let name_table = name_table.exit_network();
        let name_table = name_table.insert_request("netname", (45, 123)).unwrap();

        name_table
            .validate()
            .expect("This network is correctly defined");
    }

    #[test]
    fn test_missing_network() {
        let name_table = GlobalNameTable::new();
        let name_table = name_table.insert_request("net", (0, 1)).unwrap();
        let err = name_table
            .validate()
            .expect_err("`net` is not a defined network");
        match err {
            NameError::UndefinedNetwork(err) => {
                assert_eq!(err.names, vec![("net", (0, 1))])
            }
            _ => panic!("expected UndefinedNetwork, found: {:?}", err),
        }
    }

    #[test]
    fn test_ridefined_name() {
        let name_table = GlobalNameTable::new();
        let name_table = name_table.declare_network("netname", (0, 1)).unwrap();
        let name_table = name_table.declare_event("ev", (4, 5)).unwrap();
        let name_table = name_table.declare_link("lk", (6, 7)).unwrap();
        let name_table = name_table.declare_automata("at", (10, 12)).unwrap();
        let name_table = name_table.exit_automata();
        let err = name_table
            .declare_link("at", (15, 18))
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
        let name_table = name_table.declare_network("netname", (0, 1)).unwrap();
        let name_table = name_table.declare_event("ev", (4, 5)).unwrap();
        let name_table = name_table.declare_link("lk", (6, 7)).unwrap();
        let name_table = name_table.declare_automata("at", (10, 12)).unwrap();
        let name_table = name_table.declare_begin("s0", (45, 12)).unwrap();
        let name_table = name_table.declare_begin("s1", (56, 142)).unwrap();

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
        let name_table = name_table.declare_network("netname", (0, 1)).unwrap();
        let name_table = name_table.declare_event("ev", (4, 5)).unwrap();
        let name_table = name_table.declare_link("lk", (6, 7)).unwrap();
        let name_table = name_table.declare_automata("at", (10, 12)).unwrap();

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
        let name_table = name_table.declare_network("netname", (0, 1)).unwrap();
        let name_table = name_table.declare_event("ev", (4, 5)).unwrap();
        let name_table = name_table.declare_link("lk", (6, 7)).unwrap();
        let name_table = name_table.declare_automata("at", (10, 12)).unwrap();
        let name_table = name_table.declare_begin("s0", (45, 12)).unwrap();
        let name_table = name_table.declare_state("s1", (56, 142)).unwrap();

        let err = name_table
            .declare_state("s1", (67, 132))
            .expect_err("State s1 is defined twice");

        match err {
            NameError::NameRidefinitionError(err) => {
                assert_eq!(err.name, "s1");
                assert_eq!(err.orig_class, NameClass::State);
                assert_eq!(err.ridef_class, NameClass::State);
            }
            err => panic!("expected NameRidefinitionError: found `{:?}`", err),
        }
    }

    #[test]
    fn test_undefined_automata() {
        let name_table = GlobalNameTable::new();
        let name_table = name_table.declare_network("net", (0, 1)).unwrap();
        let name_table = name_table.declare_link("L1", (0, 1)).unwrap();
        let name_table = name_table.add_automata("A", (10, 10)).unwrap();
        let name_table = name_table.add_automata("B", (10, 10)).unwrap();

        let name_table = name_table.declare_automata("A", (14, 15)).unwrap();

        let name_table = name_table.exit_automata();
        let name_table = name_table.exit_network();

        let err = name_table
            .validate()
            .expect_err("Automata `b` is not defined");
        match err {
            NameError::UndefinedNameError(err) => {
                assert_eq!(err.name, "B");
            }
            _ => panic!("expected UndefinedNameError, found {:?}", err),
        }
    }

    #[test]
    fn test_event_name_riusage() {
        let name_table = GlobalNameTable::new();
        let name_table = name_table.declare_network("net", (0, 1)).unwrap();
        let name_table = name_table.declare_link("L1", (0, 1)).unwrap();
        let name_table = name_table.add_event("A", (67, 23)).unwrap();
        let err = name_table
            .add_automata("A", (10, 10))
            .expect_err("Name `A` is found as Event");
        match err {
            NameError::NameRidefinitionError(err) => {
                assert_eq!(err.name, "A");
                assert_eq!(err.orig_class, NameClass::Event);
                assert_eq!(err.ridef_class, NameClass::Automata);
            }
            _ => panic!("expected NameRidefinitionError, found {:?}", err),
        }
    }

    #[test]
    fn test_event_name_ridefinition() {
        let name_table = GlobalNameTable::new();
        let name_table = name_table.declare_network("net", (0, 1)).unwrap();
        let name_table = name_table.declare_link("L1", (0, 1)).unwrap();
        let name_table = name_table.add_event("A", (67, 23)).unwrap();
        let err = name_table
            .declare_automata("A", (10, 10))
            .expect_err("Name `A` is found as Event");
        match err {
            NameError::NameRidefinitionError(err) => {
                assert_eq!(err.name, "A");
                assert_eq!(err.orig_class, NameClass::Event);
                assert_eq!(err.ridef_class, NameClass::Automata);
            }
            _ => panic!("expected NameRidefinitionError, found {:?}", err),
        }
    }

    #[test]
    fn test_automata_name_riusage() {
        let name_table = GlobalNameTable::new();
        let name_table = name_table.declare_network("net", (0, 1)).unwrap();
        let name_table = name_table.declare_link("L1", (0, 1)).unwrap();
        let name_table = name_table.add_automata("A", (67, 12)).unwrap();
        let err = name_table
            .add_event("A", (67, 23))
            .expect_err("Name `A` is found as Automata");

        match err {
            NameError::NameRidefinitionError(err) => {
                assert_eq!(err.name, "A");
                assert_eq!(err.orig_class, NameClass::Automata);
                assert_eq!(err.ridef_class, NameClass::Event);
            }
            _ => panic!("expected NameRidefinitionError, found {:?}", err),
        }
    }

    #[test]
    fn test_automata_name_ridefinition() {
        let name_table = GlobalNameTable::new();
        let name_table = name_table.declare_network("net", (0, 1)).unwrap();
        let name_table = name_table.declare_link("L1", (0, 1)).unwrap();
        let name_table = name_table.declare_automata("A", (67, 12)).unwrap();
        let err = name_table
            .add_event("A", (67, 23))
            .expect_err("Name `A` is Declared as Automata");

        match err {
            NameError::NameRidefinitionError(err) => {
                assert_eq!(err.name, "A");
                assert_eq!(err.orig_class, NameClass::Automata);
                assert_eq!(err.ridef_class, NameClass::Event);
            }
            _ => panic!("expected NameRidefinitionError, found {:?}", err),
        }
    }
}
