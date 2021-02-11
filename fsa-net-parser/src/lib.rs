#[macro_use]
extern crate lalrpop_util;
lalrpop_mod!(fsa_net_lang);

mod syntax_tree;

#[cfg(test)]
mod tests {

    use std::path::PathBuf;
    use std::fs::File;
    use std::io::Read;
    use super::*;

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
        Ok(result.is_ok())
    }


    fn load_file(path: &PathBuf) -> Result<String, std::io::Error> {
        let mut file = File::open(path)?;
        let mut code = String::new();
        file.read_to_string(&mut code)?;
        Ok(code)
    }

}
