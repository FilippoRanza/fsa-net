use crate::utils::zeros;

#[derive(PartialEq, Eq, Clone, Debug, Hash)]
pub struct State {
    states: Vec<usize>,
    links: Vec<Option<usize>>,
    index: usize,
}

impl State {
    pub fn initial(states: Vec<usize>, link_count: usize) -> Self {
        Self {
            states,
            links: zeros(link_count),
            index: 0,
        }
    }

    pub fn is_final(&self) -> bool {
        for l in &self.links {
            if l.is_some() {
                return false;
            }
        }
        true
    }

    pub fn set_index(mut self, index: usize) -> Self {
        self.index = index;
        self
    }

    pub fn get_index(&self) -> usize {
        self.index
    }

    pub fn get_states<'a>(&'a self) -> impl Iterator<Item = (usize, usize)> + 'a {
        self.states.iter().enumerate().map(|(i, s)| (i, *s))
    }

    pub fn get_links<'a>(&'a self) -> impl Iterator<Item = (usize, Option<usize>)> + 'a {
        self.links.iter().enumerate().map(|(i, l)| (i, *l))
    }

    fn get_state(&self, automata: usize) -> usize {
        self.states[automata]
    }

    fn set_state(&self, automata: usize, next: usize) -> Self {
        let mut out = self.clone();
        out.states[automata] = next;
        out
    }

    fn fill_link(mut self, link: usize, val: usize) -> Self {
        self.links[link] = Some(val);
        self
    }

    fn drain_link(mut self, link: usize) -> Self {
        self.links[link] = None;
        self
    }

    fn is_empty_link(&self, link: usize) -> bool {
        self.links[link].is_none()
    }

    fn has_event_link(&self, link: usize, ev: usize) -> bool {
        let link = self.links[link];
        if let Some(event) = link {
            event == ev
        } else {
            false
        }
    }
}
#[derive(Debug, PartialEq)]
pub struct Network {
    automata: Vec<Automata>,
    links: Vec<Link>,
}

impl Network {
    pub fn new(automata: Vec<Automata>, links: Vec<Link>) -> Self {
        Self { automata, links }
    }

    pub fn get_initial_state(&self) -> State {
        State::initial(self.get_automata_initial_state(), self.links.len())
    }

    pub fn step_one(&self, state: &State) -> Vec<(TransEvent, State)> {
        let mut output = Vec::new();
        for auto in &self.automata {
            let mut next = auto.step_one(state);
            output.append(&mut next)
        }
        output
    }

    fn get_automata_initial_state(&self) -> Vec<usize> {
        self.automata
            .iter()
            .map(|a| a.get_initial_state())
            .collect()
    }
}

#[derive(Debug, PartialEq)]
pub struct Automata {
    adjacent_list: Vec<Vec<Adjacent>>,
    index: usize,
    begin: usize,
}

impl Automata {
    pub fn new(begin: usize, index: usize, adjacent_list: Vec<Vec<Adjacent>>) -> Self {
        Self {
            adjacent_list,
            index,
            begin,
        }
    }

    fn step_one(&self, net_state: &State) -> Vec<(TransEvent, State)> {
        let curr_state = net_state.get_state(self.index);
        let next_states = &self.adjacent_list[curr_state];
        let mut output = Vec::new();
        for trans in next_states.iter() {
            if trans.trans.is_enabled(net_state) {
                let next = net_state.set_state(self.index, trans.state);
                let next = trans.trans.apply_transition(next);
                output.push(next);
            }
        }
        output
    }

    fn get_initial_state(&self) -> usize {
        self.begin
    }
}

#[derive(Debug, PartialEq)]
pub struct Adjacent {
    state: usize,
    trans: Transition,
}

impl Adjacent {
    pub fn new(state: usize, trans: Transition) -> Self {
        Self { state, trans }
    }
}

#[derive(Default, Debug, PartialEq)]
pub struct Transition {
    input: Option<Event>,
    output: Option<Vec<Event>>,
    rel: Option<usize>,
    obs: Option<usize>,
}

impl Transition {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_input(mut self, ev: Event) -> Self {
        self.input = Some(ev);
        self
    }

    pub fn add_output(mut self, ev: Event) -> Self {
        if let Some(output) = &mut self.output {
            output.push(ev);
        } else {
            self.output = Some(vec![ev]);
        }
        self
    }

    pub fn set_relevance(mut self, rel: usize) -> Self {
        self.rel = Some(rel);
        self
    }

    pub fn set_observability(mut self, obs: usize) -> Self {
        self.obs = Some(obs);
        self
    }

    fn is_enabled(&self, state: &State) -> bool {
        if let Some(input) = &self.input {
            if !state.has_event_link(input.link, input.event) {
                return false;
            }
        }

        if let Some(output) = &self.output {
            for out in output {
                if !state.is_empty_link(out.link) {
                    return false;
                }
            }
        }

        true
    }

    fn apply_transition(&self, mut state: State) -> (TransEvent, State) {
        if let Some(input) = &self.input {
            state = state.drain_link(input.link);
        }

        if let Some(output) = &self.output {
            for out in output {
                state = state.fill_link(out.link, out.event);
            }
        }

        (self.into(), state)
    }
}

