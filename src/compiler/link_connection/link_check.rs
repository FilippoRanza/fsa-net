use super::super::compiler_utils::is_network;
use ahash::AHashMap;
use fsa_net_parser::syntax_tree::*;
use fsa_net_parser::Code;

pub fn link_check<'a>(code: &'a Code<'a>) -> Result<(), LinkError<'a>> {
    code.iter()
        .filter_map(is_network)
        .map(CheckLinkFactory::new)
        .try_fold((), |_, curr| validate_link(curr))
}

#[derive(Debug)]
pub enum LinkError<'a> {
    NotInput(LinkConnectionError<'a>),
    NotOutput(LinkConnectionError<'a>),
    MultipleLinkUse(Vec<LinkCountError<'a>>),
}

impl<'a> LinkError<'a> {
    fn new_not_input_error(automata: &'a str, link: &'a str) -> Self {
        Self::NotInput(LinkConnectionError { automata, link })
    }

    fn new_not_output_error(automata: &'a str, link: &'a str) -> Self {
        Self::NotOutput(LinkConnectionError { automata, link })
    }
}

#[derive(Debug)]
pub struct LinkCountError<'a> {
    pub automata: &'a str,
    pub link: &'a str,
    pub count: usize,
}

impl<'a> LinkCountError<'a> {
    fn new(automata: &'a str, link: &'a str, count: usize) -> Self {
        Self {
            automata,
            link,
            count,
        }
    }
}

#[derive(Debug)]
pub struct LinkConnectionError<'a> {
    pub automata: &'a str,
    pub link: &'a str,
}

fn validate_link<'a>(factory: CheckLinkFactory<'a>) -> Result<(), LinkError<'a>> {
    let mut usage_counter = LinkUsageCounter::default();
    for trans in &factory.links_use {
        usage_counter.count(trans);
        let link = factory.links_def.get(trans.link).unwrap();
        match trans.usage {
            LinkUsageType::Input => {
                if link.dst != trans.automata {
                    return Err(LinkError::new_not_input_error(trans.automata, link.name));
                }
            }
            LinkUsageType::Output => {
                if link.src != trans.automata {
                    return Err(LinkError::new_not_output_error(trans.automata, link.name));
                }
            }
        }
    }

    if let Some(multiple_use_err) = usage_counter.collect_error() {
        Err(LinkError::MultipleLinkUse(multiple_use_err))
    } else {
        Ok(())
    }
}

#[derive(Default)]
struct CheckLinkFactory<'a> {
    links_def: AHashMap<&'a str, LinkInfo<'a>>,
    links_use: Vec<LinkUsage<'a>>,
}

impl<'a> CheckLinkFactory<'a> {
    fn new(net: &'a Network<'a>) -> Self {
        let output = Self::default();
        net.params
            .iter()
            .fold(output, |acc, curr| acc.insert_network_param(curr))
    }

    fn insert_network_param(self, param: &'a NetworkParameterDecl<'a>) -> Self {
        match &param.param {
            NetworkParameter::Automata(auto) => self.insert_automata(auto),
            NetworkParameter::Link(link) => self.insert_link(link),
            _ => self,
        }
    }

    fn insert_link(mut self, link: &Link<'a>) -> Self {
        let info = LinkInfo::new(link);
        self.links_def.insert(link.name, info);
        self
    }

    fn insert_automata(self, auto: &'a Automata<'a>) -> Self {
        auto.params
            .iter()
            .filter_map(is_transaction)
            .fold(self, |acc, curr| acc.insert_transition(curr, &auto.name))
    }

    fn insert_transition(mut self, trans: &TransitionDeclaration<'a>, auto_name: &'a str) -> Self {
        if let Some(input) = &trans.input {
            let info = LinkUsage::new(auto_name, input.link, trans.name, LinkUsageType::Input);
            self.links_use.push(info);
        }

        if let Some(outputs) = &trans.output {
            for output in outputs {
                let info =
                    LinkUsage::new(auto_name, output.link, trans.name, LinkUsageType::Output);
                self.links_use.push(info);
            }
        }

        self
    }
}

fn is_transaction<'a, 'b: 'a>(
    param: &'b AutomataParameterDecl<'a>,
) -> Option<&'b TransitionDeclaration<'a>> {
    match &param.param {
        AutomataParameter::StateDecl(_) => None,
        AutomataParameter::Transition(trans) => Some(trans),
    }
}

struct LinkUsage<'a> {
    automata: &'a str,
    link: &'a str,
    trans: &'a str,
    usage: LinkUsageType,
}

impl<'a> LinkUsage<'a> {
    fn new(
        automata_name: &'a str,
        link_name: &'a str,
        trans: &'a str,
        usage: LinkUsageType,
    ) -> Self {
        Self {
            automata: automata_name,
            link: link_name,
            trans,
            usage,
        }
    }
}

enum LinkUsageType {
    Input,
    Output,
}

struct LinkInfo<'a> {
    name: &'a str,
    src: &'a str,
    dst: &'a str,
}
impl<'a> LinkInfo<'a> {
    fn new(lk: &Link<'a>) -> Self {
        Self {
            name: lk.name,
            src: lk.source,
            dst: lk.destination,
        }
    }
}

#[derive(Default)]
struct LinkUsageCounter<'a> {
    counter: AHashMap<LinkUsageKey<'a>, usize>,
}

impl<'a> LinkUsageCounter<'a> {
    fn count(&mut self, info: &LinkUsage<'a>) {
        let key = info.into();
        if let Some(count) = self.counter.get_mut(&key) {
            *count += 1;
        } else {
            self.counter.insert(key, 1);
        }
    }

    fn collect_error(self) -> Option<Vec<LinkCountError<'a>>> {
        let count_err: Vec<LinkCountError<'a>> = self
            .counter
            .into_iter()
            .filter(|(_, v)| *v > 1)
            .map(|(k, v)| LinkCountError::new(k.automata, k.link, v))
            .collect();
        if count_err.len() > 0 {
            Some(count_err)
        } else {
            None
        }
    }
}

#[derive(Hash, PartialEq, Eq)]
struct LinkUsageKey<'a> {
    automata: &'a str,
    link: &'a str,
    trans: &'a str,
}

impl<'a> LinkUsageKey<'a> {
    fn new(automata: &'a str, link: &'a str, trans: &'a str) -> Self {
        Self {
            automata,
            link,
            trans,
        }
    }
}

impl<'a> From<&LinkUsage<'a>> for LinkUsageKey<'a> {
    fn from(usage: &LinkUsage<'a>) -> Self {
        Self::new(usage.automata, usage.link, usage.trans)
    }
}
