mod macros;
mod name_class;
mod name_error;
mod name_table;
mod name_table_factory;
mod request_table;
mod class_index;

type Loc = (usize, usize);

pub use name_error::NameError;
pub use name_table::GlobalNameTable;
pub use name_table_factory::build_name_table;
