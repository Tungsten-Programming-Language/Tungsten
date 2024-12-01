/// List manipulation functions

/// Create a new list
pub fn new<T>() -> Vec<T> {
    Vec::new()
}

/// Append an element to the end of a list
pub fn append<T>(list: &mut Vec<T>, element: T) {
    list.push(element);
}

/// Remove the last element from a list
pub fn pop<T>(list: &mut Vec<T>) -> Option<T> {
    list.pop()
}

/// Get the length of a list
pub fn length<T>(list: &Vec<T>) -> usize {
    list.len()
}

/// Check if a list is empty
pub fn is_empty<T>(list: &Vec<T>) -> bool {
    list.is_empty()
}

/// Remove an element at a specific index
pub fn remove<T>(list: &mut Vec<T>, index: usize) -> Option<T> {
    if index < list.len() {
        Some(list.remove(index))
    } else {
        None
    }
}
