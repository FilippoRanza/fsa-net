pub fn zeros<T>(count: usize) -> Vec<T>
where
    T: Default,
{
    (0..count).map(|_| T::default()).collect()
}