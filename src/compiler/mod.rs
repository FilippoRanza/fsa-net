
pub struct Location {
    begin: usize,
    end: usize
}
impl Location {
    fn new(begin: usize, end: usize) -> Location {
        Self {
            begin, end
        }
    }
}



mod name_table;

