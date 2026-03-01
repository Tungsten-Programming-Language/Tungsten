use w::lexer::{Lexer, Token};
use w::ast::{Expression, Operator};
use w::parser::Parser;

// ── Lexer tests ──

#[test]
fn test_pipe_token() {
    let mut lexer = Lexer::new("x |> F[y]".to_string());

    assert_eq!(lexer.next_token(), Some(Token::Identifier("x".to_string())));
    assert_eq!(lexer.next_token(), Some(Token::Pipe));
    assert_eq!(lexer.next_token(), Some(Token::Identifier("F".to_string())));
    assert_eq!(lexer.next_token(), Some(Token::LeftBracket));
    assert_eq!(lexer.next_token(), Some(Token::Identifier("y".to_string())));
    assert_eq!(lexer.next_token(), Some(Token::RightBracket));
    assert_eq!(lexer.next_token(), None);
}

#[test]
fn test_pipe_token_no_spaces() {
    let mut lexer = Lexer::new("x|>F".to_string());

    assert_eq!(lexer.next_token(), Some(Token::Identifier("x".to_string())));
    assert_eq!(lexer.next_token(), Some(Token::Pipe));
    assert_eq!(lexer.next_token(), Some(Token::Identifier("F".to_string())));
    assert_eq!(lexer.next_token(), None);
}

// ── Parser tests ──

#[test]
fn test_pipe_simple() {
    // x |> F[y]  →  F[y, x]
    let mut parser = Parser::new("x |> F[y]".to_string());
    let expr = parser.parse().unwrap();

    match expr {
        Expression::FunctionCall { function, arguments } => {
            assert_eq!(*function, Expression::Identifier("F".to_string()));
            assert_eq!(arguments.len(), 2);
            assert_eq!(arguments[0], Expression::Identifier("y".to_string()));
            assert_eq!(arguments[1], Expression::Identifier("x".to_string()));
        }
        _ => panic!("Expected FunctionCall, got {:?}", expr),
    }
}

#[test]
fn test_pipe_bare_function() {
    // x |> F  →  F[x]
    let mut parser = Parser::new("x |> F".to_string());
    let expr = parser.parse().unwrap();

    match expr {
        Expression::FunctionCall { function, arguments } => {
            assert_eq!(*function, Expression::Identifier("F".to_string()));
            assert_eq!(arguments.len(), 1);
            assert_eq!(arguments[0], Expression::Identifier("x".to_string()));
        }
        _ => panic!("Expected FunctionCall, got {:?}", expr),
    }
}

#[test]
fn test_pipe_chained() {
    // x |> F[y] |> G[z]  →  G[z, F[y, x]]
    let mut parser = Parser::new("x |> F[y] |> G[z]".to_string());
    let expr = parser.parse().unwrap();

    match expr {
        Expression::FunctionCall { function, arguments } => {
            assert_eq!(*function, Expression::Identifier("G".to_string()));
            assert_eq!(arguments.len(), 2);
            assert_eq!(arguments[0], Expression::Identifier("z".to_string()));
            // Second argument should be F[y, x]
            match &arguments[1] {
                Expression::FunctionCall { function: inner_fn, arguments: inner_args } => {
                    assert_eq!(**inner_fn, Expression::Identifier("F".to_string()));
                    assert_eq!(inner_args.len(), 2);
                    assert_eq!(inner_args[0], Expression::Identifier("y".to_string()));
                    assert_eq!(inner_args[1], Expression::Identifier("x".to_string()));
                }
                other => panic!("Expected inner FunctionCall, got {:?}", other),
            }
        }
        _ => panic!("Expected FunctionCall, got {:?}", expr),
    }
}

#[test]
fn test_pipe_with_binary_op_lhs() {
    // 1 + 2 |> F  →  F[1 + 2]
    let mut parser = Parser::new("1 + 2 |> F".to_string());
    let expr = parser.parse().unwrap();

    match expr {
        Expression::FunctionCall { function, arguments } => {
            assert_eq!(*function, Expression::Identifier("F".to_string()));
            assert_eq!(arguments.len(), 1);
            match &arguments[0] {
                Expression::BinaryOp { left, operator, right } => {
                    assert_eq!(**left, Expression::Number(1));
                    assert_eq!(*operator, Operator::Add);
                    assert_eq!(**right, Expression::Number(2));
                }
                other => panic!("Expected BinaryOp, got {:?}", other),
            }
        }
        _ => panic!("Expected FunctionCall, got {:?}", expr),
    }
}

#[test]
fn test_pipe_chained_bare_functions() {
    // x |> F |> G |> H  →  H[G[F[x]]]
    let mut parser = Parser::new("x |> F |> G |> H".to_string());
    let expr = parser.parse().unwrap();

    // Outermost: H[...]
    match expr {
        Expression::FunctionCall { function, arguments } => {
            assert_eq!(*function, Expression::Identifier("H".to_string()));
            assert_eq!(arguments.len(), 1);
            // G[...]
            match &arguments[0] {
                Expression::FunctionCall { function: g_fn, arguments: g_args } => {
                    assert_eq!(**g_fn, Expression::Identifier("G".to_string()));
                    assert_eq!(g_args.len(), 1);
                    // F[x]
                    match &g_args[0] {
                        Expression::FunctionCall { function: f_fn, arguments: f_args } => {
                            assert_eq!(**f_fn, Expression::Identifier("F".to_string()));
                            assert_eq!(f_args.len(), 1);
                            assert_eq!(f_args[0], Expression::Identifier("x".to_string()));
                        }
                        other => panic!("Expected F call, got {:?}", other),
                    }
                }
                other => panic!("Expected G call, got {:?}", other),
            }
        }
        _ => panic!("Expected FunctionCall, got {:?}", expr),
    }
}
