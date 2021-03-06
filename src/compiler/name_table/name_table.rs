/*
    Code in this file collects all
    names defined in the specification file
    and validate them:  ensuring that
    there aren't reused name
    and ensuring some simple validation:
        - ensure that each name is defined and not just used
        - ensure that names used in requests are actually defined
        names of the correct type
*/

use ahash::AHashMap;

use super::name_error::*;
use crate::new_name_error;

use super::super::index_name_table::{
    AutomataNamesFactory, GlobalIndexTable, GlobalIndexTableFactory, NetworkIndexTableFactory,
};
use super::class_index::ClassIndex;
use super::name_class::NameClass;
use super::request_table::{Request, RequestTable};
use super::Loc;

/**
 * This struct contain both the definition
 * in the network and the requests on this network
 */
#[derive(Debug)]
pub struct GlobalNameTable<'a> {
    networks: AHashMap<&'a str, NetworkNameTable<'a>>,
    requests: AHashMap<&'a str, RequestTable<'a>>,

    status: CollectionStatus<'a>,
    net_index: usize,
}

/*
    declare_ methods are used when a name is
    declare, i.e. automata declaration

    insert_ and add_ methods are used when a name is
    used by other declaration, i.e.
    automata name when in link declaration

    This methods allows to
        - identify name reuse
        - identify quickly if
        a name is used in the wrong
        contex.

*/
impl<'a> GlobalNameTable<'a> {
    pub fn new() -> Self {
        Self {
            networks: AHashMap::new(),
            requests: AHashMap::new(),
            status: CollectionStatus::Global,
            net_index: 0,
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

    /*
        Most of the declare_ methods are just
        a convenience wrapper around the actual
        name insertion.
    */
    pub fn declare_link(self, name: &'a str, loc: Loc) -> GlobalNameResult<'a> {
        let output = self.insert_network_name(name, NetworkName::Link, loc, NameStatus::Defined)?;
        Ok(output.set_network_index(name))
    }

    pub fn declare_rel_label(self, name: &'a str, loc: Loc) -> GlobalNameResult<'a> {
        let output =
            self.insert_network_name(name, NetworkName::RelLabel, loc, NameStatus::Defined)?;
        Ok(output.set_network_index(name))
    }

    pub fn declare_obs_label(self, name: &'a str, loc: Loc) -> GlobalNameResult<'a> {
        let output =
            self.insert_network_name(name, NetworkName::ObsLabel, loc, NameStatus::Defined)?;
        Ok(output.set_network_index(name))
    }

    pub fn declare_event(self, name: &'a str, loc: Loc) -> GlobalNameResult<'a> {
        let output =
            self.insert_network_name(name, NetworkName::Event, loc, NameStatus::Defined)?;
        Ok(output.set_network_index(name))
    }

    fn set_network_index(mut self, name: &'a str) -> Self {
        if let CollectionStatus::Network(net_name) = self.status {
            let net_table = self.networks.get_mut(net_name).unwrap();
            let mut item = net_table.names.get_mut(name).unwrap();
            let cls = item.class;
            let index = net_table.counter.get_count(cls);
            item.index = index;
            self
        } else {
            panic!("cannot call `add_automata` outside netword")
        }
    }

    pub fn declare_begin(self, name: &'a str, loc: Loc) -> GlobalNameResult<'a> {
        let output =
            self.insert_automata_name(name, loc, AutomataName::Begin, NameStatus::Defined)?;
        Ok(output.set_automata_index(name))
    }

    pub fn declare_state(self, name: &'a str, loc: Loc) -> GlobalNameResult<'a> {
        let output =
            self.insert_automata_name(name, loc, AutomataName::State, NameStatus::Defined)?;
        Ok(output.set_automata_index(name))
    }

    pub fn declare_transition(self, name: &'a str, loc: Loc) -> GlobalNameResult<'a> {
        let output =
            self.insert_automata_name(name, loc, AutomataName::Transition, NameStatus::Defined)?;
        Ok(output.set_automata_index(name))
    }

    fn set_automata_index(mut self, name: &'a str) -> Self {
        if let CollectionStatus::Automata { net, automata } = self.status {
            let net_table = self.networks.get_mut(net).unwrap();
            let auto_table = net_table.automata.get_mut(automata).unwrap();
            auto_table.set_index(name);
            self
        } else {
            panic!("call `set_automata_index` outside automata block");
        }
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

    /*
        Most of the add_ method are gust convenience
        wrapper around the actual name insertion method
    */
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

    pub fn get_network_name_index(&self, net_name: &str, item_name: &str) -> usize {
        let net_table = self.networks.get(net_name).unwrap();
        if let Some(item) = net_table.names.get(item_name) {
            item.index
        } else {
            let automata = net_table.automata.get(item_name).unwrap();
            automata.index
        }
    }

    pub fn get_file_index(&self, net_name: &str, name: &str) -> usize {
        let req_table = self.requests.get(net_name).unwrap();
        req_table.get_file_index(name)
    }

    pub fn get_automata_name_index(
        &self,
        net_name: &str,
        automata_name: &str,
        item_name: &str,
    ) -> usize {
        let net_table = self.networks.get(net_name).unwrap();
        let automata = net_table.automata.get(automata_name).unwrap();
        let item = automata.names.get(item_name).unwrap();
        item.index
    }

    pub fn get_index_table(mut self) -> GlobalIndexTable<'a> {
        let mut factory = GlobalIndexTableFactory::default();
        for (name, table) in self.networks.into_iter() {
            let (net_factory, index) = table.into_index_table(name);
            let req_table = self.requests.remove(name);
            let net_factory = if let Some(req_table) = req_table {
                let mut net_factory = net_factory;
                net_factory.add_files(req_table.get_files());
                net_factory
            } else {
                net_factory
            };
            factory.add_network(net_factory, index);
        }
        factory.build()
    }

    fn validate_network(self) -> GlobalNameResult<'a> {
        for (net_name, table) in self.networks.iter() {
            table.validate(net_name)?;
        }

        Ok(self)
    }

    fn validate_requests(self) -> GlobalNameResult<'a> {
        let names = self.get_undefined_network_names();
        if names.len() > 0 {
            let err = UndefinedNetwork { names };
            Err(NameError::UndefinedNetwork(err))
        } else {
            self.validate_request_labels()
        }
    }

    fn get_undefined_network_names(&self) -> Vec<(&'a str, Loc)> {
        self.requests
            .iter()
            .filter(|(k, _)| !self.networks.contains_key(*k))
            .map(|(k, v)| (*k, v.get_location()))
            .collect()
    }

    fn validate_request_labels(self) -> GlobalNameResult<'a> {
        for (net_name, req) in self.requests.iter() {
            let net_table = self.networks.get(net_name).unwrap();
            validate_labels(net_table, req.get_linspace_labels(), NameClass::ObsLabel)?;
            validate_labels(net_table, req.get_diagnosis_labels(), NameClass::ObsLabel)?;
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
        self.networks
            .insert(name, NetworkNameTable::new(loc, stat, self.net_index));
        self.net_index += 1;
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
        let index = net_table.counter.get_count(NetworkName::Automata);
        let automata_table = AutomataNameTable::new(automata_name, loc, index);
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
        match &self.status {
            CollectionStatus::Automata { net, automata: _ } | CollectionStatus::Network(net) => {
                let net_class: NameClass = (&class).into();
                let name_stat = self.check_network_name(name, net, loc, net_class, &stat)?;
                let name_stat = name_stat.get_name_status();
                let net_table = self.networks.get_mut(net).unwrap();
                let stat = NameStatus::next(name_stat, stat);
                if let Some(info) = net_table.names.get_mut(name) {
                    info.stat = stat;
                    info.loc = loc;
                } else {
                    let info = NetworkNameInfo {
                        stat,
                        class,
                        loc,
                        index: 0,
                    };
                    net_table.names.insert(name, info);
                }
                Ok(self)
            }
            _ => panic!("Call add_automata in state: {:?}", self.status),
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
            CollectionStatus::Automata { net, automata } => {
                let net_table = self.networks.get(net).unwrap();
                let automata_table = net_table.automata.get(automata).unwrap();
                automata_table.loc
            }
            CollectionStatus::Network(net) => {
                let net_table = self.networks.get(net).unwrap();
                net_table.loc
            }
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

#[derive(Debug)]
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
    counter: ClassIndex<NetworkName>,
    names: AHashMap<&'a str, NetworkNameInfo>,
    automata: AHashMap<&'a str, AutomataNameTable<'a>>,
    index: usize,
}

impl<'a> NetworkNameTable<'a> {
    fn new(loc: Loc, stat: NameStatus, index: usize) -> Self {
        NetworkNameTable {
            loc,
            stat,
            counter: ClassIndex::new(),
            names: AHashMap::new(),
            automata: AHashMap::new(),
            index,
        }
    }

    fn into_index_table(self, name: &'a str) -> (NetworkIndexTableFactory<'a>, usize) {
        let mut factory = NetworkIndexTableFactory::new(name);
        for (name, info) in self.names.into_iter() {
            match info.class {
                NetworkName::Event => factory.add_ev_name(name, info.index),
                NetworkName::Link => factory.add_link_name(name, info.index),
                NetworkName::ObsLabel => factory.add_obs_label(name, info.index),
                NetworkName::RelLabel => factory.add_rel_label(name, info.index),
                _ => {}
            }
        }

        for (name, table) in self.automata.into_iter() {
            let (auto_factory, index) = table.into_index_table(name);
            factory.add_automata(auto_factory, index);
        }

        (factory, self.index)
    }

    fn check_network_name<T: Into<NameClass>>(
        &self,
        name: &'a str,
        cls: T,
        loc: Loc,
        stat: NameStatus,
    ) -> Result<CheckNameResult, NameError<'a>> {
        let cls = cls.into();
        let stat = self.check_network_level_names(name, cls, loc, stat)?;
        if let Some(stat) = stat {
            Ok(CheckNameResult::NameStatus(stat))
        } else {
            self.check_automata_items(name, cls, loc)
        }
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
    ) -> Result<Option<NameStatus>, NameError<'a>> {
        if let Some(prev) = self.names.get(name) {
            let stat =
                check_previous_definition(name, &prev.class, cls, prev.loc, loc, prev.stat, stat)?;
            Ok(Some(stat))
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
            Ok(None)
        } else {
            Ok(None)
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

    fn validate(&self, net_name: &'a str) -> Result<(), NameError<'a>> {
        self.stat.validate(net_name, self.loc)?;
        for (name, item) in self.names.iter() {
            item.validate(name)?;
        }
        for automata in self.automata.values() {
            automata.validate()?;
        }

        Ok(())
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
    index: usize,
    loc: Loc,
}

impl<'a> NetworkNameInfo {
    fn validate(&self, name: &'a str) -> Result<(), UndefinedNameError<'a>> {
        self.stat.validate(name, self.loc)
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone)]
enum NetworkName {
    Automata,
    Link,
    Event,
    ObsLabel,
    RelLabel,
    UnknowAutomata,
}

#[derive(Debug)]
struct AutomataNameTable<'a> {
    names: AHashMap<&'a str, AutomataInfo>,
    loc: Loc,
    index: usize,
    counter: ClassIndex<AutomataName>,
    name: &'a str,
}

impl<'a> AutomataNameTable<'a> {
    fn new(name: &'a str, loc: Loc, index: usize) -> Self {
        AutomataNameTable {
            names: AHashMap::new(),
            loc,
            index,
            counter: ClassIndex::new(),
            name,
        }
    }

    fn into_index_table(self, name: &'a str) -> (AutomataNamesFactory<'a>, usize) {
        let mut factory = AutomataNamesFactory::new(name);
        for (name, info) in self.names.into_iter() {
            match info.class {
                AutomataName::State | AutomataName::Begin => factory.add_state(name, info.index),
                AutomataName::Transition => factory.add_transition(name, info.index),
            }
        }

        (factory, self.index)
    }

    fn set_index(&mut self, name: &str) {
        let item = self.names.get_mut(name).unwrap();
        let cls = item.class;
        let index = self.counter.get_count(cls);
        item.index = index;
    }

    fn get_name_class(&self, name: &str) -> Option<NameClass> {
        if let Some(def) = self.names.get(name) {
            Some((&def.class).into())
        } else {
            None
        }
    }

    fn insert_name(&mut self, name: &'a str, loc: Loc, class: AutomataName, stat: NameStatus) {
        if let Some(info) = self.names.get_mut(name) {
            info.loc = loc;
            info.stat = stat;
        } else {
            let info = AutomataInfo {
                loc,
                class,
                stat,
                index: 0,
            };
            self.names.insert(name, info);
        }
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
        let class = match begin_states.len() {
            0 => Some(BeginStateErrorClass::NoBeginState),
            1 => None,
            _ => Some(BeginStateErrorClass::MultipleBeginState(begin_states)),
        };

        if let Some(class) = class {
            let err = BeginStateError {
                name: self.name,
                loc: self.loc,
                class,
            };
            Err(err)
        } else {
            Ok(())
        }
    }
}

#[derive(Debug, Copy, Clone)]
enum AutomataName {
    State,
    Begin,
    Transition,
}

impl PartialEq for AutomataName {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Transition, Self::Transition) => true,
            (Self::Begin, Self::Begin) => true,
            (Self::Begin, Self::State) => true,
            (Self::State, Self::State) => true,
            (Self::State, Self::Begin) => true,
            _ => false,
        }
    }
}

impl Eq for AutomataName {}

impl std::hash::Hash for AutomataName {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let val = match self {
            Self::State | Self::Begin => 0,
            _ => 1,
        };
        state.write(&[val])
    }
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
    index: usize,
}

#[derive(Debug, Copy, Clone)]
enum NameStatus {
    Defined,
    Used,
    Undefined,
    Unknown,
}

impl<'a> NameStatus {
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

    fn validate(&self, name: &'a str, loc: Loc) -> Result<(), UndefinedNameError<'a>> {
        match self {
            Self::Undefined => Err(UndefinedNameError { name, loc }),
            _ => Ok(()),
        }
    }
}

enum CheckStatus {
    Success,
    Failure,
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
            NetworkName::Automata => unreachable!(),
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
            NameError::BeginStateError(err) => match err.class {
                BeginStateErrorClass::MultipleBeginState(states) => {
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
            NameError::BeginStateError(err) => match err.class {
                BeginStateErrorClass::NoBeginState => {}
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
