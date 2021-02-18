use super::Location;
use std::collections::HashMap;

type Loc = (usize, usize);

pub enum NameError<'a> {
    Global(GlobalNameError<'a>),
}

impl<'a> From<GlobalNameError<'a>> for NameError<'a> {
    fn from(err: GlobalNameError<'a>) -> Self {
        Self::Global(err)
    }
}

#[derive(Debug, PartialEq)]
enum GlobalClassName {
    Network,
    Request,
}

#[derive(Debug)]
pub struct GlobalNameError<'a> {
    name: &'a str,
    class: GlobalClassName,
    orig_loc: Location,
    new_loc: Location,
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

pub struct UndefinedNetwork<'a> {
    names: Vec<(&'a str, Loc)>,
}

type GlobalNameResult<'a> = Result<GlobalNameTable<'a>, GlobalNameError<'a>>;

#[derive(Debug)]
pub struct GlobalNameTable<'a> {
    names: HashMap<&'a str, NetworkInfo>,
}

impl<'a> GlobalNameTable<'a> {
    pub fn new() -> Self {
        Self {
            names: HashMap::new(),
        }
    }

    pub fn validate(self) -> Result<Self, UndefinedNetwork<'a>> {
        let undef_nets: Vec<(&'a str, Loc)> = self
            .names
            .iter()
            .filter_map(|(name, net_info)| {
                if net_info.network_loc.is_none() {
                    Some((*name, net_info.request_loc.unwrap()))
                } else {
                    None
                }
            })
            .collect();
        if undef_nets.len() == 0 {
            Ok(self)
        } else {
            Err(UndefinedNetwork { names: undef_nets })
        }
    }

    pub fn insert_network(mut self, name: &'a str, loc: (usize, usize)) -> GlobalNameResult {
        self.insert_name(name, GlobalClassName::Network, loc)
    }

    pub fn insert_request(mut self, name: &'a str, loc: (usize, usize)) -> GlobalNameResult {
        self.insert_name(name, GlobalClassName::Request, loc)
    }

    fn insert_name(
        mut self,
        name: &'a str,
        class: GlobalClassName,
        loc: (usize, usize),
    ) -> GlobalNameResult {
        if self.names.contains_key(name) {
            self.update_name_info(name, class, loc)
        } else {
            self.add_new_name(name, class, loc)
        }
    }

    fn update_name_info(
        mut self,
        name: &'a str,
        class: GlobalClassName,
        loc: Loc,
    ) -> GlobalNameResult {
        let prev = self.names.get_mut(name).unwrap();

        let err = match (&prev.state, class) {
            (NetworkDefinitionState::NetworkDefined, GlobalClassName::Request) => {
                prev.state = NetworkDefinitionState::FullDefined;
                prev.request_loc = Some(loc);
                None
            }
            (NetworkDefinitionState::RequestDefined, GlobalClassName::Network) => {
                prev.state = NetworkDefinitionState::FullDefined;
                prev.network_loc = Some(loc);
                None
            }
            (_, class) => {
                let orig_loc = match class {
                    GlobalClassName::Network => prev.network_loc.unwrap(),
                    GlobalClassName::Request => prev.request_loc.unwrap(),
                };
                let err = GlobalNameError::new(name, class, orig_loc, loc);
                Some(err)
            }
        };

        if let Some(err) = err {
            Err(err)
        } else {
            Ok(self)
        }
    }

    fn add_new_name(
        mut self,
        name: &'a str,
        class: GlobalClassName,
        loc: (usize, usize),
    ) -> GlobalNameResult {
        let info = match class {
            GlobalClassName::Network => NetworkInfo::new_network(loc),
            GlobalClassName::Request => NetworkInfo::new_request(loc),
        };
        self.names.insert(name, info);
        Ok(self)
    }
}

#[derive(Debug)]
struct NetworkInfo {
    state: NetworkDefinitionState,
    network_loc: Option<Loc>,
    request_loc: Option<Loc>,
}

impl NetworkInfo {
    fn new_network(loc: (usize, usize)) -> NetworkInfo {
        Self {
            state: NetworkDefinitionState::NetworkDefined,
            network_loc: Some(loc),
            request_loc: None,
        }
    }

    fn new_request(loc: (usize, usize)) -> Self {
        Self {
            state: NetworkDefinitionState::RequestDefined,
            network_loc: None,
            request_loc: Some(loc),
        }
    }
}

#[derive(Debug)]
enum NetworkDefinitionState {
    RequestDefined,
    NetworkDefined,
    FullDefined,
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn test_global_name_ridefinition() {
        let name_table = GlobalNameTable::new();

        let name_table = name_table.insert_network("testNet", (0, 1)).unwrap();
        let name_table = name_table.insert_request("testNet", (5, 6)).unwrap();

        let result = name_table.insert_network("testNet", (10, 20));
        match result {
            Ok(_) => panic!("testNet is redefined"),
            Err(err) => {
                assert_eq!(err.name, "testNet");
                assert_eq!(err.class, GlobalClassName::Network);

                assert_eq!(err.orig_loc.begin, 0);
                assert_eq!(err.orig_loc.end, 1);

                assert_eq!(err.new_loc.begin, 10);
                assert_eq!(err.new_loc.end, 20);
            }
        }
    }

    #[test]
    fn test_global_names() {
        let name_table = GlobalNameTable::new();

        let names = ["testA", "testB", "testC", "testD"];
        let name_table = names
            .iter()
            .try_fold(name_table, |nt, name| nt.insert_network(name, (0, 0)))
            .unwrap();
        names
            .iter()
            .try_fold(name_table, |nt, name| nt.insert_request(name, (0, 0)))
            .unwrap();
    }


    #[test]
    fn test_undefined_network() {
        let name_table = GlobalNameTable::new();

        let name_table = name_table.insert_network("a", (0, 1)).unwrap();
        let name_table = name_table.insert_request("a", (2, 4)).unwrap();

        let name_table = name_table.insert_request("b", (5, 6)).unwrap();
        let name_table = name_table.insert_request("c", (8, 9)).unwrap();
        
        let name_table = name_table.insert_network("d", (10, 11)).unwrap();
        let name_table = name_table.insert_network("c", (12, 13)).unwrap();


        let undefined = name_table.validate().unwrap_err();
        
        assert_eq!(undefined.names.len(), 1);
        let err = undefined.names[0];
        assert_eq!(err.0, "b");
        assert_eq!(err.1, (5, 6));
    }


}
