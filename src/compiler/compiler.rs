use super::error;
use super::automata_connection;
use super::name_table;

use fsa_net_parser::Code;

pub fn compile<'a>(code: &Code<'a>) -> Result<(), error::CompileError<'a>> {
    let _ = name_table::build_name_table(code)?;
    automata_connection::check_connection(code)?;

    Ok(())
}


#[cfg(test)]
mod test {

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
            err => panic!("Expected GraphError, found: {:?}", err)
        }
    }


}
