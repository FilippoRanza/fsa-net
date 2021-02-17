use std::fs;
use std::io;
use std::path;

pub fn get_fsa_code(file_path: &Option<path::PathBuf>) -> io::Result<String> {
    if let Some(file_path) = file_path {
        let file = fs::File::open(file_path)?;
        load_code(file)
    } else {
        let stdin = io::stdin();
        load_code(stdin)
    }
}

fn load_code(mut reader: impl io::Read) -> io::Result<String> {
    let mut buff = String::new();
    reader.read_to_string(&mut buff)?;
    Ok(buff)
}
