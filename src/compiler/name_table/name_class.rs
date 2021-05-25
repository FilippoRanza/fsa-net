/**
 * The various type of entities definable
 * in fsa-lang
 */
#[derive(Debug, PartialEq, Eq, Copy, Clone, Hash)]
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