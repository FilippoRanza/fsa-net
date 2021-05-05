use super::automata_connection;
use super::error;
use super::link_connection;
use super::name_table;

use fsa_net_parser::Code;

pub fn compile<'a>(code: &'a Code<'a>) -> Result<(), error::CompileError<'a>> {
    let _ = name_table::build_name_table(code)?;
    automata_connection::check_connection(code)?;
    link_connection::link_check(code)?;
    Ok(())
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
}
