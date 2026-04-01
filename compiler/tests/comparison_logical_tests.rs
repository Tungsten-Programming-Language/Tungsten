use w::ast::{Expression, Operator};
use w::lexer::{Lexer, Token};
use w::parser::Parser;
use w::rust_codegen::RustCodeGenerator;

// ==================== Lexer Tests ====================

#[test]
fn test_less_equal_token() {
    let mut lexer = Lexer::new("<=".to_string());
    assert_eq!(lexer.next_token(), Some(Token::LessEqual));
    assert_eq!(lexer.next_token(), None);
}

#[test]
fn test_greater_equal_token() {
    let mut lexer = Lexer::new(">=".to_string());
    assert_eq!(lexer.next_token(), Some(Token::GreaterEqual));
    assert_eq!(lexer.next_token(), None);
}

#[test]
fn test_and_token() {
    let mut lexer = Lexer::new("&&".to_string());
    assert_eq!(lexer.next_token(), Some(Token::And));
    assert_eq!(lexer.next_token(), None);
}

#[test]
fn test_less_than_vs_less_equal() {
    let mut lexer = Lexer::new("< <=".to_string());
    assert_eq!(lexer.next_token(), Some(Token::LessThan));
    assert_eq!(lexer.next_token(), Some(Token::LessEqual));
    assert_eq!(lexer.next_token(), None);
}

#[test]
fn test_greater_than_vs_greater_equal() {
    let mut lexer = Lexer::new("> >=".to_string());
    assert_eq!(lexer.next_token(), Some(Token::GreaterThan));
    assert_eq!(lexer.next_token(), Some(Token::GreaterEqual));
    assert_eq!(lexer.next_token(), None);
}

#[test]
fn test_and_in_expression() {
    let mut lexer = Lexer::new("x && y".to_string());
    assert_eq!(lexer.next_token(), Some(Token::Identifier("x".to_string())));
    assert_eq!(lexer.next_token(), Some(Token::And));
    assert_eq!(lexer.next_token(), Some(Token::Identifier("y".to_string())));
    assert_eq!(lexer.next_token(), None);
}

#[test]
fn test_single_ampersand_invalid() {
    let mut lexer = Lexer::new("&".to_string());
    assert_eq!(lexer.next_token(), None);
}

// ==================== Parser Tests ====================

#[test]
fn test_parse_less_equal() {
    let mut parser = Parser::new("x <= y".to_string());
    let expr = parser.parse_expression().unwrap();

    match expr {
        Expression::BinaryOp {
            left,
            operator: Operator::LessEqual,
            right,
        } => match (*left, *right) {
            (Expression::Identifier(l), Expression::Identifier(r)) => {
                assert_eq!(l, "x");
                assert_eq!(r, "y");
            }
            _ => panic!("Expected identifiers"),
        },
        other => panic!("Expected BinaryOp with LessEqual, got {:?}", other),
    }
}

#[test]
fn test_parse_greater_equal() {
    let mut parser = Parser::new("x >= y".to_string());
    let expr = parser.parse_expression().unwrap();

    match expr {
        Expression::BinaryOp {
            left,
            operator: Operator::GreaterEqual,
            right,
        } => match (*left, *right) {
            (Expression::Identifier(l), Expression::Identifier(r)) => {
                assert_eq!(l, "x");
                assert_eq!(r, "y");
            }
            _ => panic!("Expected identifiers"),
        },
        other => panic!("Expected BinaryOp with GreaterEqual, got {:?}", other),
    }
}

#[test]
fn test_parse_and() {
    let mut parser = Parser::new("x && y".to_string());
    let expr = parser.parse_expression().unwrap();

    match expr {
        Expression::BinaryOp {
            left,
            operator: Operator::And,
            right,
        } => match (*left, *right) {
            (Expression::Identifier(l), Expression::Identifier(r)) => {
                assert_eq!(l, "x");
                assert_eq!(r, "y");
            }
            _ => panic!("Expected identifiers"),
        },
        other => panic!("Expected BinaryOp with And, got {:?}", other),
    }
}

#[test]
fn test_and_lower_precedence_than_comparison() {
    let mut parser = Parser::new("a < b && c > d".to_string());
    let expr = parser.parse_expression().unwrap();

    match expr {
        Expression::BinaryOp {
            left,
            operator: Operator::And,
            right,
        } => match (*left, *right) {
            (
                Expression::BinaryOp {
                    operator: Operator::LessThan,
                    ..
                },
                Expression::BinaryOp {
                    operator: Operator::GreaterThan,
                    ..
                },
            ) => {}
            _ => panic!("Expected comparison operators as operands"),
        },
        other => panic!("Expected And at top level, got {:?}", other),
    }
}

#[test]
fn test_comparison_lower_precedence_than_arithmetic() {
    let mut parser = Parser::new("1 + 2 <= 3 * 4".to_string());
    let expr = parser.parse_expression().unwrap();

    match expr {
        Expression::BinaryOp {
            left,
            operator: Operator::LessEqual,
            right,
        } => match (*left, *right) {
            (
                Expression::BinaryOp {
                    operator: Operator::Add,
                    ..
                },
                Expression::BinaryOp {
                    operator: Operator::Multiply,
                    ..
                },
            ) => {}
            _ => panic!("Expected arithmetic operators as operands"),
        },
        other => panic!("Expected LessEqual at top level, got {:?}", other),
    }
}

