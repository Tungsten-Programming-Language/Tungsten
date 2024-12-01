/// Basic mathematical functions for the standard library

/// Compute the factorial of a number
pub fn factorial(n: u64) -> u64 {
    (1..=n).product()
}

/// Compute the power of a number
pub fn pow(base: f64, exponent: f64) -> f64 {
    base.powf(exponent)
}

/// Compute the square root of a number
pub fn sqrt(x: f64) -> f64 {
    x.sqrt()
}
