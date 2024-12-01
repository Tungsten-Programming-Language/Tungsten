/// Mathematical utility functions

/// Absolute value of a number
pub fn abs(x: i64) -> i64 {
    x.abs()
}

/// Square root of a number (integer approximation)
pub fn sqrt(x: i64) -> i64 {
    (x as f64).sqrt() as i64
}

/// Power function
pub fn pow(base: i64, exponent: u32) -> i64 {
    base.pow(exponent)
}

/// Maximum of two numbers
pub fn max(a: i64, b: i64) -> i64 {
    a.max(b)
}

/// Minimum of two numbers
pub fn min(a: i64, b: i64) -> i64 {
    a.min(b)
}
