pub enum Block<'a> {
    Network(Network<'a>),
    Request(Request<'a>),
}

pub struct Network<'a> {
    name: &'a str,
    params: Vec<NetworkParameter<'a>>,
}

impl<'a> Network<'a> {
    pub fn new(name: &'a str, params: Vec<NetworkParameter<'a>>) -> Self {
        Self { name, params }
    }
}

pub enum NetworkParameter<'a> {
    Automata(Automata<'a>),
    Link(Link<'a>),
}

pub struct Automata<'a> {
    name: &'a str,
    params: Vec<AutomataParameter<'a>>,
}

impl<'a> Automata<'a> {
    pub fn new(name: &'a str, params: Vec<AutomataParameter<'a>>) -> Self {
        Self { name, params }
    }
}

pub enum AutomataParameter<'a> {
    StateDecl(StateDeclaration<'a>),
    Transition(TransitionDeclaration<'a>),
}

pub enum StateDeclaration<'a> {
    State(&'a str),
    Begin(&'a str),
}

pub struct TransitionDeclaration<'a> {
    name: &'a str,
    source: &'a str,
    destination: &'a str,
    input: Option<Event<'a>>,
    output: Option<Vec<Event<'a>>>,
    rel_label: Option<&'a str>,
    obs_label: Option<&'a str>,
}

impl<'a> TransitionDeclaration<'a> {
    pub fn simple_decl(name: &'a str, source: &'a str, destination: &'a str) -> Self {
        Self {
            name,
            source,
            destination,
            input: None,
            output: None,
            rel_label: None,
            obs_label: None,
        }
    }
}

#[derive(Default)]
pub struct TransitionParameterFactory<T> {
    param: Option<T>,
}

impl<T> TransitionParameterFactory<T>
where
    T: Default,
{
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_value(mut self, param: T) -> Option<Self> {
        if let Some(_) = self.param {
            None
        } else {
            self.param = Some(param);
            Some(self)
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
}

impl<'a> ComplexTransactionFactory<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn build_transition(self, name: &'a str) -> Option<TransitionDeclaration<'a>> {
        if self.src.is_set() && self.dst.is_set() {
            let output = TransitionDeclaration {
                name,
                source: self.src.unwrap(),
                destination: self.dst.unwrap(),
                input: self.input.get_param(),
                output: self.output.get_param(),
                rel_label: self.rel.get_param(),
                obs_label: self.obs.get_param(),
            };
            Some(output)
        } else {
            None
        }
    }

    pub fn set_parameter(mut self, key: TransitionKeys<'a>) -> Option<Self> {
        match key {
            TransitionKeys::Src(param) => self.src = self.src.set_value(param)?,
            TransitionKeys::Dst(param) => self.dst = self.dst.set_value(param)?,
            TransitionKeys::Input(param) => self.input = self.input.set_value(param)?,
            TransitionKeys::Output(param) => self.output = self.output.set_value(param)?,
            TransitionKeys::Rel(param) => self.rel = self.rel.set_value(param)?,
            TransitionKeys::Obs(param) => self.obs = self.obs.set_value(param)?,
        }
        None
    }
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

#[derive(Default)]
pub struct Event<'a> {
    name: &'a str,
    link: &'a str,
}

impl<'a> Event<'a> {
    pub fn new(name: &'a str, link: &'a str) -> Self {
        Self { name, link }
    }
}

pub struct Link<'a> {
    name: &'a str,
    source: &'a str,
    destination: &'a str,
}

impl<'a> Link<'a> {
    pub fn new(name: &'a str, source: &'a str, destination: &'a str) -> Self {
        Self {
            name,
            source,
            destination,
        }
    }
}

pub struct Request<'a> {
    list: Vec<Command<'a>>,
}

impl<'a> Request<'a> {
    pub fn new(list: Vec<Command<'a>>) -> Self {
        Self { list }
    }
}

pub enum Command<'a> {
    Space(&'a str),
    Linspace(LinspaceCommand<'a>),
    Diagnosis(DiagnosisCommand<'a>),
}

pub struct LinspaceCommand<'a> {
    name: &'a str,
    name_list: Vec<&'a str>,
}

impl<'a> LinspaceCommand<'a> {
    pub fn new(name: &'a str, name_list: Vec<&'a str>) -> Self {
        Self { name, name_list }
    }
}

pub struct DiagnosisCommand<'a> {
    name: &'a str,
    name_list: Vec<&'a str>,
}

impl<'a> DiagnosisCommand<'a> {
    pub fn new(name: &'a str, name_list: Vec<&'a str>) -> Self {
        Self { name, name_list }
    }
}
