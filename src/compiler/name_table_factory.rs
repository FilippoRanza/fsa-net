use super::name_table::*;

use fsa_net_parser::syntax_tree::*;
use fsa_net_parser::Code;

pub fn build_name_table<'a>(code: &Code<'a>) -> Result<GlobalNameTable<'a>, NameError<'a>> {
    let name_table = GlobalNameTable::new();
    let name_table = code.iter().try_fold(name_table, |nt, curr| match curr {
        Block::Network(net) => collect_network(nt, net),
        Block::Request(_req) => Ok(nt),
    })?;

    name_table.validate()
}

fn collect_network<'a>(nt: GlobalNameTable<'a>, net: &Network<'a>) -> GlobalNameResult<'a> {
    let nt = nt.add_network(net.name, net.get_location())?;
    let nt = net.params.iter().try_fold(nt, collect_net_param)?;
    Ok(nt.exit_network())
}

fn collect_net_param<'a>(
    nt: GlobalNameTable<'a>,
    param: &NetworkParameter<'a>,
) -> GlobalNameResult<'a> {
    match param {
        NetworkParameter::Automata(automata) => collect_automata(nt, automata),
        NetworkParameter::Events(events) => events
            .iter()
            .try_fold(nt, |nt, ev| nt.add_event(ev, (0, 0))),
        NetworkParameter::ObserveLabels(labels) => labels
            .iter()
            .try_fold(nt, |nt, lbl| nt.add_rel_label(lbl, (0, 0))),
        NetworkParameter::RelevanceLabels(labels) => labels
            .iter()
            .try_fold(nt, |nt, lbl| nt.add_obs_label(lbl, (0, 0))),
        NetworkParameter::Link(link) => nt.add_link(link.name, link.get_location()),
    }
}

fn collect_automata<'a>(nt: GlobalNameTable<'a>, automata: &Automata<'a>) -> GlobalNameResult<'a> {
    let nt = nt.add_automata(automata.name, automata.get_location())?;
    let nt = automata
        .params
        .iter()
        .try_fold(nt, collect_automata_param)?;
    Ok(nt.exit_automata())
}

fn collect_automata_param<'a>(
    nt: GlobalNameTable<'a>,
    param: &AutomataParameter<'a>,
) -> GlobalNameResult<'a> {
    match param {
        AutomataParameter::StateDecl(state) => match state {
            StateDeclaration::Begin(state) => nt.add_begin(state, (0, 0)),
            StateDeclaration::State(state) => nt.add_state(state, (0, 0)),
        },
        AutomataParameter::Transition(trans) => nt.add_transition(trans.name, trans.get_location()),
    }
}

#[cfg(test)]
mod test {

    use super::*;
    use fsa_net_parser::parse;

    use std::fs::File;
    use std::io::Read;
    use std::path::PathBuf;

    #[test]
    fn test_correct_file_code() {
        let code = load_code_from_file("correct-code");
        let ast = parse(&code).expect("`correct-code` should be syntactically correct");

        let _ = build_name_table(&ast).unwrap();
    }

    #[test]
    fn test_ridefined_begin_state() {
        let code = load_code_from_file("duplicate-begin");
        let ast = parse(&code).expect("`duplicate-begin` should be syntactically correct");

        let err = build_name_table(&ast).expect_err("`duplicate-begin` should contain semantic errors");
        match err {
            NameError::BeginStateError(BeginStateError::MultipleBeginState(states)) => {
                assert_eq!(states.len(), 2);
                assert!(states.contains(&"s0"));
                assert!(states.contains(&"s1"));
            },
            err => panic!("Expected BeginStateError(MultipleBeginState), found {:?}", err)
        }
    }

    #[test]
    fn test_missing_begin_state() {
        let code = load_code_from_file("missing-begin");
        let ast = parse(&code).expect("`missing-begin` should be syntactically correct");

        let err = build_name_table(&ast).expect_err("`missing-begin` should contain semantic errors");
        match err {
            NameError::BeginStateError(BeginStateError::NoBeginState) => {},
            err => panic!("Expected BeginStateError(NoBeginState), found {:?}", err)
        }
    }


    fn load_code_from_file(name: &str) -> String {
        let file_path = if name.ends_with(".fnl") {
            PathBuf::from("fnl-test-code").join(name)
        } else {
            let name = format!("{}.fnl", name);
            PathBuf::from("fnl-test-code").join(name)
        };

        let mut buff = String::new();
        let mut file = File::open(&file_path).expect(&format!("{:?} should exist", &file_path));
        file.read_to_string(&mut buff)
            .expect(&format!("{:?} should be read", &file_path));
        buff
    }
}
