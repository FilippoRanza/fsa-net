use crate::state_table;
use crate::timer;
use std::collections::VecDeque;

pub fn get_next_index<T>(
    stat: T,
    table: &mut state_table::StateTable<T>,
    stack: &mut VecDeque<usize>,
) -> (usize, bool)
where
    T: Eq + std::hash::Hash,
{
    if !table.is_present(&stat) {
        let tmp_index = table.insert_state(stat);
        stack.push_front(tmp_index);
        (tmp_index, true)
    } else {
        (table.get_index(&stat).unwrap(), false)
    }
}

pub fn get_next_state<T>(stack: &mut VecDeque<T>, timer: &timer::Timer) -> Option<T> {
    if timer.timeout() {
        None
    } else {
        stack.pop_front()
    }
}
