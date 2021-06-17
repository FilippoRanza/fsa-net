#[derive(Debug)]
pub struct Requests {
    pub commands: Vec<Command>,
}

impl Requests {
    pub fn new(commands: Vec<Command>) -> Self {
        Self { commands }
    }
}

#[derive(Debug)]
pub enum Command {
    FullSpace,
    Linspace((Vec<usize>, Option<usize>)),
    Diagnosis(DiagnosisCommand),
}

#[derive(Debug)]
pub enum DiagnosisCommand {
    Fresh(Vec<usize>),
    Load(usize),
}
