#[macro_use]
extern crate lalrpop_util;
lalrpop_mod!(fsa_net_lang);

pub mod syntax_tree;



pub type Code<'a> = Vec<syntax_tree::Block<'a>>;
pub type SyntaxError<'a> = lalrpop_util::ParseError<usize, lalrpop_util::lexer::Token<'a>, syntax_tree::TransitionFactoryError>;

pub fn parse<'a>(code: &'a str) -> Result<Code<'a>, SyntaxError<'a>>{

    let parser = fsa_net_lang::FsacodeParser::new();
    parser.parse(code)
}



#[cfg(test)]
mod tests {

    use super::*;
    use std::fs::File;
    use std::io::Read;
    use std::path::PathBuf;

    #[test]
    fn test_syntax() {
        let base_dir = PathBuf::from("fnl-test-code");
        assert!(base_dir.is_dir());
        for entry in base_dir.read_dir().expect("Cannot read fnl-test-code") {
            let file = entry.unwrap().path();
            let is_ok = try_syntax(&file).unwrap();
            assert!(is_ok);
        }
    }

    fn try_syntax(file: &PathBuf) -> Result<bool, std::io::Error> {
        let code = load_file(file)?;
        let parser = fsa_net_lang::FsacodeParser::new();
        let result = parser.parse(&code);
        result.unwrap();
        Ok(true)
    }

    fn load_file(path: &PathBuf) -> Result<String, std::io::Error> {
        let mut file = File::open(path)?;
        let mut code = String::new();
        file.read_to_string(&mut code)?;
        Ok(code)
    }
}
