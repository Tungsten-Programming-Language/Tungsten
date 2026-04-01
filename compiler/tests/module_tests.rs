use w::ast::Expression;
use w::lexer::{Lexer, Token};
use w::parser::Parser;
use w::rust_codegen::RustCodeGenerator;
use w::type_inference::TypeInference;

// ============================================================================
// Lexer Tests for Module
// ============================================================================

#[test]
fn test_module_tokens() {
    let mut lexer = Lexer::new("Module[{x = 1}, x]".to_string());

    assert_eq!(
        lexer.next_token(),
        Some(Token::Identifier("Module".to_string()))
    );
    assert_eq!(lexer.next_token(), Some(Token::LeftBracket));
    assert_eq!(lexer.next_token(), Some(Token::LeftBrace));
    assert_eq!(lexer.next_token(), Some(Token::Identifier("x".to_string())));
    assert_eq!(lexer.next_token(), Some(Token::Assign));
    assert_eq!(lexer.next_token(), Some(Token::Number(1)));
    assert_eq!(lexer.next_token(), Some(Token::RightBrace));
    assert_eq!(lexer.next_token(), Some(Token::Comma));
    assert_eq!(lexer.next_token(), Some(Token::Identifier("x".to_string())));
    assert_eq!(lexer.next_token(), Some(Token::RightBracket));
    assert_eq!(lexer.next_token(), None);
}

#[test]
fn test_module_tokens_uninitialized() {
    let mut lexer = Lexer::new("Module[{x, y}, x + y]".to_string());

    assert_eq!(
        lexer.next_token(),
        Some(Token::Identifier("Module".to_string()))
    );
    assert_eq!(lexer.next_token(), Some(Token::LeftBracket));
    assert_eq!(lexer.next_token(), Some(Token::LeftBrace));
    assert_eq!(lexer.next_token(), Some(Token::Identifier("x".to_string())));
    assert_eq!(lexer.next_token(), Some(Token::Comma));
    assert_eq!(lexer.next_token(), Some(Token::Identifier("y".to_string())));
    assert_eq!(lexer.next_token(), Some(Token::RightBrace));
    assert_eq!(lexer.next_token(), Some(Token::Comma));
    assert_eq!(lexer.next_token(), Some(Token::Identifier("x".to_string())));
    assert_eq!(lexer.next_token(), Some(Token::Plus));
    assert_eq!(lexer.next_token(), Some(Token::Identifier("y".to_string())));
    assert_eq!(lexer.next_token(), Some(Token::RightBracket));
    assert_eq!(lexer.next_token(), None);
}

// ============================================================================
// Parser Tests for Module
// ============================================================================

#[test]
fn test_parse_module_single_initialized_binding() {
    let mut parser = Parser::new("Module[{x = 1}, x]".to_string());
    let expr = parser.parse_expression().unwrap();

    match expr {
        Expression::Module { bindings, body } => {
            assert_eq!(bindings.len(), 1);
            assert_eq!(bindings[0].0, "x");
            assert!(bindings[0].1.is_some());
            match bindings[0].1.as_ref().unwrap() {
                Expression::Number(n) => assert_eq!(*n, 1),
                _ => panic!("Expected number in binding"),
            }
            match body.as_ref() {
                Expression::Identifier(name) => assert_eq!(name, "x"),
                _ => panic!("Expected identifier in body"),
            }
        }
        _ => panic!("Expected Module expression, got {:?}", expr),
    }
}

#[test]
fn test_parse_module_single_uninitialized_binding() {
    let mut parser = Parser::new("Module[{x}, x]".to_string());
    let expr = parser.parse_expression().unwrap();

    match expr {
        Expression::Module { bindings, body } => {
            assert_eq!(bindings.len(), 1);
            assert_eq!(bindings[0].0, "x");
            assert!(bindings[0].1.is_none());
            match body.as_ref() {
                Expression::Identifier(name) => assert_eq!(name, "x"),
                _ => panic!("Expected identifier in body"),
            }
        }
        _ => panic!("Expected Module expression, got {:?}", expr),
    }
}

#[test]
fn test_parse_module_multiple_initialized_bindings() {
    let mut parser = Parser::new("Module[{x = 1, y = 2}, x + y]".to_string());
    let expr = parser.parse_expression().unwrap();

    match expr {
        Expression::Module { bindings, body } => {
            assert_eq!(bindings.len(), 2);

            assert_eq!(bindings[0].0, "x");
            assert!(bindings[0].1.is_some());
            match bindings[0].1.as_ref().unwrap() {
                Expression::Number(n) => assert_eq!(*n, 1),
                _ => panic!("Expected number in first binding"),
            }

            assert_eq!(bindings[1].0, "y");
            assert!(bindings[1].1.is_some());
            match bindings[1].1.as_ref().unwrap() {
                Expression::Number(n) => assert_eq!(*n, 2),
                _ => panic!("Expected number in second binding"),
            }
        }
        _ => panic!("Expected Module expression, got {:?}", expr),
    }
}

#[test]
fn test_parse_module_mixed_bindings() {
    let mut parser = Parser::new("Module[{x = 1, y}, x + y]".to_string());
    let expr = parser.parse_expression().unwrap();

    match expr {
        Expression::Module { bindings, body } => {
            assert_eq!(bindings.len(), 2);

            assert_eq!(bindings[0].0, "x");
            assert!(bindings[0].1.is_some());

            assert_eq!(bindings[1].0, "y");
            assert!(bindings[1].1.is_none());
        }
        _ => panic!("Expected Module expression, got {:?}", expr),
    }
}

