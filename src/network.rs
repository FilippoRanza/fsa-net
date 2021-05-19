


#[derive(PartialEq, Clone, Debug)]
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
    links: Vec<Link>
}

impl Network {
    fn step_one(&self, state: &State) -> Vec<State> {
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
    index: usize
}

impl Automata {
    fn new(index: usize, adjacent_list: Vec<Vec<Adjacent>>) -> Self {
        Self { adjacent_list, index }
    }

    fn step_one(&self, net_state: &State) -> Vec<State> {
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

    fn apply_transition(&self, mut state: State) -> State {
        if let Some(input) = &self.input {
            state = state.drain_link(input.link);
        }

        if let Some(output) = &self.output {
            for out in output {
                state = state.fill_link(out.link, out.event);
            }
        }
        state
    }

}


struct Event {
    event: usize,
    link: usize
}

struct Link {
    src: usize, 
    dst: usize
}

fn zeros<T>(count: usize) -> Vec<T>
where
    T: Default,
{
    (0..count).map(|_| T::default()).collect()
}
