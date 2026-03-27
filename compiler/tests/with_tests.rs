use w::ast::Expression;
use w::ast::Type;
use w::lexer::{Lexer, Token};
use w::parser::Parser;
use w::rust_codegen::RustCodeGenerator;
use w::type_inference::{TypeError, TypeInference};

// ============================================================================
// Lexer Tests for With and Assign Token
// ============================================================================

#[test]
fn test_assign_token() {
    let mut lexer = Lexer::new("x = 5".to_string());

    assert_eq!(lexer.next_token(), Some(Token::Identifier("x".to_string())));
    assert_eq!(lexer.next_token(), Some(Token::Assign));
    assert_eq!(lexer.next_token(), Some(Token::Number(5)));
    assert_eq!(lexer.next_token(), None);
}

#[test]
fn test_equals_still_works() {
    let mut lexer = Lexer::new("x == y".to_string());

    assert_eq!(lexer.next_token(), Some(Token::Identifier("x".to_string())));
    assert_eq!(lexer.next_token(), Some(Token::Equals));
    assert_eq!(lexer.next_token(), Some(Token::Identifier("y".to_string())));
    assert_eq!(lexer.next_token(), None);
}

#[test]
fn test_with_tokens() {
    let mut lexer = Lexer::new("With[{x = 1}, x]".to_string());

    assert_eq!(
        lexer.next_token(),
        Some(Token::Identifier("With".to_string()))
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

// ============================================================================
// Parser Tests for With
// ============================================================================

#[test]
fn test_parse_with_single_binding() {
    let mut parser = Parser::new("With[{x = 1}, x]".to_string());
    let expr = parser.parse_expression().unwrap();

    match expr {
        Expression::With { bindings, body } => {
            assert_eq!(bindings.len(), 1);
            assert_eq!(bindings[0].0, "x");
            match &bindings[0].1 {
                Expression::Number(n) => assert_eq!(*n, 1),
                _ => panic!("Expected number in binding"),
            }
            match body.as_ref() {
                Expression::Identifier(name) => assert_eq!(name, "x"),
                _ => panic!("Expected identifier in body"),
            }
        }
        _ => panic!("Expected With expression, got {:?}", expr),
    }
}

#[test]
fn test_parse_with_multiple_bindings() {
    let mut parser = Parser::new("With[{x = 1, y = 2}, x + y]".to_string());
    let expr = parser.parse_expression().unwrap();

    match expr {
        Expression::With { bindings, body } => {
            assert_eq!(bindings.len(), 2);

            // First binding: x = 1
            assert_eq!(bindings[0].0, "x");
            match &bindings[0].1 {
                Expression::Number(n) => assert_eq!(*n, 1),
                _ => panic!("Expected number in first binding"),
            }

            // Second binding: y = 2
            assert_eq!(bindings[1].0, "y");
            match &bindings[1].1 {
                Expression::Number(n) => assert_eq!(*n, 2),
                _ => panic!("Expected number in second binding"),
            }

            // Body: x + y (binary op)
            match body.as_ref() {
                Expression::BinaryOp { .. } => {}
                _ => panic!("Expected binary operation in body"),
            }
        }
        _ => panic!("Expected With expression"),
    }
}

#[test]
fn test_parse_with_sequential_bindings() {
    let mut parser = Parser::new("With[{x = 1, y = x + 1}, y]".to_string());
    let expr = parser.parse_expression().unwrap();

    match expr {
        Expression::With { bindings, body } => {
            assert_eq!(bindings.len(), 2);

            // First binding: x = 1
            assert_eq!(bindings[0].0, "x");
            match &bindings[0].1 {
                Expression::Number(n) => assert_eq!(*n, 1),
                _ => panic!("Expected number"),
            }

            // Second binding: y = x + 1
            assert_eq!(bindings[1].0, "y");
            match &bindings[1].1 {
                Expression::BinaryOp { left, right, .. } => {
                    match left.as_ref() {
                        Expression::Identifier(n) => assert_eq!(n, "x"),
                        _ => panic!("Expected x"),
                    }
                    match right.as_ref() {
                        Expression::Number(n) => assert_eq!(*n, 1),
                        _ => panic!("Expected 1"),
                    }
                }
                _ => panic!("Expected binary op in second binding"),
            }

            // Body: y
            match body.as_ref() {
                Expression::Identifier(name) => assert_eq!(name, "y"),
                _ => panic!("Expected identifier"),
            }
        }
        _ => panic!("Expected With expression"),
    }
}

#[test]
fn test_parse_with_nested() {
    let mut parser = Parser::new("With[{x = 1}, With[{y = 2}, x + y]]".to_string());
    let expr = parser.parse_expression().unwrap();

    match expr {
        Expression::With {
            bindings: outer_bindings,
            body: outer_body,
        } => {
            assert_eq!(outer_bindings.len(), 1);
            match outer_body.as_ref() {
                Expression::With {
                    bindings: inner_bindings,
                    body: inner_body,
                } => {
                    assert_eq!(inner_bindings.len(), 1);
                    match inner_body.as_ref() {
                        Expression::BinaryOp { .. } => {}
                        _ => panic!("Expected binary op in inner body"),
                    }
                }
                _ => panic!("Expected nested With"),
            }
        }
        _ => panic!("Expected outer With"),
    }
}

#[test]
fn test_parse_with_in_print() {
    let mut parser = Parser::new("Print[With[{x = 5}, x + 10]]".to_string());
    let expr = parser.parse_expression().unwrap();

    match expr {
        Expression::FunctionCall {
            function,
            arguments,
        } => {
            match function.as_ref() {
                Expression::Identifier(name) => assert_eq!(name, "Print"),
                _ => panic!("Expected Print identifier"),
            }
            assert_eq!(arguments.len(), 1);
            match &arguments[0] {
                Expression::With { .. } => {}
                _ => panic!("Expected With in argument"),
            }
        }
        _ => panic!("Expected function call"),
    }
}

// ============================================================================
// Code Generation Tests for With
// ============================================================================

#[test]
fn test_codegen_with_single_binding() {
    let mut parser = Parser::new("With[{x = 5}, x + 1]".to_string());
    let expr = parser.parse_expression().unwrap();

    let mut codegen = RustCodeGenerator::new();
    let rust_code = codegen.generate(&expr).unwrap();

    assert!(
        rust_code.contains("let x = 5;"),
        "Should generate let binding, got: {}",
        rust_code
    );
    assert!(
        rust_code.contains("(x + 1)"),
        "Should contain body expression, got: {}",
        rust_code
    );
}

#[test]
fn test_codegen_with_multiple_bindings() {
    let mut parser = Parser::new("With[{x = 1, y = 2}, x + y]".to_string());
    let expr = parser.parse_expression().unwrap();

    let mut codegen = RustCodeGenerator::new();
    let rust_code = codegen.generate(&expr).unwrap();

    assert!(
        rust_code.contains("let x = 1;"),
        "Should have first binding, got: {}",
        rust_code
    );
    assert!(
        rust_code.contains("let y = 2;"),
        "Should have second binding, got: {}",
        rust_code
    );
    assert!(
        rust_code.contains("(x + y)"),
        "Should have body, got: {}",
        rust_code
    );
}

#[test]
fn test_codegen_with_nested() {
    let mut parser = Parser::new("With[{x = 1}, With[{y = x + 1}, y]]".to_string());
    let expr = parser.parse_expression().unwrap();

    let mut codegen = RustCodeGenerator::new();
    let rust_code = codegen.generate(&expr).unwrap();

    // Should contain nested blocks
    assert!(
        rust_code.contains("let x = 1;"),
        "Should have outer binding, got: {}",
        rust_code
    );
    assert!(
        rust_code.contains("let y ="),
        "Should have inner binding, got: {}",
        rust_code
    );
}

#[test]
fn test_codegen_with_snake_case_conversion() {
    let mut parser = Parser::new("With[{myVar = 10}, myVar]".to_string());
    let expr = parser.parse_expression().unwrap();

    let mut codegen = RustCodeGenerator::new();
    let rust_code = codegen.generate(&expr).unwrap();

    assert!(
        rust_code.contains("let my_var = 10;"),
        "Should convert to snake_case, got: {}",
        rust_code
    );
}

#[test]
fn test_codegen_with_in_print() {
    let mut parser = Parser::new("Print[With[{x = 42}, x]]".to_string());
    let expr = parser.parse_expression().unwrap();

    let mut codegen = RustCodeGenerator::new();
    let rust_code = codegen.generate(&expr).unwrap();

    assert!(
        rust_code.contains("let x = 42;"),
        "Should have binding in print, got: {}",
        rust_code
    );
    assert!(
        rust_code.contains("println!"),
        "Should have print call, got: {}",
        rust_code
    );
}

// ============================================================================
// Type Inference Tests for With
// ============================================================================

#[test]
fn test_infer_with_simple() {
    let mut parser = Parser::new("With[{x = 1}, x]".to_string());
    let expr = parser.parse_expression().unwrap();

    let mut inference = TypeInference::new();
    let result = inference.infer_expression(&expr);

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Type::Int32);
}

#[test]
fn test_infer_with_multiple_bindings() {
    let mut parser = Parser::new("With[{x = 1, y = 2}, x + y]".to_string());
    let expr = parser.parse_expression().unwrap();

    let mut inference = TypeInference::new();
    let result = inference.infer_expression(&expr);

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Type::Int32);
}

