mod macros;
mod name_class;
mod name_error;
mod name_table;
mod name_table_factory;
mod request_table;

type Loc = (usize, usize);

pub use name_error::NameError;
pub use name_table_factory::build_name_table;
