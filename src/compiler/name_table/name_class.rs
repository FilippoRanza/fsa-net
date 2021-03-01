#[derive(Debug, PartialEq, Copy, Clone)]
pub enum NameClass {
    Network,
    Request,
    Automata,
    Link,
    Event,
    ObsLabel,
    RelLabel,
    State,
    Transition,
}
