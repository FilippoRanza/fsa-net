mod check_connection;
mod adjacent_list_graph;
pub type GraphError<'a> = Vec<&'a str>;

pub use check_connection::check_connection;
pub use adjacent_list_graph::GraphBuilder;
