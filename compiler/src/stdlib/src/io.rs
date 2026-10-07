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

/// Read the entire contents of a file, returning None if it can't be read
pub fn read_file(path: &str) -> Option<String> {
    std::fs::read_to_string(path).ok()
}

/// Write contents to a file, returning an error message on failure
pub fn write_file(path: &str, contents: &str) -> Result<(), String> {
    std::fs::write(path, contents).map_err(|e| e.to_string())
}
