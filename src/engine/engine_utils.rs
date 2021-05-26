use crate::state_table;
use std::collections::VecDeque;

pub fn get_next_index<T>(
    stat: T,
    table: &mut state_table::StateTable<T>,
    stack: &mut VecDeque<usize>,
) -> usize
where
    T: Eq + std::hash::Hash,
{
    if !table.is_present(&stat) {
        let tmp_index = table.insert_state(stat);
        stack.push_front(tmp_index);
        tmp_index
    } else {
        table.get_index(&stat).unwrap()
    }
}
