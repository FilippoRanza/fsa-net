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

mod name_table;