#[derive(Debug)]
pub struct TransEvent {
    pub obs: Option<usize>,
    pub rel: Option<usize>,
}

impl From<&Transition> for TransEvent {
    fn from(trans: &Transition) -> Self {
        Self {
            obs: trans.obs,
            rel: trans.rel,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Event {
    event: usize,
    link: usize,
}

impl Event {
    pub fn new(event: usize, link: usize) -> Self {
        Self { event, link }
    }
}

#[derive(Debug, PartialEq)]
pub struct Link {
    src: usize,
    dst: usize,
}

impl Link {
    pub fn new(src: usize, dst: usize) -> Self {
        Self { src, dst }
    }
}

#[cfg(test)]
mod test {

    use super::*;
    use crate::compiler::compile;
    use fsa_net_parser::parse;
    use test_utils::load_code_from_file;

    #[test]
    fn test_compile() {
        let src_code = load_code_from_file("simple-network");
        let code = parse(&src_code).expect("`simple-network` should be syntactically correct");
        let comp_res = compile(&code).expect("`simple-network` should be semantically correct");
        let net = &comp_res.compile_network[0].net;

        let trans_a_a = Transition::new()
            .set_input(Event::new(0, 0))
            .add_output(Event::new(1, 1))
            .set_observability(0);
        let trans_b_a = Transition::new()
            .add_output(Event::new(1, 1))
            .set_relevance(0);

        let auto_a = Automata::new(
            1,
            0,
            vec![
                vec![Adjacent::new(1, trans_b_a)],
                vec![Adjacent::new(0, trans_a_a)],
            ],
        );

        let trans_a_b = Transition::new()
            .add_output(Event::new(0, 0))
            .set_observability(1);
        let trans_b_b = Transition::new().set_input(Event::new(1, 1));
        let trans_c_b = Transition::new()
            .set_input(Event::new(1, 1))
            .set_relevance(1);

        let auto_b = Automata::new(
            0,
            1,
            vec![
                vec![Adjacent::new(1, trans_a_b)],
                vec![Adjacent::new(0, trans_b_b), Adjacent::new(1, trans_c_b)],
            ],
        );

        let expect_net = Network::new(vec![auto_a, auto_b], vec![Link::new(1, 0), Link::new(0, 1)]);

        assert_eq!(&expect_net, net);
    }

    #[test]
    fn test_initial_state() {
        let initial = State::initial(zeros(4), 1);
        assert!(initial.is_final());
    }

    #[test]
    fn test_enabled_transition() {
        let state = State::initial(zeros(3), 2).fill_link(1, 3);
        let trans = Transition {
            input: Some(Event { event: 3, link: 1 }),
            output: None,
            rel: None,
            obs: None,
        };
        assert!(trans.is_enabled(&state));

        let trans = Transition {
            input: None,
            output: Some(vec![Event { event: 3, link: 1 }]),
            rel: None,
            obs: None,
        };
        assert!(!trans.is_enabled(&state));

        let trans = Transition {
            input: Some(Event { event: 3, link: 1 }),
            output: Some(vec![Event { event: 2, link: 0 }]),
            rel: None,
            obs: None,
        };
        assert!(trans.is_enabled(&state));
    }

    #[test]
    fn test_apply_transition() {
        let state = State::initial(zeros(3), 2).fill_link(1, 3);

        let in_link = 1;
        let out_link = 0;
        let in_ev = 3;
        let out_ev = 2;
        let trans = Transition {
            input: Some(Event {
                event: in_ev,
                link: in_link,
            }),
            output: Some(vec![Event {
                event: out_ev,
                link: out_link,
            }]),
            rel: Some(31),
            obs: Some(12),
        };
        assert!(trans.is_enabled(&state));

        let (event, state) = trans.apply_transition(state);

        assert!(state.links[in_link].is_none());
        assert_eq!(state.links[out_link].unwrap(), out_ev);

        assert_eq!(event.obs.unwrap(), 12);
        assert_eq!(event.rel.unwrap(), 31);
    }

    #[test]
    fn test_step_one() {
        let in_link = 1;
        let out_link = 0;
        let in_ev = 3;
        let out_ev = 2;
        let trans = Transition {
            input: Some(Event {
                event: in_ev,
                link: in_link,
            }),
            output: Some(vec![Event {
                event: out_ev,
                link: out_link,
            }]),
            rel: Some(31),
            obs: Some(12),
        };

        let automata = Automata::new(
            0,
            0,
            vec![vec![Adjacent {
                state: 1,
                trans: trans,
            }]],
        );

        let state = State::initial(zeros(1), 2).fill_link(1, 3);

        let next = automata.step_one(&state);
        assert_eq!(next.len(), 1);

        let (event, state) = &next[0];

        assert!(state.links[in_link].is_none());
        assert_eq!(state.links[out_link].unwrap(), out_ev);

        assert_eq!(state.states[0], 1);

        assert_eq!(event.obs.unwrap(), 12);
        assert_eq!(event.rel.unwrap(), 31);
    }
}
