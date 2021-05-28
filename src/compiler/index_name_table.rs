use crate::utils::auto_sort;

pub struct GlobalIndexTable<'a> {
    networks: Vec<NetworkIndexTable<'a>>,
}

impl<'a> GlobalIndexTable<'a> {
    pub fn get_network_table(&'a self, index: usize) -> &NetworkIndexTable<'a> {
        &self.networks[index]
    }
}

pub struct NetworkIndexTable<'a> {
    name: &'a str,
    net_names: NetNames<'a>,
    automata_names: Vec<AutomataNames<'a>>,
}

impl<'a> NetworkIndexTable<'a> {
    pub fn get_name(&self) -> &str {
        self.name
    }

    pub fn get_network_names(&self) -> &NetNames<'a> {
        &self.net_names
    }

    pub fn get_automata_names(&self, index: usize) -> &AutomataNames<'a> {
        &self.automata_names[index]
    }
}

pub struct NetNames<'a> {
    rel_names: Vec<&'a str>,
    obs_names: Vec<&'a str>,
    ev_names: Vec<&'a str>,
    link_names: Vec<&'a str>,
}

impl<'a> NetNames<'a> {
    pub fn get_rel_name(&self, index: usize) -> &str {
        &self.rel_names[index]
    }

    pub fn get_obs_name(&self, index: usize) -> &str {
        &self.obs_names[index]
    }

    pub fn get_ev_name(&self, index: usize) -> &str {
        &self.ev_names[index]
    }

    pub fn get_link_name(&self, index: usize) -> &str {
        &self.link_names[index]
    }
}

pub struct AutomataNames<'a> {
    name: &'a str,
    state_names: Vec<&'a str>,
    trans_names: Vec<&'a str>,
}

impl<'a> AutomataNames<'a> {
    pub fn get_name(&self) -> &str {
        self.name
    }

    pub fn get_state_name(&self, index: usize) -> &str {
        &self.state_names[index]
    }

    pub fn get_transition_name(&self, index: usize) -> &str {
        &self.trans_names[index]
    }
}

#[derive(Default)]
pub struct GlobalIndexTableFactory<'a> {
    networks: Vec<(NetworkIndexTableFactory<'a>, usize)>,
}

macro_rules! build {
    ($name:expr) => {{
        let mut tmp = $name.into_iter().map(|(f, i)| (f.build(), i));
        auto_sort(&mut tmp)
    }};
}

macro_rules! sort {
    ($name:expr) => {
        auto_sort(&mut $name.into_iter())
    };
}

macro_rules! add_name {
    ($field:ident, $func:ident) => {
        pub fn $func(&mut self, name: &'a str, index: usize) {
            self.$field.push((name, index))
        }
    };
}

macro_rules! add_net_name {
    ($name:ident) => {
        pub fn $name(&mut self, name: &'a str, index: usize) {
            self.net_names.$name(name, index);
        }
    };
}

impl<'a> GlobalIndexTableFactory<'a> {
    pub fn add_network(&mut self, factory: NetworkIndexTableFactory<'a>, index: usize)  {
        self.networks.push((factory, index));
    }

    pub fn build(self) -> GlobalIndexTable<'a> {
        let networks = build! { self.networks };
        GlobalIndexTable { networks }
    }
}

#[derive(Default)]
pub struct NetworkIndexTableFactory<'a> {
    name: &'a str,
    net_names: NetNamesFactory<'a>,
    automata_names: Vec<(AutomataNamesFactory<'a>, usize)>,
}

impl<'a> NetworkIndexTableFactory<'a> {
    pub fn new(name: &'a str) -> Self {
        Self {
            name,
            ..Default::default()
        }
    }

    add_net_name! {add_rel_label}
    add_net_name! {add_obs_label}
    add_net_name! {add_ev_name}
    add_net_name! {add_link_name}

    pub fn add_automata(&mut self, factory: AutomataNamesFactory<'a>, index: usize)  {
        self.automata_names.push((factory, index));
    }

    fn build(self) -> NetworkIndexTable<'a> {
        let automata_names = build! { self.automata_names };
        NetworkIndexTable {
            name: self.name,
            net_names: self.net_names.build(),
            automata_names,
        }
    }
}

