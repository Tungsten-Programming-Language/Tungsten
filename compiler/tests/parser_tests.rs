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
        let expr = parser.parse_expression().unwrap();

        match expr {
            Expression::Cond {
                conditions,
                default_statements,
            } => {
                assert_eq!(conditions.len(), 1);
                assert!(default_statements.is_none());

                // Check the condition
                match &conditions[0] {
                    (condition, statements) => {
                        match condition {
                            Expression::BinaryOp {
                                left,
                                operator: _,
                                right: _,
                            } => match **left {
                                Expression::Identifier(ref name) => assert_eq!(name, "x"),
                                _ => panic!("Expected x identifier"),
                            },
                            _ => panic!("Expected binary operation"),
                        }

                        match statements {
                            Expression::FunctionCall {
                                function,
                                arguments,
                            } => {
                                match **function {
                                    Expression::Identifier(ref name) => assert_eq!(name, "Print"),
                                    _ => panic!("Expected Print function"),
                                }
                                assert_eq!(arguments.len(), 1);
                                match arguments[0] {
                                    Expression::String(ref msg) => {
                                        assert_eq!(msg, "Greater than 10")
                                    }
                                    _ => panic!("Expected string argument"),
                                }
                            }
                            _ => panic!("Expected function call"),
                        }
                    }
                }
            }
            _ => panic!("Expected Cond expression"),
        }
    }

    #[test]
    fn test_cond_multiple_conditions() {
        let mut parser = Parser::new("Cond[[x > 10 Print[\"Greater than 10\"]] [x < 5 Print[\"Less than 5\"]] [Print[\"Between 5 and 10\"]]]".to_string());
        let expr = parser.parse_expression().unwrap();

        match expr {
            Expression::Cond {
                conditions,
                default_statements,
            } => {
                assert_eq!(conditions.len(), 2);

                // Check first condition
                match &conditions[0] {
                    (condition, statements) => {
                        match condition {
                            Expression::BinaryOp {
                                left,
                                operator: _,
                                right: _,
                            } => match **left {
                                Expression::Identifier(ref name) => assert_eq!(name, "x"),
                                _ => panic!("Expected x identifier"),
                            },
                            _ => panic!("Expected binary operation"),
                        }

                        match statements {
                            Expression::FunctionCall {
                                function,
                                arguments,
                            } => {
                                match **function {
                                    Expression::Identifier(ref name) => assert_eq!(name, "Print"),
                                    _ => panic!("Expected Print function"),
                                }
                                assert_eq!(arguments.len(), 1);
                                match arguments[0] {
                                    Expression::String(ref msg) => {
                                        assert_eq!(msg, "Greater than 10")
                                    }
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
                            Expression::BinaryOp {
                                left,
                                operator: _,
                                right: _,
                            } => match **left {
                                Expression::Identifier(ref name) => assert_eq!(name, "x"),
                                _ => panic!("Expected x identifier"),
                            },
                            _ => panic!("Expected binary operation"),
                        }

                        match statements {
                            Expression::FunctionCall {
                                function,
                                arguments,
                            } => {
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
                    Expression::FunctionCall {
                        function,
                        arguments,
                    } => {
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
        let mut parser =
            Parser::new("Cond[[42 Print[\"The answer\"]] [0 Print[\"Zero\"]]]".to_string());
        let expr = parser.parse_expression().unwrap();

        match expr {
            Expression::Cond {
                conditions,
                default_statements,
            } => {
                assert_eq!(conditions.len(), 2);

                // Check first condition
                match &conditions[0] {
                    (condition, statements) => {
                        match condition {
                            Expression::Number(num) => assert_eq!(*num, 42),
                            _ => panic!("Expected number"),
                        }

                        match statements {
                            Expression::FunctionCall {
                                function,
                                arguments,
                            } => {
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
                            Expression::FunctionCall {
                                function,
                                arguments,
                            } => {
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

    #[test]
    fn test_operator_precedence_multiplication_before_addition() {
        let mut parser = Parser::new("2 + 3 * 4".to_string());
        let expr = parser.parse().unwrap();

        match expr {
            Expression::BinaryOp {
                left,
                operator: w::ast::Operator::Add,
                right,
            } => {
                match *left {
                    Expression::Number(n) => assert_eq!(n, 2),
                    _ => panic!("Expected number 2"),
                }
                match *right {
                    Expression::BinaryOp {
                        left,
                        operator: w::ast::Operator::Multiply,
                        right,
                    } => {
                        match *left {
                            Expression::Number(n) => assert_eq!(n, 3),
                            _ => panic!("Expected number 3"),
                        }
                        match *right {
                            Expression::Number(n) => assert_eq!(n, 4),
                            _ => panic!("Expected number 4"),
                        }
                    }
                    _ => panic!("Expected multiplication"),
                }
            }
            _ => panic!("Expected addition"),
        }
    }

    #[test]
    fn test_operator_precedence_division_before_subtraction() {
        let mut parser = Parser::new("10 - 6 / 2".to_string());
        let expr = parser.parse().unwrap();

        match expr {
            Expression::BinaryOp {
                left,
                operator: w::ast::Operator::Subtract,
                right,
            } => {
                match *left {
                    Expression::Number(n) => assert_eq!(n, 10),
                    _ => panic!("Expected number 10"),
                }
                match *right {
                    Expression::BinaryOp {
                        operator: w::ast::Operator::Divide,
                        ..
                    } => {}
                    _ => panic!("Expected division"),
                }
            }
            _ => panic!("Expected subtraction"),
        }
    }

    #[test]
    fn test_operator_precedence_power_before_multiply() {
        let mut parser = Parser::new("2 * 3 ^ 4".to_string());
        let expr = parser.parse().unwrap();

        match expr {
            Expression::BinaryOp {
                left,
                operator: w::ast::Operator::Multiply,
                right,
            } => {
                match *left {
                    Expression::Number(n) => assert_eq!(n, 2),
                    _ => panic!("Expected number 2"),
                }
                match *right {
                    Expression::BinaryOp {
                        operator: w::ast::Operator::Power,
                        ..
                    } => {}
                    _ => panic!("Expected power"),
                }
            }
            _ => panic!("Expected multiplication"),
        }
    }

    #[test]
    fn test_operator_precedence_power_right_associative() {
        let mut parser = Parser::new("2 ^ 3 ^ 4".to_string());
        let expr = parser.parse().unwrap();

        match expr {
            Expression::BinaryOp {
                left,
                operator: w::ast::Operator::Power,
                right,
            } => {
                match *left {
                    Expression::Number(n) => assert_eq!(n, 2),
                    _ => panic!("Expected number 2"),
                }
                match *right {
                    Expression::BinaryOp {
                        operator: w::ast::Operator::Power,
                        ..
                    } => {}
                    _ => panic!("Expected nested power"),
                }
            }
            _ => panic!("Expected power"),
        }
    }

    #[test]
    fn test_operator_precedence_left_associative_multiply() {
        let mut parser = Parser::new("8 / 4 / 2".to_string());
        let expr = parser.parse().unwrap();

        match expr {
            Expression::BinaryOp {
                left,
                operator: w::ast::Operator::Divide,
                right,
            } => {
                match *left {
                    Expression::BinaryOp {
                        operator: w::ast::Operator::Divide,
                        ..
                    } => {}
                    _ => panic!("Expected left division to be grouped first"),
                }
                match *right {
                    Expression::Number(n) => assert_eq!(n, 2),
                    _ => panic!("Expected number 2"),
                }
            }
            _ => panic!("Expected division"),
        }
    }

    #[test]
    fn test_operator_precedence_comparison_lowest() {
        let mut parser = Parser::new("2 + 3 < 4 * 5".to_string());
        let expr = parser.parse().unwrap();

        match expr {
            Expression::BinaryOp {
                left,
                operator: w::ast::Operator::LessThan,
                right,
            } => {
                match *left {
                    Expression::BinaryOp {
                        operator: w::ast::Operator::Add,
                        ..
                    } => {}
                    _ => panic!("Expected addition on left"),
                }
                match *right {
                    Expression::BinaryOp {
                        operator: w::ast::Operator::Multiply,
                        ..
                    } => {}
                    _ => panic!("Expected multiplication on right"),
                }
            }
            _ => panic!("Expected less than"),
        }
    }
}
