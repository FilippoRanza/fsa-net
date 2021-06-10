use std::fs::File;
use std::path::PathBuf;
use std::io::Read;



pub fn load_code_from_file(name: &str) -> String {
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