#[test]
fn test_infer_with_sequential_binding_reference() {
    let mut parser = Parser::new("With[{x = 1, y = x + 1}, y]".to_string());
    let expr = parser.parse_expression().unwrap();

    let mut inference = TypeInference::new();
    let result = inference.infer_expression(&expr);

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Type::Int32);
}

#[test]
fn test_infer_with_string_binding() {
    let mut parser = Parser::new("With[{s = \"hello\"}, s]".to_string());
    let expr = parser.parse_expression().unwrap();

    let mut inference = TypeInference::new();
    let result = inference.infer_expression(&expr);

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Type::String);
}

#[test]
fn test_infer_with_tuple_binding() {
    let mut parser = Parser::new("With[{t = (1, \"x\")}, t]".to_string());
    let expr = parser.parse_expression().unwrap();

    let mut inference = TypeInference::new();
    let result = inference.infer_expression(&expr);

    assert!(result.is_ok());
    match result.unwrap() {
        Type::Tuple(types) => {
            assert_eq!(types.len(), 2);
            assert_eq!(types[0], Type::Int32);
            assert_eq!(types[1], Type::String);
        }
        _ => panic!("Expected tuple type"),
    }
}

#[test]
fn test_infer_with_undefined_reference() {
    let mut parser = Parser::new("With[{x = 1}, y]".to_string());
    let expr = parser.parse_expression().unwrap();

    let mut inference = TypeInference::new();
    let result = inference.infer_expression(&expr);

    assert!(result.is_err());
    match result.unwrap_err() {
        TypeError::UndefinedIdentifier(name) => assert_eq!(name, "y"),
        _ => panic!("Expected UndefinedIdentifier error"),
    }
}

#[test]
fn test_infer_with_nested() {
    let mut parser = Parser::new("With[{x = 1}, With[{y = x + 1}, y]]".to_string());
    let expr = parser.parse_expression().unwrap();

    let mut inference = TypeInference::new();
    let result = inference.infer_expression(&expr);

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Type::Int32);
}
