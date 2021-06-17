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

pub fn save_str_to_file<'a, P>(s: &str, file: P) -> io::Result<()>
where
    P: AsRef<path::Path>,
{
    let file = fs::File::create(file)?;
    dump_code(s, file)
}

pub fn load_str_from_file<'a, P>(file: P) -> io::Result<String>
where
    P: AsRef<path::Path>,
{
    let file = fs::File::open(file)?;
    load_code(file)
}

fn load_code(mut reader: impl io::Read) -> io::Result<String> {
    let mut buff = String::new();
    reader.read_to_string(&mut buff)?;
    Ok(buff)
}

fn dump_code<D>(data: D, mut writer: impl io::Write) -> io::Result<()>
where
    D: AsRef<[u8]>,
{
    writer.write_all(data.as_ref())
}
