use w::lexer::{Lexer, Token};
use w::ast::Expression;
use w::parser::Parser;
use w::rust_codegen::RustCodeGenerator;

// ============================================
// Lexer Tests - Arrow Token
// ============================================

#[test]
fn test_arrow_token() {
    let mut lexer = Lexer::new("x -> x + 1".to_string());

    assert_eq!(lexer.next_token(), Some(Token::Identifier("x".to_string())));
    assert_eq!(lexer.next_token(), Some(Token::Arrow));
    assert_eq!(lexer.next_token(), Some(Token::Identifier("x".to_string())));
    assert_eq!(lexer.next_token(), Some(Token::Plus));
    assert_eq!(lexer.next_token(), Some(Token::Number(1)));
    assert_eq!(lexer.next_token(), None);
}

#[test]
fn test_arrow_token_no_spaces() {
    let mut lexer = Lexer::new("x->y".to_string());

    assert_eq!(lexer.next_token(), Some(Token::Identifier("x".to_string())));
    assert_eq!(lexer.next_token(), Some(Token::Arrow));
    assert_eq!(lexer.next_token(), Some(Token::Identifier("y".to_string())));
    assert_eq!(lexer.next_token(), None);
}

#[test]
fn test_minus_not_confused_with_arrow() {
    let mut lexer = Lexer::new("x - y".to_string());

    assert_eq!(lexer.next_token(), Some(Token::Identifier("x".to_string())));
    assert_eq!(lexer.next_token(), Some(Token::Minus));
    assert_eq!(lexer.next_token(), Some(Token::Identifier("y".to_string())));
    assert_eq!(lexer.next_token(), None);
}

// ============================================
// Parser Tests - Lambda Shorthand
// ============================================

#[test]
fn test_parse_arrow_lambda_simple() {
    // x -> x * 2  desugars to  Function[{x}, x * 2]
    let mut parser = Parser::new("x -> x * 2".to_string());
    let expr = parser.parse_expression().unwrap();

    match expr {
        Expression::Lambda { parameters, body: _ } => {
            assert_eq!(parameters.len(), 1);
            assert_eq!(parameters[0].name, "x");
        }
        _ => panic!("Expected Lambda expression, got {:?}", expr),
    }
}

#[test]
fn test_parse_arrow_lambda_comparison() {
    // x -> x > 100  desugars to  Function[{x}, x > 100]
    let mut parser = Parser::new("x -> x > 100".to_string());
    let expr = parser.parse_expression().unwrap();

    match expr {
        Expression::Lambda { parameters, body } => {
            assert_eq!(parameters.len(), 1);
            assert_eq!(parameters[0].name, "x");
            // Body should be a BinaryOp with >
            match *body {
                Expression::BinaryOp { left, operator, right } => {
                    assert_eq!(*left, Expression::Identifier("x".to_string()));
                    assert_eq!(operator, w::ast::Operator::GreaterThan);
                    assert_eq!(*right, Expression::Number(100));
                }
                _ => panic!("Expected BinaryOp body, got {:?}", body),
            }
        }
        _ => panic!("Expected Lambda expression, got {:?}", expr),
    }
}

#[test]
fn test_parse_arrow_lambda_in_filter() {
    // Filter[x -> x > 100]  desugars to  Filter[Function[{x}, x > 100]]
    let mut parser = Parser::new("Filter[x -> x > 100]".to_string());
    let expr = parser.parse_expression().unwrap();

    match expr {
        Expression::FunctionCall { function, arguments } => {
            assert_eq!(*function, Expression::Identifier("Filter".to_string()));
            assert_eq!(arguments.len(), 1);
            match &arguments[0] {
                Expression::Lambda { parameters, .. } => {
                    assert_eq!(parameters.len(), 1);
                    assert_eq!(parameters[0].name, "x");
                }
                _ => panic!("Expected Lambda as argument, got {:?}", arguments[0]),
            }
        }
        _ => panic!("Expected FunctionCall, got {:?}", expr),
    }
}

#[test]
fn test_parse_arrow_lambda_in_map() {
    // Map[x -> x * 2]  desugars to  Map[Function[{x}, x * 2]]
    let mut parser = Parser::new("Map[x -> x * 2]".to_string());
    let expr = parser.parse_expression().unwrap();

    match expr {
        Expression::FunctionCall { function, arguments } => {
            assert_eq!(*function, Expression::Identifier("Map".to_string()));
            assert_eq!(arguments.len(), 1);
            match &arguments[0] {
                Expression::Lambda { parameters, .. } => {
                    assert_eq!(parameters.len(), 1);
                    assert_eq!(parameters[0].name, "x");
                }
                _ => panic!("Expected Lambda as argument, got {:?}", arguments[0]),
            }
        }
        _ => panic!("Expected FunctionCall, got {:?}", expr),
    }
}