#[derive(Default)]
pub struct NetNamesFactory<'a> {
    rel_names: Vec<(&'a str, usize)>,
    obs_names: Vec<(&'a str, usize)>,
    ev_names: Vec<(&'a str, usize)>,
    link_names: Vec<(&'a str, usize)>,
}

impl<'a> NetNamesFactory<'a> {
    fn build(self) -> NetNames<'a> {
        let rel_names = sort! { self.rel_names };
        let obs_names = sort! { self.obs_names };
        let ev_names = sort! { self.ev_names };
        let link_names = sort! { self.link_names };
        NetNames {
            rel_names,
            obs_names,
            ev_names,
            link_names,
        }
    }

    add_name! {rel_names, add_rel_label}
    add_name! {obs_names, add_obs_label}
    add_name! {ev_names, add_ev_name}
    add_name! {link_names, add_link_name}
}

pub struct AutomataNamesFactory<'a> {
    name: &'a str,
    state_names: Vec<(&'a str, usize)>,
    trans_names: Vec<(&'a str, usize)>,
}

impl<'a> AutomataNamesFactory<'a> {
    pub fn new(name: &'a str) -> Self {
        Self {
            name,
            state_names: Vec::new(),
            trans_names: Vec::new(),
        }
    }

    fn build(self) -> AutomataNames<'a> {
        let state_names = sort! { self.state_names };
        let trans_names = sort! { self.trans_names };
        AutomataNames {
            name: self.name,
            state_names,
            trans_names,
        }
    }

    add_name! {state_names, add_state}
    add_name! {trans_names, add_transition}
}




#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn test_index_table() {
        let mut global_factory = GlobalIndexTableFactory::default();
        
        let mut net_factory = NetworkIndexTableFactory::new("a");
        net_factory.add_ev_name("b", 1);
        net_factory.add_ev_name("c", 0);

        global_factory.add_network(net_factory, 1);

        let mut net_factory = NetworkIndexTableFactory::new("A");
        net_factory.add_link_name("B", 1);
        net_factory.add_link_name("C", 0);

        let mut automata_factory = AutomataNamesFactory::new("AA");
        automata_factory.add_state("S0", 1);
        automata_factory.add_state("S1", 0);
        automata_factory.add_state("S3", 2);

        automata_factory.add_transition("T0", 0);
        automata_factory.add_transition("T2", 2);
        automata_factory.add_transition("T1", 1);

        net_factory.add_automata(automata_factory, 1);


        let mut automata_factory = AutomataNamesFactory::new("BB");
        automata_factory.add_state("St0", 1);
        automata_factory.add_state("St1", 0);
        automata_factory.add_state("St3", 2);

        automata_factory.add_transition("Tr0", 0);
        automata_factory.add_transition("Tr2", 2);
        automata_factory.add_transition("Tr1", 1);

        net_factory.add_automata(automata_factory, 0);


        global_factory.add_network(net_factory, 0);

        let index_table = global_factory.build();
        assert_eq!(index_table.networks.len(), 2);

        let net = index_table.get_network_table(0);
        assert_eq!(net.get_name(), "A");

        let net_names = net.get_network_names();
        assert_eq!(net_names.get_link_name(0), "C");
        assert_eq!(net_names.get_link_name(1), "B");

        let auto_names = net.get_automata_names(0);
        assert_eq!(auto_names.get_name(), "BB");

        assert_eq!(auto_names.get_state_name(0), "St1");
        assert_eq!(auto_names.get_state_name(1), "St0");
        assert_eq!(auto_names.get_state_name(2), "St3");

        assert_eq!(auto_names.get_transition_name(0), "Tr0");
        assert_eq!(auto_names.get_transition_name(1), "Tr1");
        assert_eq!(auto_names.get_transition_name(2), "Tr2");



        let auto_names = net.get_automata_names(1);
        assert_eq!(auto_names.get_name(), "AA");

        assert_eq!(auto_names.get_state_name(0), "S1");
        assert_eq!(auto_names.get_state_name(1), "S0");
        assert_eq!(auto_names.get_state_name(2), "S3");

        assert_eq!(auto_names.get_transition_name(0), "T0");
        assert_eq!(auto_names.get_transition_name(1), "T1");
        assert_eq!(auto_names.get_transition_name(2), "T2");


        let net = index_table.get_network_table(1);
        assert_eq!(net.get_name(), "a");

        let net_names = net.get_network_names();
        assert_eq!(net_names.get_ev_name(0), "c");
        assert_eq!(net_names.get_ev_name(1), "b");




    }

}







