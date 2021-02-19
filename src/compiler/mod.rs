#[derive(Debug)]
pub struct Location {
    begin: usize,
    end: usize,
}
impl Location {
    fn new(begin: usize, end: usize) -> Location {
        Self { begin, end }
    }

    fn from_tuple(loc: (usize, usize)) -> Location {
        Self::new(loc.0, loc.1)
    }
}

impl From<(usize, usize)> for Location {
    fn from(loc: (usize, usize)) -> Self {
        Self::from_tuple(loc)
    }
}

mod name_table;
mod name_table_factory;

pub use name_table_factory::build_name_table;