#[test]
fn test_parse_arrow_lambda_with_pipe() {
    // data |> Filter[x -> x > 100] |> Map[x -> x * 2]
    let mut parser = Parser::new("data |> Filter[x -> x > 100] |> Map[x -> x * 2]".to_string());
    let expr = parser.parse_expression().unwrap();

    // Outermost: Map[x -> x * 2, Filter[x -> x > 100, data]]
    match expr {
        Expression::FunctionCall { function, arguments } => {
            assert_eq!(*function, Expression::Identifier("Map".to_string()));
            assert_eq!(arguments.len(), 2);

            // First arg: lambda x -> x * 2
            match &arguments[0] {
                Expression::Lambda { parameters, .. } => {
                    assert_eq!(parameters[0].name, "x");
                }
                _ => panic!("Expected Lambda, got {:?}", arguments[0]),
            }

            // Second arg: Filter[x -> x > 100, data]
            match &arguments[1] {
                Expression::FunctionCall { function: inner_fn, arguments: inner_args } => {
                    assert_eq!(**inner_fn, Expression::Identifier("Filter".to_string()));
                    assert_eq!(inner_args.len(), 2);

                    // Filter's first arg: lambda x -> x > 100
                    match &inner_args[0] {
                        Expression::Lambda { parameters, .. } => {
                            assert_eq!(parameters[0].name, "x");
                        }
                        _ => panic!("Expected Lambda in Filter, got {:?}", inner_args[0]),
                    }

                    // Filter's second arg: data (piped in)
                    assert_eq!(inner_args[1], Expression::Identifier("data".to_string()));
                }
                _ => panic!("Expected inner FunctionCall, got {:?}", arguments[1]),
            }
        }
        _ => panic!("Expected FunctionCall, got {:?}", expr),
    }
}

// ============================================
// Equivalence Tests - Arrow vs Function syntax
// ============================================

#[test]
fn test_arrow_equivalent_to_function_in_map() {
    // These two should produce the same AST
    let mut parser1 = Parser::new("Map[x -> x * 2, [1, 2, 3]]".to_string());
    let expr1 = parser1.parse_expression().unwrap();

    let mut parser2 = Parser::new("Map[Function[{x}, x * 2], [1, 2, 3]]".to_string());
    let expr2 = parser2.parse_expression().unwrap();

    // Both should be FunctionCalls to Map with a Lambda as first arg
    match (&expr1, &expr2) {
        (
            Expression::FunctionCall { arguments: args1, .. },
            Expression::FunctionCall { arguments: args2, .. },
        ) => {
            assert_eq!(args1.len(), args2.len());
            // Both first args should be Lambda with param "x"
            match (&args1[0], &args2[0]) {
                (
                    Expression::Lambda { parameters: p1, .. },
                    Expression::Lambda { parameters: p2, .. },
                ) => {
                    assert_eq!(p1[0].name, p2[0].name);
                }
                _ => panic!("Expected both to have Lambda first args"),
            }
        }
        _ => panic!("Expected both to be FunctionCalls"),
    }
}

// ============================================
// Code Generation Tests - Lambda Shorthand
// ============================================

#[test]
fn test_codegen_arrow_lambda() {
    let mut parser = Parser::new("x -> x * 2".to_string());
    let expr = parser.parse_expression().unwrap();

    let mut codegen = RustCodeGenerator::new();
    let rust_code = codegen.generate(&expr).unwrap();

    assert!(rust_code.contains("|x|"),
        "Should generate Rust closure, got: {}", rust_code);
    assert!(rust_code.contains("x * 2"),
        "Should contain closure body, got: {}", rust_code);
}

#[test]
fn test_codegen_arrow_map() {
    let mut parser = Parser::new("Map[x -> x * 2, [1, 2, 3]]".to_string());
    let expr = parser.parse_expression().unwrap();

    let mut codegen = RustCodeGenerator::new();
    let rust_code = codegen.generate(&expr).unwrap();

    assert!(rust_code.contains(".into_iter().map("),
        "Should generate iterator map, got: {}", rust_code);
    assert!(rust_code.contains("|x| (x * 2)"),
        "Should inline arrow lambda in map, got: {}", rust_code);
    assert!(rust_code.contains(".collect::<Vec<_>>()"),
        "Should collect into Vec, got: {}", rust_code);
}

#[test]
fn test_codegen_arrow_filter() {
    let mut parser = Parser::new("Filter[x -> x > 5, [1, 10, 3, 8]]".to_string());
    let expr = parser.parse_expression().unwrap();

    let mut codegen = RustCodeGenerator::new();
    let rust_code = codegen.generate(&expr).unwrap();

    assert!(rust_code.contains(".into_iter().filter("),
        "Should generate iterator filter, got: {}", rust_code);
    assert!(rust_code.contains("|&x| (x > 5)"),
        "Should use pattern matching in filter, got: {}", rust_code);
}

#[test]
fn test_codegen_arrow_with_pipe() {
    let mut parser = Parser::new("[1, 2, 3] |> Map[x -> x * 2]".to_string());
    let expr = parser.parse_expression().unwrap();

    let mut codegen = RustCodeGenerator::new();
    let rust_code = codegen.generate(&expr).unwrap();

    assert!(rust_code.contains(".into_iter().map(|x| (x * 2))"),
        "Should generate piped map with arrow lambda, got: {}", rust_code);
}

#[test]
fn test_codegen_arrow_pipe_chain() {
    let input = "[1, 50, 150, 200] |> Filter[x -> x > 100] |> Map[x -> x * 2]";
    let mut parser = Parser::new(input.to_string());
    let expr = parser.parse_expression().unwrap();

    let mut codegen = RustCodeGenerator::new();
    let rust_code = codegen.generate(&expr).unwrap();

    assert!(rust_code.contains(".filter("),
        "Should contain filter, got: {}", rust_code);
    assert!(rust_code.contains(".map("),
        "Should contain map, got: {}", rust_code);
}
