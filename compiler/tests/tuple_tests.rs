use w::lexer::{Lexer, Token};
use w::parser::Parser;
use w::ast::Expression;
use w::rust_codegen::RustCodeGenerator;

// ============================================
// Lexer Tests
// ============================================

#[test]
fn test_lexer_parentheses() {
    let mut lexer = Lexer::new("()".to_string());
    assert_eq!(lexer.next_token().unwrap(), Token::LeftParen);
    assert_eq!(lexer.next_token().unwrap(), Token::RightParen);
}

#[test]
fn test_lexer_tuple_expression() {
    let mut lexer = Lexer::new("(1, 2)".to_string());
    assert_eq!(lexer.next_token().unwrap(), Token::LeftParen);
    assert_eq!(lexer.next_token().unwrap(), Token::Number(1));
    assert_eq!(lexer.next_token().unwrap(), Token::Comma);
    assert_eq!(lexer.next_token().unwrap(), Token::Number(2));
    assert_eq!(lexer.next_token().unwrap(), Token::RightParen);
}

#[test]
fn test_lexer_parentheses_vs_comments() {
    // Ensure parentheses don't interfere with ML-style comments
    let mut lexer = Lexer::new("(* comment *) (1, 2)".to_string());
    assert_eq!(lexer.next_token().unwrap(), Token::LeftParen);
    assert_eq!(lexer.next_token().unwrap(), Token::Number(1));
}

// ============================================
// Parser Tests - Tuple Expressions
// ============================================

#[test]
fn test_parse_empty_tuple() {
    let mut parser = Parser::new("()".to_string());
    let expr = parser.parse_expression().unwrap();

    match expr {
        Expression::Tuple(elements) => {
            assert_eq!(elements.len(), 0, "Empty tuple should have 0 elements");
        }
        _ => panic!("Expected Tuple expression, got {:?}", expr),
    }
}

#[test]
fn test_parse_single_element_tuple() {
    let mut parser = Parser::new("(42,)".to_string());
    let expr = parser.parse_expression().unwrap();

    match expr {
        Expression::Tuple(elements) => {
            assert_eq!(elements.len(), 1);
            match &elements[0] {
                Expression::Number(n) => assert_eq!(*n, 42),
                _ => panic!("Expected number in tuple"),
            }
        }
        _ => panic!("Expected Tuple expression"),
    }
}

#[test]
fn test_parse_two_element_tuple() {
    let mut parser = Parser::new("(1, \"hello\")".to_string());
    let expr = parser.parse_expression().unwrap();

    match expr {
        Expression::Tuple(elements) => {
            assert_eq!(elements.len(), 2);
            match &elements[0] {
                Expression::Number(n) => assert_eq!(*n, 1),
                _ => panic!("Expected number as first element"),
            }
            match &elements[1] {
                Expression::String(s) => assert_eq!(s, "hello"),
                _ => panic!("Expected string as second element"),
            }
        }
        _ => panic!("Expected Tuple expression"),
    }
}

#[test]
fn test_parse_three_element_tuple() {
    let mut parser = Parser::new("(42, \"test\", true)".to_string());
    let expr = parser.parse_expression().unwrap();

    match expr {
        Expression::Tuple(elements) => {
            assert_eq!(elements.len(), 3);
            match &elements[0] {
                Expression::Number(n) => assert_eq!(*n, 42),
                _ => panic!("Expected number"),
            }
            match &elements[1] {
                Expression::String(s) => assert_eq!(s, "test"),
                _ => panic!("Expected string"),
            }
            match &elements[2] {
                Expression::Boolean(b) => assert_eq!(*b, true),
                _ => panic!("Expected boolean"),
            }
        }
        _ => panic!("Expected Tuple expression"),
    }
}

#[test]
fn test_parse_nested_tuple() {
    let mut parser = Parser::new("((1, 2), (3, 4))".to_string());
    let expr = parser.parse_expression().unwrap();

    match expr {
        Expression::Tuple(elements) => {
            assert_eq!(elements.len(), 2);
            // Check first nested tuple
            match &elements[0] {
                Expression::Tuple(inner) => {
                    assert_eq!(inner.len(), 2);
                }
                _ => panic!("Expected nested tuple"),
            }
        }
        _ => panic!("Expected Tuple expression"),
    }
}

#[test]
fn test_parse_tuple_with_expressions() {
    let mut parser = Parser::new("(1 + 2, 3 * 4)".to_string());
    let expr = parser.parse_expression().unwrap();

    match expr {
        Expression::Tuple(elements) => {
            assert_eq!(elements.len(), 2);
            match &elements[0] {
                Expression::BinaryOp { .. } => {},
                _ => panic!("Expected binary operation"),
            }
        }
        _ => panic!("Expected Tuple expression"),
    }
}

