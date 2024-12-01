/// String manipulation functions

/// Convert a string to uppercase
pub fn to_uppercase(s: &str) -> String {
    s.to_uppercase()
}

/// Convert a string to lowercase
pub fn to_lowercase(s: &str) -> String {
    s.to_lowercase()
}

/// Get the length of a string
pub fn length(s: &str) -> usize {
    s.len()
}

/// Check if a string contains a substring
pub fn contains(s: &str, substring: &str) -> bool {
    s.contains(substring)
}

/// Trim whitespace from start and end of a string
pub fn trim(s: &str) -> String {
    s.trim().to_string()
}
