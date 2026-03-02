use w::lexer::{Lexer, Token};
use w::parser::Parser;
use w::ast::Expression;
use w::rust_codegen::RustCodeGenerator;

// ==================== Lexer Tests ====================

#[test]
fn test_question_token() {
    let mut lexer = Lexer::new("?".to_string());
    assert_eq!(lexer.next_token(), Some(Token::Question));
    assert_eq!(lexer.next_token(), None);
}

#[test]
fn test_question_after_identifier() {
    let mut lexer = Lexer::new("x?".to_string());
    assert_eq!(lexer.next_token(), Some(Token::Identifier("x".to_string())));
    assert_eq!(lexer.next_token(), Some(Token::Question));
    assert_eq!(lexer.next_token(), None);
}

#[test]
fn test_question_after_function_call() {
    let mut lexer = Lexer::new("F[x]?".to_string());
    assert_eq!(lexer.next_token(), Some(Token::Identifier("F".to_string())));
    assert_eq!(lexer.next_token(), Some(Token::LeftBracket));
    assert_eq!(lexer.next_token(), Some(Token::Identifier("x".to_string())));
    assert_eq!(lexer.next_token(), Some(Token::RightBracket));
    assert_eq!(lexer.next_token(), Some(Token::Question));
    assert_eq!(lexer.next_token(), None);
}

#[test]
fn test_question_with_spaces() {
    let mut lexer = Lexer::new("x ?".to_string());
    assert_eq!(lexer.next_token(), Some(Token::Identifier("x".to_string())));
    assert_eq!(lexer.next_token(), Some(Token::Question));
}

// ==================== Parser Tests ====================

#[test]
fn test_parse_simple_propagate() {
    let mut parser = Parser::new("x?".to_string());
    let expr = parser.parse_expression().unwrap();

    match expr {
        Expression::Propagate { expr } => {
            match *expr {
                Expression::Identifier(name) => assert_eq!(name, "x"),
                other => panic!("Expected Identifier, got {:?}", other),
            }
        }
        other => panic!("Expected Propagate, got {:?}", other),
    }
}

#[test]
fn test_parse_function_call_propagate() {
    let mut parser = Parser::new("F[x]?".to_string());
    let expr = parser.parse_expression().unwrap();

    match expr {
        Expression::Propagate { expr } => {
            match *expr {
                Expression::FunctionCall { function, arguments } => {
                    match *function {
                        Expression::Identifier(name) => assert_eq!(name, "F"),
                        other => panic!("Expected Identifier, got {:?}", other),
                    }
                    assert_eq!(arguments.len(), 1);
                }
                other => panic!("Expected FunctionCall, got {:?}", other),
            }
        }
        other => panic!("Expected Propagate, got {:?}", other),
    }
}

#[test]
fn test_parse_propagate_binds_tighter_than_binary_op() {
    // 1 + x? should parse as 1 + (x?) not (1 + x)?
    let mut parser = Parser::new("1 + x?".to_string());
    let expr = parser.parse_expression().unwrap();

    match expr {
        Expression::BinaryOp { left, operator: _, right } => {
            match *left {
                Expression::Number(1) => {},
                other => panic!("Expected Number(1), got {:?}", other),
            }
            match *right {
                Expression::Propagate { expr } => {
                    match *expr {
                        Expression::Identifier(name) => assert_eq!(name, "x"),
                        other => panic!("Expected Identifier, got {:?}", other),
                    }
                }
                other => panic!("Expected Propagate, got {:?}", other),
            }
        }
        other => panic!("Expected BinaryOp, got {:?}", other),
    }
}

#[test]
fn test_parse_propagate_with_pipe() {
    // x |> F? should become Propagate(FunctionCall(F, [x]))
    let mut parser = Parser::new("x |> F?".to_string());
    let expr = parser.parse_expression().unwrap();

    match expr {
        Expression::Propagate { expr } => {
            match *expr {
                Expression::FunctionCall { function, arguments } => {
                    match *function {
                        Expression::Identifier(name) => assert_eq!(name, "F"),
                        other => panic!("Expected Identifier(F), got {:?}", other),
                    }
                    assert_eq!(arguments.len(), 1);
                    match &arguments[0] {
                        Expression::Identifier(name) => assert_eq!(name, "x"),
                        other => panic!("Expected Identifier(x), got {:?}", other),
                    }
                }
                other => panic!("Expected FunctionCall, got {:?}", other),
            }
        }
        other => panic!("Expected Propagate, got {:?}", other),
    }
}

