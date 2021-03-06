use super::automata_connection;
use super::error;
use super::link_connection;
use super::name_table;
use super::net_compiler;
use super::CompileResult;

use fsa_net_parser::Code;

pub fn compile<'a>(code: &'a Code<'a>) -> Result<CompileResult, error::CompileError<'a>> {
    let table = name_table::build_name_table(code)?;
    automata_connection::check_connection(code)?;
    link_connection::link_check(code)?;
    let comp_res = net_compiler::compile_networks(code, &table);

    let output = CompileResult {
        compile_network: comp_res,
        index_table: table.get_index_table(),
    };

    Ok(output)
}

#[cfg(test)]
mod test {

    use super::super::link_connection::LinkError;
    use super::*;

    use fsa_net_parser::parse;
    use test_utils::load_code_from_file;

    #[test]
    fn test_check_connected_automata() {
        let code = load_code_from_file("correct-code");
        let ast = parse(&code).expect("`correct-coude` should be syntactically correcy");
        let res = compile(&ast);
        assert!(res.is_ok());
    }

    #[test]
    fn test_check_unconnected_automata() {
        let code = load_code_from_file("unconnected-automata");
        let ast = parse(&code).expect("`unconnected-automata` should be syntactically correcy");
        let res = compile(&ast);
        let err = res.unwrap_err();
        match err {
            error::CompileError::GraphError(err) => assert_eq!(err, vec!["a4"]),
            err => panic!("Expected GraphError, found: {:?}", err),
        }
    }

    #[test]
    fn test_non_input_link() {
        let code = load_code_from_file("link_not_in_input.fnl");
        let ast = parse(&code).expect("`link_not_in_input.fnl` should be syntactically correcy");
        let res = compile(&ast);
        let err = res.unwrap_err();
        match err {
            error::CompileError::LinkError(err) => match err {
                LinkError::NotInput(err) => {
                    assert_eq!(err.automata, "A");
                    assert_eq!(err.link, "L1");
                }
                _ => panic!(),
            },
            err => panic!("Expected GraphError, found: {:?}", err),
        }
    }
    #[test]
    fn test_multiple_link_use() {
        let code = load_code_from_file("multiple_link_usage.fnl");
        let ast = parse(&code).expect("`multiple_link_usage.fnl` should be syntactically correcy");
        let res = compile(&ast);
        let err = res.unwrap_err();
        match err {
            error::CompileError::LinkError(err) => match err {
                LinkError::MultipleLinkUse(err) => {
                    assert_eq!(err.len(), 1);
                    let err = &err[0];
                    assert_eq!(err.automata, "B");
                    assert_eq!(err.link, "L2");
                    assert_eq!(err.count, 2);
                }
                _ => panic!(),
            },
            err => panic!("Expected GraphError, found: {:?}", err),
        }
    }

    #[test]
    fn test_compile() {
        let src_code = load_code_from_file("simple-network");
        let code = parse(&src_code).expect("`simple-network` should be syntactically correct");
        let comp_res = compile(&code).expect("`simple-network` should be semantically correct");
        assert_eq!(comp_res.compile_network.len(), 1);
    }

    #[test]
    fn test_index_table_build() {
        let src_code = load_code_from_file("simple-network");
        let code = parse(&src_code).expect("`simple-network` should be syntactically correct");
        let comp_res = compile(&code).expect("`simple-network` should be semantically correct");

        let index_table = comp_res.index_table;

        let net_index = index_table.get_network_table(0);
        assert_eq!(net_index.get_name(), "TestNetwork");

        let net_names_index = net_index.get_network_names();
        assert_eq!(net_names_index.get_ev_name(0), "e2");
        assert_eq!(net_names_index.get_ev_name(1), "e3");

        assert_eq!(net_names_index.get_link_name(0), "L2");
        assert_eq!(net_names_index.get_link_name(1), "L3");

        assert_eq!(net_names_index.get_obs_name(0), "o2");
        assert_eq!(net_names_index.get_obs_name(1), "o3");

        assert_eq!(net_names_index.get_rel_name(0), "r");
        assert_eq!(net_names_index.get_rel_name(1), "f");

        let automata_index = net_index.get_automata_names(0);
        assert_eq!(automata_index.get_name(), "TestA");

        assert_eq!(automata_index.get_state_name(0), "b");
        assert_eq!(automata_index.get_state_name(1), "a");

        assert_eq!(automata_index.get_transition_name(0), "ta");
        assert_eq!(automata_index.get_transition_name(1), "tb");

        let automata_index = net_index.get_automata_names(1);
        assert_eq!(automata_index.get_name(), "TestB");

        assert_eq!(automata_index.get_state_name(0), "a");
        assert_eq!(automata_index.get_state_name(1), "b");

        assert_eq!(automata_index.get_transition_name(0), "ta");
        assert_eq!(automata_index.get_transition_name(1), "tb");
        assert_eq!(automata_index.get_transition_name(2), "tc");
    }
}