#[test]
fn test_parse_module_all_uninitialized() {
    let mut parser = Parser::new("Module[{x, y, z}, x]".to_string());
    let expr = parser.parse_expression().unwrap();

    match expr {
        Expression::Module { bindings, body } => {
            assert_eq!(bindings.len(), 3);
            assert!(bindings[0].1.is_none());
            assert!(bindings[1].1.is_none());
            assert!(bindings[2].1.is_none());
        }
        _ => panic!("Expected Module expression, got {:?}", expr),
    }
}

#[test]
fn test_parse_module_nested_expression() {
    let mut parser = Parser::new("Module[{sum = 0}, Do[Set[sum, sum + i], {i, 5}]]".to_string());
    let expr = parser.parse_expression().unwrap();

    match expr {
        Expression::Module { bindings, body } => {
            assert_eq!(bindings.len(), 1);
            assert_eq!(bindings[0].0, "sum");
            match body.as_ref() {
                Expression::Do { .. } => {}
                _ => panic!("Expected Do expression in body"),
            }
        }
        _ => panic!("Expected Module expression, got {:?}", expr),
    }
}

// ============================================================================
// Type Inference Tests for Module
// ============================================================================

#[test]
fn test_module_type_inference_initialized() {
    let mut parser = Parser::new("Module[{x = 1}, x]".to_string());
    let expr = parser.parse_expression().unwrap();

    let mut inference = TypeInference::new();
    let result = inference.infer_expression(&expr);
    assert!(result.is_ok());
}

#[test]
fn test_module_type_inference_arithmetic() {
    let mut parser = Parser::new("Module[{x = 1, y = 2}, x + y]".to_string());
    let expr = parser.parse_expression().unwrap();

    let mut inference = TypeInference::new();
    let result = inference.infer_expression(&expr);
    assert!(result.is_ok());
}

// ============================================================================
// Code Generation Tests for Module
// ============================================================================

#[test]
fn test_module_codegen_initialized() {
    let mut parser = Parser::new("Module[{x = 1}, x]".to_string());
    let expr = parser.parse_expression().unwrap();

    let mut codegen = RustCodeGenerator::new();
    let result = codegen.generate(&expr).unwrap();

    assert!(result.contains("let mut x = 1"));
}

#[test]
fn test_module_codegen_uninitialized() {
    let mut parser = Parser::new("Module[{x}, x]".to_string());
    let expr = parser.parse_expression().unwrap();

    let mut codegen = RustCodeGenerator::new();
    let result = codegen.generate(&expr).unwrap();

    assert!(result.contains("let mut x: _"));
}

#[test]
fn test_module_codegen_multiple_bindings() {
    let mut parser = Parser::new("Module[{x = 1, y = 2}, x + y]".to_string());
    let expr = parser.parse_expression().unwrap();

    let mut codegen = RustCodeGenerator::new();
    let result = codegen.generate(&expr).unwrap();

    assert!(result.contains("let mut x = 1"));
    assert!(result.contains("let mut y = 2"));
}

#[test]
fn test_module_codegen_mixed_bindings() {
    let mut parser = Parser::new("Module[{x = 1, y}, x]".to_string());
    let expr = parser.parse_expression().unwrap();

    let mut codegen = RustCodeGenerator::new();
    let result = codegen.generate(&expr).unwrap();

    assert!(result.contains("let mut x = 1"));
    assert!(result.contains("let mut y: _"));
}

// ============================================================================
// Comparison Tests: With vs Module
// ============================================================================

#[test]
fn test_with_vs_module_codegen_difference() {
    let mut with_parser = Parser::new("With[{x = 1}, x]".to_string());
    let with_expr = with_parser.parse_expression().unwrap();

    let mut module_parser = Parser::new("Module[{x = 1}, x]".to_string());
    let module_expr = module_parser.parse_expression().unwrap();

    let mut codegen = RustCodeGenerator::new();

    let with_result = codegen.generate(&with_expr).unwrap();
    let module_result = codegen.generate(&module_expr).unwrap();

    assert!(with_result.contains("let x = 1"));
    assert!(with_result.contains("let mut x") == false);
    assert!(module_result.contains("let mut x = 1"));
}

#[test]
fn test_module_can_be_used_for_mutable_state() {
    let mut parser = Parser::new("Module[{counter = 0}, counter]".to_string());
    let expr = parser.parse_expression().unwrap();

    let mut codegen = RustCodeGenerator::new();
    let result = codegen.generate(&expr).unwrap();

    assert!(result.contains("let mut counter = 0"));
}

// ============================================================================
// Edge Cases
// ============================================================================

#[test]
fn test_module_empty_bindings() {
    let mut parser = Parser::new("Module[{}, 42]".to_string());
    let expr = parser.parse_expression().unwrap();

    match expr {
        Expression::Module { bindings, body } => {
            assert_eq!(bindings.len(), 0);
            match body.as_ref() {
                Expression::Number(n) => assert_eq!(*n, 42),
                _ => panic!("Expected number in body"),
            }
        }
        _ => panic!("Expected Module expression, got {:?}", expr),
    }
}

#[test]
fn test_module_nested_modules() {
    let mut parser = Parser::new("Module[{x = 1}, Module[{y = 2}, x + y]]".to_string());
    let expr = parser.parse_expression().unwrap();

    match expr {
        Expression::Module { bindings, body } => {
            assert_eq!(bindings.len(), 1);
            assert_eq!(bindings[0].0, "x");
            match body.as_ref() {
                Expression::Module { .. } => {}
                _ => panic!("Expected nested Module in body"),
            }
        }
        _ => panic!("Expected Module expression, got {:?}", expr),
    }
}
