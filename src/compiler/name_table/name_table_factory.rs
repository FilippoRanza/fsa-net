use super::name_table::*;
use super::request_table::convert_command;

use fsa_net_parser::syntax_tree::*;
use fsa_net_parser::Code;

pub fn build_name_table<'a>(code: &Code<'a>) -> Result<GlobalNameTable<'a>, NameError<'a>> {
    let name_table = GlobalNameTable::new();
    let name_table = code.iter().try_fold(name_table, |nt, curr| match curr {
        Block::Network(net) => collect_network(nt, net),
        Block::Request(req) => collect_request(nt, req),
    })?;

    name_table.validate()
}

fn collect_request<'a>(nt: GlobalNameTable<'a>, req: &Request<'a>) -> GlobalNameResult<'a> {
    let nt = nt.insert_request(req.name, req.get_location())?;
    let nt = req.list.iter().try_fold(nt, collect_command)?;
    Ok(nt.exit_request())
}

fn collect_command<'a>(nt: GlobalNameTable<'a>, cmd: &Command<'a>) -> GlobalNameResult<'a> {
    let req = convert_command(cmd);
    nt.add_request(req)
}

fn collect_network<'a>(nt: GlobalNameTable<'a>, net: &Network<'a>) -> GlobalNameResult<'a> {
    let nt = nt.declare_network(net.name, net.get_location())?;
    let nt = net.params.iter().try_fold(nt, collect_net_param)?;
    Ok(nt.exit_network())
}

fn collect_net_param<'a>(
    nt: GlobalNameTable<'a>,
    param: &NetworkParameterDecl<'a>,
) -> GlobalNameResult<'a> {
    let loc = param.get_location();
    match &param.param {
        NetworkParameter::Automata(automata) => collect_automata(nt, automata),
        NetworkParameter::Events(events) => events
            .iter()
            .try_fold(nt, |nt, ev| nt.declare_event(ev, loc)),
        NetworkParameter::ObserveLabels(labels) => labels
            .iter()
            .try_fold(nt, |nt, lbl| nt.declare_obs_label(lbl, loc)),
        NetworkParameter::RelevanceLabels(labels) => labels
            .iter()
            .try_fold(nt, |nt, lbl| nt.declare_rel_label(lbl, loc)),
        NetworkParameter::Link(link) => collect_link(nt, link),
    }
}

fn collect_link<'a>(nt: GlobalNameTable<'a>, link: &Link<'a>) -> GlobalNameResult<'a> {
    let loc = link.get_location();
    let nt = nt.declare_link(link.name, loc)?;

    let nt = nt.add_automata(link.source, loc)?;
    nt.add_automata(link.destination, loc)
}

fn collect_automata<'a>(nt: GlobalNameTable<'a>, automata: &Automata<'a>) -> GlobalNameResult<'a> {
    let nt = nt.declare_automata(automata.name, automata.get_location())?;
    let nt = automata
        .params
        .iter()
        .try_fold(nt, collect_automata_param)?;
    Ok(nt.exit_automata())
}

fn collect_automata_param<'a>(
    nt: GlobalNameTable<'a>,
    param: &AutomataParameterDecl<'a>,
) -> GlobalNameResult<'a> {
    let loc = param.get_location();
    match &param.param {
        AutomataParameter::StateDecl(state) => match state {
            StateDeclaration::Begin(state) => nt.declare_begin(state, loc),
            StateDeclaration::State(state) => nt.declare_state(state, loc),
        },
        AutomataParameter::Transition(trans) => collect_transition(nt, trans),
    }
}

fn collect_transition<'a>(
    nt: GlobalNameTable<'a>,
    trans: &TransitionDeclaration<'a>,
) -> GlobalNameResult<'a> {
    let loc = trans.get_location();
    let nt = nt.declare_transition(trans.name, loc)?;

    let nt = nt.add_state(trans.source, loc)?;
    let nt = nt.add_state(trans.destination, loc)?;

    let nt = if let Some(input_event) = &trans.input {
        collect_event(nt, input_event, loc)?
    } else {
        nt
    };

    let nt = if let Some(obs_label) = &trans.obs_label {
        nt.add_obs_label(obs_label, loc)?
    } else {
        nt
    };

    let nt = if let Some(rel_label) = &trans.rel_label {
        nt.add_rel_label(rel_label, loc)?
    } else {
        nt
    };

    if let Some(output_events) = &trans.output {
        output_events
            .iter()
            .try_fold(nt, |nt, ev| collect_event(nt, ev, loc))
    } else {
        Ok(nt)
    }
}

