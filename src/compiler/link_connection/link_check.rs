use super::super::compiler_utils::is_network;
use fsa_net_parser::syntax_tree::*;
use fsa_net_parser::Code;
use std::collections::HashMap;

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
    let mut usage_counter = HashMap::new();
    for trans in &factory.links_use {
        let link = factory.links_def.get(trans.link).unwrap();
        if let Some(count) = usage_counter.get_mut(&(trans.automata, link.name)) {
            *count += 1;
        } else {
            usage_counter.insert((trans.automata, link.name), 1);
        }
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

    let multiple_use: Vec<LinkCountError> = usage_counter
        .into_iter()
        .filter(|(_, v)| *v > 1)
        .map(|((a, l), v)| LinkCountError::new(a, l, v))
        .collect();
    if multiple_use.len() > 0 {
        Err(LinkError::MultipleLinkUse(multiple_use))
    } else {
        Ok(())
    }
}

#[derive(Default)]
struct CheckLinkFactory<'a> {
    links_def: HashMap<&'a str, LinkInfo<'a>>,
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
            let info = LinkUsage::new(auto_name, input.link, LinkUsageType::Input);
            self.links_use.push(info);
        }

        if let Some(outputs) = &trans.output {
            for output in outputs {
                let info = LinkUsage::new(auto_name, output.link, LinkUsageType::Output);
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
    usage: LinkUsageType,
}

impl<'a> LinkUsage<'a> {
    fn new(automata_name: &'a str, link_name: &'a str, usage: LinkUsageType) -> Self {
        Self {
            automata: automata_name,
            link: link_name,
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
