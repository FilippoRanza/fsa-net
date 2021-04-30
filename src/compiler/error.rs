use super::automata_connection::GraphError;
use super::link_connection::LinkError;
use super::name_table::NameError;

#[derive(Debug)]
pub enum CompileError<'a> {
    NameError(NameError<'a>),
    GraphError(GraphError<'a>),
    LinkError(LinkError<'a>),
}

#[macro_export]
macro_rules! into_compile_error {
    ($name:ident ) => {
        impl<'a> From<$name<'a>> for CompileError<'a> {
            fn from(err: $name) -> CompileError {
                CompileError::$name(err)
            }
        }
    };
}

into_compile_error! {NameError}
into_compile_error! {GraphError}
into_compile_error! {LinkError}
