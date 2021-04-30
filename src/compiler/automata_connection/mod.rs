mod adjacent_list_graph;
mod check_connection;
pub type GraphError<'a> = Vec<&'a str>;

pub use adjacent_list_graph::GraphBuilder;
pub use check_connection::check_connection;