#[test]
fn test_parse_chained_pipe_with_propagate() {
    // A[x]? |> B? |> C? should become:
    // Propagate(FunctionCall(C, [Propagate(FunctionCall(B, [Propagate(FunctionCall(A, [x]))]))]))
    let mut parser = Parser::new("A[x]? |> B? |> C?".to_string());
    let expr = parser.parse_expression().unwrap();

    // Outermost should be Propagate { FunctionCall(C, [...]) }
    match expr {
        Expression::Propagate { expr: c_prop } => {
            match *c_prop {
                Expression::FunctionCall { function: c_fn, arguments: c_args } => {
                    match *c_fn {
                        Expression::Identifier(name) => assert_eq!(name, "C"),
                        other => panic!("Expected C, got {:?}", other),
                    }
                    assert_eq!(c_args.len(), 1);

                    // Inner should be Propagate { FunctionCall(B, [...]) }
                    match &c_args[0] {
                        Expression::Propagate { expr: b_prop } => {
                            match b_prop.as_ref() {
                                Expression::FunctionCall { function: b_fn, arguments: b_args } => {
                                    match b_fn.as_ref() {
                                        Expression::Identifier(name) => assert_eq!(name, "B"),
                                        other => panic!("Expected B, got {:?}", other),
                                    }
                                    assert_eq!(b_args.len(), 1);

                                    // Innermost should be Propagate { FunctionCall(A, [x]) }
                                    match &b_args[0] {
                                        Expression::Propagate { expr: a_prop } => {
                                            match a_prop.as_ref() {
                                                Expression::FunctionCall { function: a_fn, arguments: a_args } => {
                                                    match a_fn.as_ref() {
                                                        Expression::Identifier(name) => assert_eq!(name, "A"),
                                                        other => panic!("Expected A, got {:?}", other),
                                                    }
                                                    assert_eq!(a_args.len(), 1);
                                                    match &a_args[0] {
                                                        Expression::Identifier(name) => assert_eq!(name, "x"),
                                                        other => panic!("Expected x, got {:?}", other),
                                                    }
                                                }
                                                other => panic!("Expected FunctionCall(A), got {:?}", other),
                                            }
                                        }
                                        other => panic!("Expected Propagate(A[x]), got {:?}", other),
                                    }
                                }
                                other => panic!("Expected FunctionCall(B), got {:?}", other),
                            }
                        }
                        other => panic!("Expected Propagate(B[...]), got {:?}", other),
                    }
                }
                other => panic!("Expected FunctionCall(C), got {:?}", other),
            }
        }
        other => panic!("Expected Propagate at top level, got {:?}", other),
    }
}

#[test]
fn test_parse_pipe_with_function_call_propagate() {
    // x |> F[y]? should become Propagate(FunctionCall(F, [y, x]))
    let mut parser = Parser::new("x |> F[y]?".to_string());
    let expr = parser.parse_expression().unwrap();

    match expr {
        Expression::Propagate { expr } => {
            match *expr {
                Expression::FunctionCall { function, arguments } => {
                    match *function {
                        Expression::Identifier(name) => assert_eq!(name, "F"),
                        other => panic!("Expected Identifier(F), got {:?}", other),
                    }
                    assert_eq!(arguments.len(), 2);
                    match &arguments[0] {
                        Expression::Identifier(name) => assert_eq!(name, "y"),
                        other => panic!("Expected Identifier(y), got {:?}", other),
                    }
                    match &arguments[1] {
                        Expression::Identifier(name) => assert_eq!(name, "x"),
                        other => panic!("Expected Identifier(x), got {:?}", other),
                    }
                }
                other => panic!("Expected FunctionCall, got {:?}", other),
            }
        }
        other => panic!("Expected Propagate, got {:?}", other),
    }
}

// ==================== Code Generation Tests ====================

#[test]
fn test_codegen_simple_propagate() {
    let mut parser = Parser::new("x?".to_string());
    let expr = parser.parse_expression().unwrap();

    let mut codegen = RustCodeGenerator::new();
    let rust_code = codegen.generate(&expr).unwrap();

    assert!(rust_code.contains("(x)?"),
        "Should generate (x)?, got: {}", rust_code);
}

#[test]
fn test_codegen_function_call_propagate() {
    let mut parser = Parser::new("F[x]?".to_string());
    let expr = parser.parse_expression().unwrap();

    let mut codegen = RustCodeGenerator::new();
    let rust_code = codegen.generate(&expr).unwrap();

    assert!(rust_code.contains("(f(x))?"),
        "Should generate (f(x))?, got: {}", rust_code);
}

#[test]
fn test_codegen_propagate_with_some() {
    let mut parser = Parser::new("Some[42]?".to_string());
    let expr = parser.parse_expression().unwrap();

    let mut codegen = RustCodeGenerator::new();
    let rust_code = codegen.generate(&expr).unwrap();

    assert!(rust_code.contains("(Some(42))?"),
        "Should generate (Some(42))?, got: {}", rust_code);
}

#[test]
fn test_codegen_propagate_with_ok() {
    let mut parser = Parser::new("Ok[100]?".to_string());
    let expr = parser.parse_expression().unwrap();

    let mut codegen = RustCodeGenerator::new();
    let rust_code = codegen.generate(&expr).unwrap();

    assert!(rust_code.contains("(Ok(100))?"),
        "Should generate (Ok(100))?, got: {}", rust_code);
}