#[test]
fn test_parse_explicit_tuple_constructor() {
    let mut parser = Parser::new("Tuple[10, \"test\"]".to_string());
    let expr = parser.parse_expression().unwrap();

    match expr {
        Expression::FunctionCall { function, arguments } => {
            match function.as_ref() {
                Expression::Identifier(name) => {
                    assert_eq!(name, "Tuple");
                    assert_eq!(arguments.len(), 2);
                }
                _ => panic!("Expected Tuple identifier"),
            }
        }
        _ => panic!("Expected FunctionCall expression"),
    }
}

// ============================================
// Code Generation Tests
// ============================================

#[test]
fn test_codegen_empty_tuple() {
    let mut parser = Parser::new("()".to_string());
    let expr = parser.parse_expression().unwrap();

    let mut codegen = RustCodeGenerator::new();
    let rust_code = codegen.generate(&expr).unwrap();

    assert!(rust_code.contains("()"), "Generated code should contain empty tuple");
}

#[test]
fn test_codegen_single_element_tuple() {
    let mut parser = Parser::new("(42,)".to_string());
    let expr = parser.parse_expression().unwrap();

    let mut codegen = RustCodeGenerator::new();
    let rust_code = codegen.generate(&expr).unwrap();

    // Should have trailing comma for single-element tuples
    assert!(rust_code.contains("(42,)"),
        "Generated code should contain single-element tuple with trailing comma, got: {}", rust_code);
}

#[test]
fn test_codegen_two_element_tuple() {
    let mut parser = Parser::new("(1, 2)".to_string());
    let expr = parser.parse_expression().unwrap();

    let mut codegen = RustCodeGenerator::new();
    let rust_code = codegen.generate(&expr).unwrap();

    assert!(rust_code.contains("(1, 2)"),
        "Generated code should contain two-element tuple, got: {}", rust_code);
}

#[test]
fn test_codegen_mixed_type_tuple() {
    let mut parser = Parser::new("(42, \"hello\", true)".to_string());
    let expr = parser.parse_expression().unwrap();

    let mut codegen = RustCodeGenerator::new();
    let rust_code = codegen.generate(&expr).unwrap();

    assert!(rust_code.contains("(42,") && rust_code.contains("\"hello\"") && rust_code.contains("true"),
        "Generated code should contain mixed-type tuple, got: {}", rust_code);
}

#[test]
fn test_codegen_nested_tuple() {
    let mut parser = Parser::new("((1, 2), (3, 4))".to_string());
    let expr = parser.parse_expression().unwrap();

    let mut codegen = RustCodeGenerator::new();
    let rust_code = codegen.generate(&expr).unwrap();

    assert!(rust_code.contains("((1, 2), (3, 4))"),
        "Generated code should contain nested tuples, got: {}", rust_code);
}

#[test]
fn test_codegen_explicit_tuple_constructor() {
    let mut parser = Parser::new("Tuple[5, 10]".to_string());
    let expr = parser.parse_expression().unwrap();

    let mut codegen = RustCodeGenerator::new();
    let rust_code = codegen.generate(&expr).unwrap();

    assert!(rust_code.contains("(5, 10)"),
        "Explicit Tuple constructor should generate tuple syntax, got: {}", rust_code);
}

#[test]
fn test_codegen_tuple_in_print() {
    let mut parser = Parser::new("Print[(1, 2)]".to_string());
    let expr = parser.parse_expression().unwrap();

    let mut codegen = RustCodeGenerator::new();
    let rust_code = codegen.generate(&expr).unwrap();

    // Tuples should use {:?} formatter in println!
    assert!(rust_code.contains("{:?}"),
        "Print with tuple should use debug formatter, got: {}", rust_code);
}

// ============================================
// Integration Tests
// ============================================

#[test]
fn test_function_with_tuple_parameter() {
    let input = "Swap[pair: Tuple[Int32, String]] := pair";
    let mut parser = Parser::new(input.to_string());
    let expr = parser.parse_expression().unwrap();

    let mut codegen = RustCodeGenerator::new();
    let rust_code = codegen.generate(&expr).unwrap();

    assert!(rust_code.contains("fn swap(pair: (i32, String))"),
        "Function with tuple parameter should generate correct signature, got: {}", rust_code);
}

#[test]
fn test_function_returning_tuple() {
    let input = "MakePair[x: Int32, y: String] := (x, y)";
    let mut parser = Parser::new(input.to_string());
    let expr = parser.parse_expression().unwrap();

    let mut codegen = RustCodeGenerator::new();
    let rust_code = codegen.generate(&expr).unwrap();

    assert!(rust_code.contains("-> (i32, String)"),
        "Function returning tuple should have correct return type, got: {}", rust_code);
    assert!(rust_code.contains("(x, y)"),
        "Function body should return tuple, got: {}", rust_code);
}
