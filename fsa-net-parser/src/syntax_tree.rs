use item_location_derive::{add_location, DefaultBuilder};

pub enum Block<'a> {
    Network(Network<'a>),
    Request(Request<'a>),
}

#[add_location]
#[derive(DefaultBuilder)]
pub struct Network<'a> {
    pub name: &'a str,
    pub params: Vec<NetworkParameter<'a>>,
}

pub enum NetworkParameter<'a> {
    Automata(Automata<'a>),
    Link(Link<'a>),
    Events(Vec<&'a str>),
    ObserveLabels(Vec<&'a str>),
    RelevanceLabels(Vec<&'a str>),
}

#[add_location]
#[derive(DefaultBuilder)]
pub struct Automata<'a> {
    pub name: &'a str,
    pub params: Vec<AutomataParameter<'a>>,
}

pub enum AutomataParameter<'a> {
    StateDecl(StateDeclaration<'a>),
    Transition(TransitionDeclaration<'a>),
}

pub enum StateDeclaration<'a> {
    State(&'a str),
    Begin(&'a str),
}

#[add_location]
#[derive(DefaultBuilder)]
pub struct TransitionDeclaration<'a> {
    pub name: &'a str,
    pub source: &'a str,
    pub destination: &'a str,
    pub input: Option<Event<'a>>,
    pub output: Option<Vec<Event<'a>>>,
    pub rel_label: Option<&'a str>,
    pub obs_label: Option<&'a str>,
}

impl<'a> TransitionDeclaration<'a> {
    pub fn simple_decl(name: &'a str, source: &'a str, destination: &'a str) -> Self {
        Self::new(name, source, destination, None, None, None, None)
    }
}

#[derive(Debug)]
pub struct TransitionFactoryError {
    begin: usize,
    end: usize,
    error_type: TransitionFactoryErrorType,
}

impl TransitionFactoryError {
    fn new_duplicated_key(begin: usize, end: usize) -> Self {
        Self {
            begin,
            end,
            error_type: TransitionFactoryErrorType::DuplicatedKey,
        }
    }

    fn new_missing_src_dst(begin: usize, end: usize) -> Self {
        Self {
            begin,
            end,
            error_type: TransitionFactoryErrorType::MissingSourceOrDestination,
        }
    }
}

#[derive(Debug)]
pub enum TransitionFactoryErrorType {
    MissingSourceOrDestination,
    DuplicatedKey,
}

#[add_location]
#[derive(Default)]
pub struct TransitionParameterFactory<T> {
    param: Option<T>,
}

impl<T> TransitionParameterFactory<T>
where
    T: Default,
{
    pub fn set_value(
        mut self,
        param: T,
        loc: (usize, usize),
    ) -> Result<Self, TransitionFactoryError> {
        if let Some(_) = self.param {
            let (begin, end) = loc;
            Err(TransitionFactoryError::new_duplicated_key(begin, end))
        } else {
            self.param = Some(param);
            Ok(self)
        }
    }

    pub fn is_set(&self) -> bool {
        self.param.is_some()
    }

    pub fn get_param(self) -> Option<T> {
        self.param
    }

    pub fn unwrap(self) -> T {
        self.param.unwrap()
    }
}

#[derive(Default)]
pub struct ComplexTransactionFactory<'a> {
    src: TransitionParameterFactory<&'a str>,
    dst: TransitionParameterFactory<&'a str>,
    input: TransitionParameterFactory<Event<'a>>,
    output: TransitionParameterFactory<Vec<Event<'a>>>,
    rel: TransitionParameterFactory<&'a str>,
    obs: TransitionParameterFactory<&'a str>,
    begin: usize,
    end: usize,
}

impl<'a> ComplexTransactionFactory<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_location(mut self, begin: usize, end: usize) -> Self {
        self.begin = begin;
        self.end = end;
        self
    }

    pub fn build_transition(
        self,
        name: &'a str,
    ) -> Result<TransitionDeclaration<'a>, TransitionFactoryError> {
        if self.src.is_set() && self.dst.is_set() {
            let output = TransitionDeclaration::new(
                name,
                self.src.unwrap(),
                self.dst.unwrap(),
                self.input.get_param(),
                self.output.get_param(),
                self.rel.get_param(),
                self.obs.get_param(),
            );
            Ok(output)
        } else {
            Err(TransitionFactoryError::new_missing_src_dst(
                self.begin, self.end,
            ))
        }
    }

    pub fn set_parameter(mut self, key: TransitionKey<'a>) -> Result<Self, TransitionFactoryError> {
        let loc = key.get_location();
        match key.key {
            TransitionKeys::Src(param) => self.src = self.src.set_value(param, loc)?,
            TransitionKeys::Dst(param) => self.dst = self.dst.set_value(param, loc)?,
            TransitionKeys::Input(param) => self.input = self.input.set_value(param, loc)?,
            TransitionKeys::Output(param) => self.output = self.output.set_value(param, loc)?,
            TransitionKeys::Rel(param) => self.rel = self.rel.set_value(param, loc)?,
            TransitionKeys::Obs(param) => self.obs = self.obs.set_value(param, loc)?,
        }
        Ok(self)
    }
}

#[add_location]
#[derive(DefaultBuilder)]
pub struct TransitionKey<'a> {
    key: TransitionKeys<'a>,
}

pub enum TransitionKeys<'a> {
    Src(&'a str),
    Dst(&'a str),
    Input(Event<'a>),
    Output(Vec<Event<'a>>),
    Rel(&'a str),
    Obs(&'a str),
}

pub fn remove_quotes<'a>(quoted_str: &'a str) -> &'a str {
    let end = quoted_str.len() - 1;
    &quoted_str[1..end]
}

#[add_location]
#[derive(DefaultBuilder, Default)]
pub struct Event<'a> {
    pub name: &'a str,
    pub link: &'a str,
}

#[add_location]
#[derive(DefaultBuilder)]
pub struct Link<'a> {
    pub name: &'a str,
    pub source: &'a str,
    pub destination: &'a str,
}

#[add_location]
#[derive(DefaultBuilder)]
pub struct Request<'a> {
    pub name: &'a str,
    pub list: Vec<Command<'a>>,
}

pub enum Command<'a> {
    Space,
    Linspace(LinspaceCommand<'a>),
    Diagnosis(DiagnosisCommand<'a>),
}

#[add_location]
#[derive(DefaultBuilder)]
pub struct LinspaceCommand<'a> {
    name_list: Vec<&'a str>,
}

#[add_location]
#[derive(DefaultBuilder)]
pub struct DiagnosisCommand<'a> {
    name_list: Vec<&'a str>,
}
