mod tests {
    use w::ast::{Expression, LogLevel};
    use w::parser::Parser;

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

    // New tests for Cond expression parsing
    #[test]
    fn test_cond_single_condition() {
        let mut parser = Parser::new("Cond[[x > 10 Print[\"Greater than 10\"]]]".to_string());
        let expr = parser.parse().unwrap();
        
        match expr {
            Expression::Cond { conditions, default_statements } => {
                assert_eq!(conditions.len(), 0);
                assert!(default_statements.is_some());
                
                match *default_statements.unwrap() {
                    Expression::FunctionCall { 
                        function: func, 
                        arguments 
                    } => {
                        match *func {
                            Expression::Identifier(name) => assert_eq!(name, "Print"),
                            _ => panic!("Expected Print function"),
                        }
                        assert_eq!(arguments.len(), 1);
                        match arguments[0] {
                            Expression::String(ref msg) => assert_eq!(msg, "Greater than 10"),
                            _ => panic!("Expected string argument"),
                        }
                    }
                    _ => panic!("Expected function call"),
                }
            }
            _ => panic!("Expected Cond expression"),
        }
    }

    #[test]
    fn test_cond_multiple_conditions() {
        let mut parser = Parser::new("Cond[[x > 10 Print[\"Greater than 10\"]] [x < 5 Print[\"Less than 5\"]] [Print[\"Between 5 and 10\"]]]".to_string());
        let expr = parser.parse().unwrap();
        
        match expr {
            Expression::Cond { conditions, default_statements } => {
                assert_eq!(conditions.len(), 2);
                
                // Check first condition
                match &conditions[0] {
                    (condition, statements) => {
                        match condition {
                            Expression::BinaryOp { left, operator: _, right } => {
                                match **left {
                                    Expression::Identifier(ref name) => assert_eq!(name, "x"),
                                    _ => panic!("Expected x identifier"),
                                }
                            }
                            _ => panic!("Expected binary operation"),
                        }
                        
                        match statements {
                            Expression::FunctionCall { function, arguments } => {
                                match **function {
                                    Expression::Identifier(ref name) => assert_eq!(name, "Print"),
                                    _ => panic!("Expected Print function"),
                                }
                                assert_eq!(arguments.len(), 1);
                                match arguments[0] {
                                    Expression::String(ref msg) => assert_eq!(msg, "Greater than 10"),
                                    _ => panic!("Expected string argument"),
                                }
                            }
                            _ => panic!("Expected function call"),
                        }
                    }
                }
                
                // Check second condition
                match &conditions[1] {
                    (condition, statements) => {
                        match condition {
                            Expression::BinaryOp { left, operator: _, right } => {
                                match **left {
                                    Expression::Identifier(ref name) => assert_eq!(name, "x"),
                                    _ => panic!("Expected x identifier"),
                                }
                            }
                            _ => panic!("Expected binary operation"),
                        }
                        
                        match statements {
                            Expression::FunctionCall { function, arguments } => {
                                match **function {
                                    Expression::Identifier(ref name) => assert_eq!(name, "Print"),
                                    _ => panic!("Expected Print function"),
                                }
                                assert_eq!(arguments.len(), 1);
                                match arguments[0] {
                                    Expression::String(ref msg) => assert_eq!(msg, "Less than 5"),
                                    _ => panic!("Expected string argument"),
                                }
                            }
                            _ => panic!("Expected function call"),
                        }
                    }
                }
                
                // Check default statements
                assert!(default_statements.is_some());
                match *default_statements.unwrap() {
                    Expression::FunctionCall { function, arguments } => {
                        match *function {
                            Expression::Identifier(name) => assert_eq!(name, "Print"),
                            _ => panic!("Expected Print function"),
                        }
                        assert_eq!(arguments.len(), 1);
                        match arguments[0] {
                            Expression::String(ref msg) => assert_eq!(msg, "Between 5 and 10"),
                            _ => panic!("Expected string argument"),
                        }
                    }
                    _ => panic!("Expected function call"),
                }
            }
            _ => panic!("Expected Cond expression"),
        }
    }

    #[test]
    fn test_cond_with_numeric_conditions() {
        let mut parser = Parser::new("Cond[[42 Print[\"The answer\"]] [0 Print[\"Zero\"]]]".to_string());
        let expr = parser.parse().unwrap();
        
        match expr {
            Expression::Cond { conditions, default_statements } => {
                assert_eq!(conditions.len(), 2);
                
                // Check first condition
                match &conditions[0] {
                    (condition, statements) => {
                        match condition {
                            Expression::Number(num) => assert_eq!(*num, 42),
                            _ => panic!("Expected number"),
                        }
                        
                        match statements {
                            Expression::FunctionCall { function, arguments } => {
                                match **function {
                                    Expression::Identifier(ref name) => assert_eq!(name, "Print"),
                                    _ => panic!("Expected Print function"),
                                }
                                assert_eq!(arguments.len(), 1);
                                match arguments[0] {
                                    Expression::String(ref msg) => assert_eq!(msg, "The answer"),
                                    _ => panic!("Expected string argument"),
                                }
                            }
                            _ => panic!("Expected function call"),
                        }
                    }
                }
                
                // Check second condition
                match &conditions[1] {
                    (condition, statements) => {
                        match condition {
                            Expression::Number(num) => assert_eq!(*num, 0),
                            _ => panic!("Expected number"),
                        }
                        
                        match statements {
                            Expression::FunctionCall { function, arguments } => {
                                match **function {
                                    Expression::Identifier(ref name) => assert_eq!(name, "Print"),
                                    _ => panic!("Expected Print function"),
                                }
                                assert_eq!(arguments.len(), 1);
                                match arguments[0] {
                                    Expression::String(ref msg) => assert_eq!(msg, "Zero"),
                                    _ => panic!("Expected string argument"),
                                }
                            }
                            _ => panic!("Expected function call"),
                        }
                    }
                }
                
                assert!(default_statements.is_none());
            }
            _ => panic!("Expected Cond expression"),
        }
    }
}
