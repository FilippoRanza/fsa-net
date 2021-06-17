use super::Loc;
use fsa_net_parser::syntax_tree::{Command, CommandDecl, DiagnosisCommand};

use indexmap::IndexMap;

/*
    Collect all the user requests
    for the specified network
*/
#[derive(Debug)]
pub struct RequestTable<'a> {
    pub loc: Loc,
    requests: Vec<Request<'a>>,
    files: IndexMap<&'a str, usize>,
}

impl<'a> RequestTable<'a> {
    pub fn new(loc: Loc) -> Self {
        Self {
            loc,
            requests: vec![],
            files: IndexMap::new(),
        }
    }

    pub fn get_location(&self) -> Loc {
        self.loc
    }

    pub fn add_request(&mut self, req: Request<'a>) {
        match req.1 {
            RequestType::Diagnosis(DiagnosisRequest::Load(file)) => self.insert_file(file),
            RequestType::Linspace((_, Some(file))) => self.insert_file(file),
            _ => {}
        };
        self.requests.push(req)
    }

    fn insert_file(&mut self, name: &'a str) {
        if !self.files.contains_key(name) {
            let index = self.files.len();
            self.files.insert(name, index);
        }
    }

    pub fn get_linspace_labels(&self) -> impl Iterator<Item = &Vec<&'a str>> {
        self.requests.iter().filter_map(|(_, cmd)| {
            if let RequestType::Linspace((lbls, _)) = cmd {
                Some(lbls)
            } else {
                None
            }
        })
    }

    pub fn get_diagnosis_labels(&self) -> impl Iterator<Item = &Vec<&'a str>> {
        self.requests
            .iter()
            .filter_map(|(_, cmd)| {
                if let RequestType::Diagnosis(lbls) = cmd {
                    Some(lbls)
                } else {
                    None
                }
            })
            .filter_map(|req| match req {
                DiagnosisRequest::Fresh(lbls) => Some(lbls),
                DiagnosisRequest::Load(_) => None,
            })
    }

    pub fn get_file_index(&self, file: &str) -> usize {
        *self.files.get(file).unwrap()
    }

    pub fn get_files(self) -> impl Iterator<Item = &'a str> {
        self.files.into_iter().map(|(k, _)| k)
    }
}

pub fn convert_command<'a>(cmd: &CommandDecl<'a>) -> Request<'a> {
    match &cmd.cmd {
        Command::Space => (cmd.get_location(), RequestType::Space),
        Command::Linspace(cmd) => (
            cmd.get_location(),
            RequestType::Linspace((weak_copy(&cmd.name_list), cmd.save_file)),
        ),
        Command::Diagnosis(cmd) => {
            let (loc, cmd) = convert_diagnosis(cmd);
            (loc, RequestType::Diagnosis(cmd))
        }
    }
}

fn convert_diagnosis<'a>(cmd: &DiagnosisCommand<'a>) -> (Loc, DiagnosisRequest<'a>) {
    match cmd {
        DiagnosisCommand::Fresh(fresh) => (
            fresh.get_location(),
            DiagnosisRequest::Fresh(weak_copy(&fresh.name_list)),
        ),
        DiagnosisCommand::Load(load) => (load.get_location(), DiagnosisRequest::Load(load.file)),
    }
}

fn weak_copy<'a>(names: &[&'a str]) -> Vec<&'a str> {
    names.iter().map(|n| *n).collect()
}

pub type Request<'a> = (Loc, RequestType<'a>);

#[derive(Debug)]
pub enum RequestType<'a> {
    Space,
    Linspace((Vec<&'a str>, Option<&'a str>)),
    Diagnosis(DiagnosisRequest<'a>),
}

#[derive(Debug)]
pub enum DiagnosisRequest<'a> {
    Fresh(Vec<&'a str>),
    Load(&'a str),
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn test_add_files() {
        let mut req_table = RequestTable::new((0, 0));

        let names = ["file_a", "file_b", "file_c", "file_b", "file_a"];

        for name in &names {
            let req = ((0, 1), RequestType::Diagnosis(DiagnosisRequest::Load(name)));
            req_table.add_request(req)
        }

        let result = [("file_a", 0), ("file_b", 1), ("file_c", 2)];

        let files_index = req_table.files;
        assert_eq!(files_index.len(), 3);
        for (name, index) in &result {
            let i = files_index.get(name).unwrap();
            assert_eq!(i, index);
        }
    }
}
