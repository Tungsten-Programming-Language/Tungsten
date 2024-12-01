/// String manipulation functions for the standard library

/// Convert a string to uppercase
pub fn to_uppercase(s: &str) -> String {
    s.to_uppercase()
}

/// Convert a string to lowercase
pub fn to_lowercase(s: &str) -> String {
    s.to_lowercase()
}

/// Trim whitespace from the start and end of a string
pub fn trim(s: &str) -> String {
    s.trim().to_string()
}
