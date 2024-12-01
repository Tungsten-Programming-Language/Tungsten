/// List manipulation functions for the standard library

/// Compute the length of a list
pub fn len<T>(list: &[T]) -> usize {
    list.len()
}

/// Reverse a list
pub fn reverse<T: Clone>(list: &[T]) -> Vec<T> {
    list.iter().rev().cloned().collect()
}

/// Map a function over a list
pub fn map<T, U, F>(list: &[T], f: F) -> Vec<U>
where
    F: Fn(&T) -> U,
{
    list.iter().map(f).collect()
}
