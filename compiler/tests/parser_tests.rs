#[cfg(test)]
mod tests {
    use crate::ast::{Expression, LogLevel};
    use crate::parser::Parser;

    #[test]
    fn test_log_debug_parsing() {
        let mut parser = Parser::new("LogDebug[\"Debug message\"]".to_string());
        let expr = parser.parse().unwrap();
        
        match expr {
            Expression::LogCall { level, message } => {
                assert_eq!(level, LogLevel::Debug);
                match *message {
                    Expression::String(msg) => assert_eq!(msg, "Debug message"),
                    _ => panic!("Expected string message"),
                }
            }
            _ => panic!("Expected LogCall expression"),
        }
    }

    #[test]
    fn test_log_info_parsing() {
        let mut parser = Parser::new("LogInfo[\"Info message\"]".to_string());
        let expr = parser.parse().unwrap();
        
        match expr {
            Expression::LogCall { level, message } => {
                assert_eq!(level, LogLevel::Info);
                match *message {
                    Expression::String(msg) => assert_eq!(msg, "Info message"),
                    _ => panic!("Expected string message"),
                }
            }
            _ => panic!("Expected LogCall expression"),
        }
    }

    #[test]
    fn test_log_warn_parsing() {
        let mut parser = Parser::new("LogWarn[\"Warning message\"]".to_string());
        let expr = parser.parse().unwrap();
        
        match expr {
            Expression::LogCall { level, message } => {
                assert_eq!(level, LogLevel::Warn);
                match *message {
                    Expression::String(msg) => assert_eq!(msg, "Warning message"),
                    _ => panic!("Expected string message"),
                }
            }
            _ => panic!("Expected LogCall expression"),
        }
    }

    #[test]
    fn test_log_error_parsing() {
        let mut parser = Parser::new("LogError[\"Error message\"]".to_string());
        let expr = parser.parse().unwrap();
        
        match expr {
            Expression::LogCall { level, message } => {
                assert_eq!(level, LogLevel::Error);
                match *message {
                    Expression::String(msg) => assert_eq!(msg, "Error message"),
                    _ => panic!("Expected string message"),
                }
            }
            _ => panic!("Expected LogCall expression"),
        }
    }

    #[test]
    fn test_log_with_non_string_message() {
        let mut parser = Parser::new("LogInfo[42]".to_string());
        let expr = parser.parse().unwrap();
        
        match expr {
            Expression::LogCall { level, message } => {
                assert_eq!(level, LogLevel::Info);
                match *message {
                    Expression::Number(num) => assert_eq!(num, 42),
                    _ => panic!("Expected number message"),
                }
            }
            _ => panic!("Expected LogCall expression"),
        }
    }
}
