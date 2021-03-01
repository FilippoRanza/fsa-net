mod name_table;
mod name_table_factory;
mod request_table;
mod macros;

type Loc = (usize, usize);

pub use name_table_factory::build_name_table;
