pub fn zeros<T>(count: usize) -> Vec<T>
where
    T: Default,
{
    (0..count).map(|_| T::default()).collect()
}

pub fn auto_sort<T, K>(iter: &mut impl Iterator<Item = (T, K)>) -> Vec<T>
where
    K: Into<usize>,
{
    let mut tmp: Vec<(T, usize)> = iter.map(|(v, i)| (v, i.into())).collect();
    tmp.sort_by_key(|(_, i)| *i);
    tmp.into_iter().map(|(v, _)| v).collect()
}
