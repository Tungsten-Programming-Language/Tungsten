/// Basic input/output functions for the standard library

/// Print a message to the console
pub fn print<T: std::fmt::Display>(message: T) {
    println!("{}", message);
}

/// Read a line from standard input
pub fn read_line() -> String {
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).expect("Failed to read line");
    input.trim().to_string()
}