#[test]
fn test_chained_and() {
    let mut parser = Parser::new("a && b && c".to_string());
    let expr = parser.parse_expression().unwrap();

    match expr {
        Expression::BinaryOp {
            left,
            operator: Operator::And,
            right,
        } => {
            match *left {
                Expression::BinaryOp {
                    operator: Operator::And,
                    ..
                } => {}
                _ => panic!("Expected chained And on left"),
            }
            match *right {
                Expression::Identifier(name) => assert_eq!(name, "c"),
                _ => panic!("Expected identifier c"),
            }
        }
        other => panic!("Expected And, got {:?}", other),
    }
}

// ==================== Code Generation Tests ====================

#[test]
fn test_codegen_less_equal() {
    let mut parser = Parser::new("x <= y".to_string());
    let expr = parser.parse().unwrap();
    let mut codegen = RustCodeGenerator::new();
    let rust_code = codegen.generate(&expr).unwrap();
    assert!(rust_code.contains("<="));
}

#[test]
fn test_codegen_greater_equal() {
    let mut parser = Parser::new("x >= y".to_string());
    let expr = parser.parse().unwrap();
    let mut codegen = RustCodeGenerator::new();
    let rust_code = codegen.generate(&expr).unwrap();
    assert!(rust_code.contains(">="));
}

#[test]
fn test_codegen_and() {
    let mut parser = Parser::new("x && y".to_string());
    let expr = parser.parse().unwrap();
    let mut codegen = RustCodeGenerator::new();
    let rust_code = codegen.generate(&expr).unwrap();
    assert!(rust_code.contains("&&"));
}

#[test]
fn test_codegen_complex_expression() {
    let mut parser = Parser::new("a < b && c >= d".to_string());
    let expr = parser.parse().unwrap();
    let mut codegen = RustCodeGenerator::new();
    let rust_code = codegen.generate(&expr).unwrap();
    assert!(rust_code.contains("<"));
    assert!(rust_code.contains("&&"));
    assert!(rust_code.contains(">="));
}

#[test]
fn test_codegen_comparison_with_arithmetic() {
    let mut parser = Parser::new("1 + 2 <= 3 * 4".to_string());
    let expr = parser.parse().unwrap();
    let mut codegen = RustCodeGenerator::new();
    let rust_code = codegen.generate(&expr).unwrap();
    assert!(rust_code.contains("<="));
    assert!(rust_code.contains("+"));
    assert!(rust_code.contains("*"));
}

// Tests for operator precedence fixes

#[test]
fn test_unary_minus_number() {
    let mut parser = Parser::new("-5".to_string());
    let expr = parser.parse_expression().unwrap();
    match expr {
        Expression::BinaryOp {
            left,
            operator: Operator::Subtract,
            right,
        } => {
            match *left {
                Expression::Number(n) => assert_eq!(n, 0),
                _ => panic!("Expected 0 as left operand"),
            }
            match *right {
                Expression::Number(n) => assert_eq!(n, 5),
                _ => panic!("Expected 5 as right operand"),
            }
        }
        _ => panic!("Expected BinaryOp with Subtract"),
    }
}

#[test]
fn test_unary_minus_with_power() {
    let mut parser = Parser::new("-2 ^ 3".to_string());
    let expr = parser.parse_expression().unwrap();
    match expr {
        Expression::BinaryOp {
            operator: Operator::Subtract,
            right,
            ..
        } => match *right {
            Expression::BinaryOp {
                operator: Operator::Power,
                ..
            } => {}
            _ => panic!("Expected power on right side"),
        },
        _ => panic!("Expected Subtract at top level"),
    }
}

#[test]
fn test_power_after_function_call() {
    let mut parser = Parser::new("f[x] ^ 2".to_string());
    let expr = parser.parse_expression().unwrap();
    match expr {
        Expression::BinaryOp {
            left,
            operator: Operator::Power,
            right,
        } => {
            match *left {
                Expression::FunctionCall { .. } => {}
                _ => panic!("Expected function call on left"),
            }
            match *right {
                Expression::Number(n) => assert_eq!(n, 2),
                _ => panic!("Expected number 2 on right"),
            }
        }
        _ => panic!("Expected BinaryOp with Power"),
    }
}

#[test]
fn test_operator_chaining_after_function_call() {
    let mut parser = Parser::new("f[x] + 1 && g[y]".to_string());
    let expr = parser.parse_expression().unwrap();
    match expr {
        Expression::BinaryOp {
            left,
            operator: Operator::And,
            right,
        } => {
            match *left {
                Expression::BinaryOp {
                    operator: Operator::Add,
                    ..
                } => {}
                _ => panic!("Expected Add on left side of &&"),
            }
            match *right {
                Expression::FunctionCall { .. } => {}
                _ => panic!("Expected function call on right side of &&"),
            }
        }
        _ => panic!("Expected And at top level"),
    }
}

#[test]
fn test_codegen_unary_minus() {
    let mut parser = Parser::new("-42".to_string());
    let expr = parser.parse().unwrap();
    let mut codegen = RustCodeGenerator::new();
    let rust_code = codegen.generate(&expr).unwrap();
    assert!(rust_code.contains("0 - 42"));
}

#[test]
fn test_codegen_power_after_function_call() {
    let mut parser = Parser::new("f[x] ^ 2".to_string());
    let expr = parser.parse().unwrap();
    let mut codegen = RustCodeGenerator::new();
    let rust_code = codegen.generate(&expr).unwrap();
    assert!(rust_code.contains(".pow("));
}
