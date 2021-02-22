use super::name_table::*;

use fsa_net_parser::syntax_tree::*;
use fsa_net_parser::Code;

pub fn build_name_table<'a>(code: &Code<'a>) -> Result<GlobalNameTable<'a>, NameError<'a>> {
    let name_table = GlobalNameTable::new();
    let name_table = code.iter().try_fold(name_table, |nt, curr| match curr {
        Block::Network(net) => collect_network(nt, net),
        Block::Request(req) => collect_request(nt, req),
    })?;

    let name_table = name_table.validate()?;

    Ok(name_table)
}

fn collect_network<'a>(
    name_table: GlobalNameTable<'a>,
    network: &Network<'a>,
) -> Result<GlobalNameTable<'a>, NameError<'a>> {
    let network_table = NetworkNameTable::new(network.name);

    let network_table = collect_network_parameters(&network.params, network_table)?;
    let name_table =
        name_table.insert_network(network.name, network.get_location(), network_table)?;

    Ok(name_table)
}

fn collect_network_parameters<'a>(
    param_list: &[NetworkParameter<'a>],
    name_table: NetworkNameTable<'a>,
) -> NetworkNameResult<'a> {
    param_list
        .iter()
        .try_fold(name_table, collect_network_parameter)
}

fn collect_network_parameter<'a>(
    nt: NetworkNameTable<'a>,
    net_param: &NetworkParameter<'a>,
) -> NetworkNameResult<'a> {
    match net_param {
        NetworkParameter::Automata(automata) => {
            nt.insert_automata(automata.name, automata.get_location())
        }
        NetworkParameter::Events(events) => events
            .iter()
            .try_fold(nt, |nt, ev| nt.insert_event(ev, (0, 0))),
        NetworkParameter::Link(link) => nt.insert_link(link.name, link.get_location()),
        NetworkParameter::ObserveLabels(labels) => labels
            .iter()
            .try_fold(nt, |nt, obs| nt.insert_obs_label(obs, (0, 0))),
        NetworkParameter::RelevanceLabels(labels) => labels
            .iter()
            .try_fold(nt, |nt, rel| nt.insert_rel_label(rel, (0, 0))),
    }
}

fn collect_request<'a>(
    nt: GlobalNameTable<'a>,
    req: &Request<'a>,
) -> Result<GlobalNameTable<'a>, NameError<'a>> {
    let nt = nt.insert_request(req.name, req.get_location())?;
    Ok(nt)
}

#[cfg(test)]
mod test {

    use super::super::name_table::{GlobalClassName, NameError};
    use super::*;
    use fsa_net_parser::parse;
    use std::fs::File;
    use std::io::Read;
    use std::path::PathBuf;

    extern crate lazy_static;

    lazy_static::lazy_static! {
        static ref BASE_PATH: PathBuf = PathBuf::from("fnl-test-code");
    }

    #[test]
    fn test_correct_global_name_check() {
        let code = load_file_by_name("correct-code");
        let ast = parse(&code).expect("`correct-code` should be syntactically correct");

        let _ = build_name_table(&ast).expect("`correct-code` should be semantically correct");
    }

    #[test]
    fn test_undefined_network() {
        let code = load_file_by_name("undefined-network");
        let ast = parse(&code).expect("`undefined-network` should be syntactically correct");

        let err =
            build_name_table(&ast).expect_err("`undefined-network` contains an undefined network");

        match err {
            NameError::UndefinedNetwork(undef_net) => {
                assert_eq!(undef_net.names.len(), 1);
                let err = undef_net.names[0];
                assert_eq!(err.0, "MissingNetwork");
            }
            err => panic!("expected UndefinedNetwork, found {:?}", err),
        }
    }

    #[test]
    fn test_ridefined_network() {
        let code = load_file_by_name("ridefined_network");
        let ast = parse(&code).expect("`ridefined_network`should be syntactically correct");

        let err =
            build_name_table(&ast).expect_err("`ridefined_network` containts a ridefined network");

        match err {
            NameError::Global(err) => {
                assert_eq!(err.name, "Name");
                assert_eq!(err.class, GlobalClassName::Network);
            }
            err => panic!("expected RidefinedNetwork, found {:?}", err),
        }
    }

    #[test]
    fn test_ridefined_automata() {
        let code = load_file_by_name("ridefined-automata");
        let ast = parse(&code).expect("`ridefined-automata` should be syntactically correct");

        let err = build_name_table(&ast).expect_err("`ridefined-automata` contains a redefined automata");

        match err {
            NameError::NetworkNameError(err) => {
                assert_eq!(err.name, "A");
                assert_eq!(err.orig_class, NetworkNameClass::Automata);
                assert_eq!(err.ridef_class, NetworkNameClass::Automata);
            },
            err => panic!("expected NetworkNameError, found {:?}", err)
        }
    }

    fn load_file_by_name(name: &str) -> String {
        let path = if name.ends_with(".fnl") {
            BASE_PATH.join(name)
        } else {
            let name = format!("{}.{}", name, "fnl");
            BASE_PATH.join(name)
        };

        let mut buff = String::new();
        let mut file = File::open(&path).expect(&format!("Can't open test code: {:?}", path));
        file.read_to_string(&mut buff)
            .expect(&format!("Can't read test code: {:?}", path));

        buff
    }
}
