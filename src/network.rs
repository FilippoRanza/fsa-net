#[derive(PartialEq, Eq, Clone, Debug, Hash)]
pub struct State {
    states: Vec<usize>,
    links: Vec<Option<usize>>,
}

impl State {
    pub fn initial(automata_count: usize, link_count: usize) -> Self {
        Self {
            states: zeros(automata_count),
            links: zeros(link_count),
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

pub struct Network {
    automata: Vec<Automata>,
    links: Vec<Link>,
}

impl Network {
    fn step_one(&self, state: &State) -> Vec<(TransEvent, State)> {
        let mut output = Vec::new();
        for auto in &self.automata {
            let mut next = auto.step_one(state);
            output.append(&mut next)
        }
        output
    }
}

struct Automata {
    adjacent_list: Vec<Vec<Adjacent>>,
    index: usize,
}

impl Automata {
    fn new(index: usize, adjacent_list: Vec<Vec<Adjacent>>) -> Self {
        Self {
            adjacent_list,
            index,
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
}

struct Adjacent {
    state: usize,
    trans: Transition,
}

struct Transition {
    input: Option<Event>,
    output: Option<Vec<Event>>,
    rel: Option<usize>,
    obs: Option<usize>,
}

impl Transition {
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

pub struct TransEvent {
    obs: Option<usize>,
    rel: Option<usize>,
}

impl From<&Transition> for TransEvent {
    fn from(trans: &Transition) -> Self {
        Self {
            obs: trans.obs,
            rel: trans.rel,
        }
    }
}

struct Event {
    event: usize,
    link: usize,
}

struct Link {
    src: usize,
    dst: usize,
}

fn zeros<T>(count: usize) -> Vec<T>
where
    T: Default,
{
    (0..count).map(|_| T::default()).collect()
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn test_initial_state() {
        let initial = State::initial(4, 1);
        assert!(initial.is_final());
    }

    #[test]
    fn test_enabled_transition() {
        let state = State::initial(3, 2).fill_link(1, 3);
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
        let state = State::initial(3, 2).fill_link(1, 3);

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
}
