use std::fmt::Display;

pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

pub fn log<T: Display>(level: LogLevel, message: T) {
    let prefix = match level {
        LogLevel::Debug => "[DEBUG]",
        LogLevel::Info => "[INFO]",
        LogLevel::Warn => "[WARN]",
        LogLevel::Error => "[ERROR]",
    };
    println!("{} {}", prefix, message);
}

// Convenience functions for each log level
pub fn debug<T: Display>(message: T) {
    log(LogLevel::Debug, message);
}

pub fn info<T: Display>(message: T) {
    log(LogLevel::Info, message);
}

pub fn warn<T: Display>(message: T) {
    log(LogLevel::Warn, message);
}

pub fn error<T: Display>(message: T) {
    log(LogLevel::Error, message);
}
