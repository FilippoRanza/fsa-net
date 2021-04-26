mod check_connection;
mod link_table_graph;
pub type GraphError<'a> = Vec<&'a str>;

pub use check_connection::check_connection;
pub use link_table_graph::GraphBuilder;