fn collect_event<'a>(
    nt: GlobalNameTable<'a>,
    event: &Event<'a>,
    loc: (usize, usize),
) -> GlobalNameResult<'a> {
    let nt = nt.add_event(&event.name, loc)?;
    nt.add_link(&event.link, loc)
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

        let err =
            build_name_table(&ast).expect_err("`duplicate-begin` should contain semantic errors");
        match err {
            NameError::BeginStateError(BeginStateError::MultipleBeginState(states)) => {
                assert_eq!(states.len(), 2);
                assert!(states.contains(&"s0"));
                assert!(states.contains(&"s1"));
            }
            err => panic!(
                "Expected BeginStateError(MultipleBeginState), found {:?}",
                err
            ),
        }
    }

    #[test]
    fn test_missing_begin_state() {
        let code = load_code_from_file("missing-begin");
        let ast = parse(&code).expect("`missing-begin` should be syntactically correct");

        let err =
            build_name_table(&ast).expect_err("`missing-begin` should contain semantic errors");
        match err {
            NameError::BeginStateError(BeginStateError::NoBeginState) => {}
            err => panic!("Expected BeginStateError(NoBeginState), found {:?}", err),
        }
    }

    #[test]
    fn test_ridefined_names() {
        let test_params = [
            (
                "ridefined-name",
                "s1",
                NameClass::State,
                NameClass::ObsLabel,
            ),
            (
                "ridefined-automata",
                "A",
                NameClass::Automata,
                NameClass::Automata,
            ),
            (
                "ridefined_network",
                "Name",
                NameClass::Network,
                NameClass::Network,
            ),
            (
                "ridefined-event",
                "A",
                NameClass::Event,
                NameClass::Automata,
            ),
            (
                "ridefined-automata2",
                "A",
                NameClass::Automata,
                NameClass::Event,
            ),
        ];
        for (file, name, orig, ridef) in &test_params {
            run_name_ridefinition_test(file, name, orig, ridef);
        }
    }

    #[test]
    fn test_undefined_network() {
        let code = load_code_from_file("undefined-network");
        let ast = parse(&code).expect("`undefined-network` should be syntactically correct");

        let err =
            build_name_table(&ast).expect_err("`undefined-network` contains `MissingNetwork`");
        match err {
            NameError::UndefinedNetwork(err) => {
                assert_eq!(err.names.len(), 1);
                assert_eq!(err.names[0].0, "MissingNetwork");
            }
            _ => panic!("expect UndefinedNetwork, found: {:?}", err),
        }
    }

    #[test]
    fn test_mismatch_name_type() {
        let code = load_code_from_file("mismatch-name-type");
        let ast = parse(&code).expect("`mistmatch-name-type` should be syntactically correct");

        let err =
            build_name_table(&ast).expect_err("`mistmatch-name-type` contains a mistmatch name s0");
        match err {
            NameError::MismatchedType(err) => {
                assert_eq!(err.name, "s0");
                assert_eq!(err.orig, NameClass::State);
                assert_eq!(err.curr, NameClass::ObsLabel);
            }
            _ => panic!("expect MistmatchedType, found, {:?}", err),
        }
    }

    #[test]
    fn test_undefined_names() {
        let test_params = [("undefined-automata", "B")];
        for (file, name) in &test_params {
            run_undefined_name_test(file, name);
        }
    }

    fn run_undefined_name_test(file: &str, name: &str) {
        let code = load_code_from_file(file);
        let expect_msg = format!("`{}` should be syntactically correct", file);
        let ast = parse(&code).expect(&expect_msg);

        let expect_msg = format!(
            "in file `{}` a semantic error is expected: name {} is undefined",
            file, name
        );
        let err = build_name_table(&ast).expect_err(&expect_msg);
        match err {
            NameError::UndefinedNameError(err) => {
                assert_eq!(err.name, name);
            }
            err => panic!("Expected UndefinedNameError, found {:?}", err),
        }
    }

    #[test]
    fn test_undefined_label() {
        let code = load_code_from_file("undefined_label");
        let ast = parse(&code).expect("`undefined_label` should be syntactically correct");

        let err = build_name_table(&ast).expect_err("rel label `r4` is not defined");

        match err {
            NameError::UndefinedLabel(err) => {
                assert_eq!(err.name, "r4");
                assert_eq!(err.class, NameClass::RelLabel);
            }
            _ => panic!("Expected Undefined Label, found {:?}", err),
        }
    }

    fn run_name_ridefinition_test(file: &str, name: &str, orig: &NameClass, ridef: &NameClass) {
        let code = load_code_from_file(file);
        let expect_msg = format!("`{}` should be syntactically correct", file);
        let ast = parse(&code).expect(&expect_msg);

        let expect_msg = format!(
            "in file `{}` a semantic error is expected: name {} is defined multiple times",
            file, name
        );
        let err = build_name_table(&ast).expect_err(&expect_msg);
        match err {
            NameError::NameRidefinitionError(err) => {
                assert_eq!(err.name, name);
                assert_eq!(err.orig_class, *orig);
                assert_eq!(err.ridef_class, *ridef);
            }
            err => panic!("Expected NameRidefinitionError, found {:?}", err),
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
