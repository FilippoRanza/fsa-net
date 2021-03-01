
use super::Loc;
use super::name_class::NameClass;
use crate::into_name_error;

#[derive(Debug)]
pub enum NameError<'a> {
    UndefinedNetwork(UndefinedNetwork<'a>),
    NameRidefinitionError(NameRidefinitionError<'a>),
    BeginStateError(BeginStateError<'a>),
    UndefinedNameError(UndefinedNameError<'a>),
    UndefinedLabel(UndefinedLabel<'a>),
    MismatchedType(MismatchedType<'a>),
}

into_name_error! {UndefinedNetwork}
into_name_error! {NameRidefinitionError}
into_name_error! {BeginStateError}
into_name_error! {UndefinedNameError}
into_name_error! {UndefinedLabel}
into_name_error! {MismatchedType}

#[derive(Debug)]
pub struct UndefinedLabel<'a> {
    pub name: &'a str,
    pub class: NameClass,
}

#[derive(Debug)]
pub struct MismatchedType<'a> {
    pub name: &'a str,
    pub orig: NameClass,
    pub curr: NameClass,
}

#[derive(Debug)]
pub struct UndefinedNameError<'a> {
    pub name: &'a str,
    pub loc: Loc,
}

#[derive(Debug)]
pub struct UndefinedNetwork<'a> {
    pub names: Vec<(&'a str, Loc)>,
}

#[derive(Debug)]
pub struct NameRidefinitionError<'a> {
    pub name: &'a str,
    pub orig_loc: Loc,
    pub ridef_loc: Loc,
    pub orig_class: NameClass,
    pub ridef_class: NameClass,
}

#[derive(Debug)]
pub enum BeginStateError<'a> {
    NoBeginState,
    MultipleBeginState(Vec<&'a str>),
}

