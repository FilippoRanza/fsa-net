use super::Loc;
use fsa_net_parser::syntax_tree::Command;

#[derive(Debug)]
pub struct RequestTable<'a> {
    loc: Loc,
    requests: Vec<Request<'a>>,
}

impl<'a> RequestTable<'a> {
    pub fn new(loc: Loc) -> Self {
        Self {
            loc,
            requests: vec![],
        }
    }

    pub fn get_location(&self) -> Loc {
        self.loc
    }

    pub fn add_request(&mut self, req: Request<'a>) {
        self.requests.push(req)
    }

    pub fn get_linspace_labels(&self) -> impl Iterator<Item = &Vec<&'a str>> {
        self.requests.iter().filter_map(|(_, cmd)| {
            if let RequestType::Linspace(lbls) = cmd {
                Some(lbls)
            } else {
                None
            }
        })
    }

    pub fn get_diagnosis_labels(&self) -> impl Iterator<Item = &Vec<&'a str>> {
        self.requests.iter().filter_map(|(_, cmd)| {
            if let RequestType::Diagnosis(lbls) = cmd {
                Some(lbls)
            } else {
                None
            }
        })
    }
}

pub fn convert_command<'a>(cmd: &Command<'a>) -> Request<'a> {
    match cmd {
        Command::Space => ((0, 0), RequestType::Space),
        Command::Linspace(cmd) => (
            cmd.get_location(),
            RequestType::Linspace(weak_copy(&cmd.name_list)),
        ),
        Command::Diagnosis(cmd) => (
            cmd.get_location(),
            RequestType::Diagnosis(weak_copy(&cmd.name_list)),
        ),
    }
}

fn weak_copy<'a>(names: &[&'a str]) -> Vec<&'a str> {
    names.iter().map(|n| *n).collect()
}

pub type Request<'a> = (Loc, RequestType<'a>);

#[derive(Debug)]
pub enum RequestType<'a> {
    Space,
    Linspace(Vec<&'a str>),
    Diagnosis(Vec<&'a str>),
}
