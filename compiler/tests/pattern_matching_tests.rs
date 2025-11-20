use w::lexer::{Lexer, Token};
use w::parser::Parser;
use w::ast::{Expression, Pattern};
use w::rust_codegen::RustCodeGenerator;

// ============================================
// Lexer Tests - Pattern Matching Tokens
// ============================================

#[test]
fn test_lexer_underscore() {
    let mut lexer = Lexer::new("_".to_string());
    assert_eq!(lexer.next_token().unwrap(), Token::Underscore);
}

// ============================================
// Parser Tests - Pattern Matching
// ============================================

#[test]
fn test_parse_wildcard_pattern() {
    let mut parser = Parser::new("Match[x, [_, 0]]".to_string());
    let expr = parser.parse_expression().unwrap();

    match expr {
        Expression::Match { value: _, arms } => {
            assert_eq!(arms.len(), 1);
            match &arms[0].0 {
                Pattern::Wildcard => {},
                _ => panic!("Expected wildcard pattern"),
            }
        }
        _ => panic!("Expected Match expression"),
    }
}

#[test]
fn test_parse_literal_number_pattern() {
    let mut parser = Parser::new("Match[x, [42, \"found\"]]".to_string());
    let expr = parser.parse_expression().unwrap();

    match expr {
        Expression::Match { value: _, arms } => {
            assert_eq!(arms.len(), 1);
            match &arms[0].0 {
                Pattern::Literal(expr) => {
                    match expr.as_ref() {
                        Expression::Number(n) => assert_eq!(*n, 42),
                        _ => panic!("Expected number in literal pattern"),
                    }
                }
                _ => panic!("Expected literal pattern"),
            }
        }
        _ => panic!("Expected Match expression"),
    }
}

#[test]
fn test_parse_literal_string_pattern() {
    let mut parser = Parser::new("Match[x, [\"hello\", 1]]".to_string());
    let expr = parser.parse_expression().unwrap();

    match expr {
        Expression::Match { value: _, arms } => {
            assert_eq!(arms.len(), 1);
            match &arms[0].0 {
                Pattern::Literal(expr) => {
                    match expr.as_ref() {
                        Expression::String(s) => assert_eq!(s, "hello"),
                        _ => panic!("Expected string in literal pattern"),
                    }
                }
                _ => panic!("Expected literal pattern"),
            }
        }
        _ => panic!("Expected Match expression"),
    }
}

#[test]
fn test_parse_variable_pattern() {
    let mut parser = Parser::new("Match[x, [value, value]]".to_string());
    let expr = parser.parse_expression().unwrap();

    match expr {
        Expression::Match { value: _, arms } => {
            assert_eq!(arms.len(), 1);
            match &arms[0].0 {
                Pattern::Variable(name) => assert_eq!(name, "value"),
                _ => panic!("Expected variable pattern"),
            }
        }
        _ => panic!("Expected Match expression"),
    }
}

#[test]
fn test_parse_some_pattern() {
    let mut parser = Parser::new("Match[opt, [Some[x], x]]".to_string());
    let expr = parser.parse_expression().unwrap();

    match expr {
        Expression::Match { value: _, arms } => {
            assert_eq!(arms.len(), 1);
            match &arms[0].0 {
                Pattern::Constructor { name, patterns } => {
                    assert_eq!(name, "Some");
                    assert_eq!(patterns.len(), 1);
                    match &patterns[0] {
                        Pattern::Variable(v) => assert_eq!(v, "x"),
                        _ => panic!("Expected variable in Some pattern"),
                    }
                }
                _ => panic!("Expected constructor pattern"),
            }
        }
        _ => panic!("Expected Match expression"),
    }
}

#[test]
fn test_parse_none_pattern() {
    let mut parser = Parser::new("Match[opt, [None, 0]]".to_string());
    let expr = parser.parse_expression().unwrap();

    match expr {
        Expression::Match { value: _, arms } => {
            assert_eq!(arms.len(), 1);
            match &arms[0].0 {
                Pattern::Constructor { name, patterns } => {
                    assert_eq!(name, "None");
                    assert_eq!(patterns.len(), 0);
                }
                _ => panic!("Expected constructor pattern"),
            }
        }
        _ => panic!("Expected Match expression"),
    }
}

#[test]
fn test_parse_ok_pattern() {
    let mut parser = Parser::new("Match[result, [Ok[value], value]]".to_string());
    let expr = parser.parse_expression().unwrap();

    match expr {
        Expression::Match { value: _, arms } => {
            assert_eq!(arms.len(), 1);
            match &arms[0].0 {
                Pattern::Constructor { name, patterns } => {
                    assert_eq!(name, "Ok");
                    assert_eq!(patterns.len(), 1);
                }
                _ => panic!("Expected constructor pattern"),
            }
        }
        _ => panic!("Expected Match expression"),
    }
}

#[test]
fn test_parse_err_pattern() {
    let mut parser = Parser::new("Match[result, [Err[e], e]]".to_string());
    let expr = parser.parse_expression().unwrap();

    match expr {
        Expression::Match { value: _, arms } => {
            assert_eq!(arms.len(), 1);
            match &arms[0].0 {
                Pattern::Constructor { name, patterns } => {
                    assert_eq!(name, "Err");
                    assert_eq!(patterns.len(), 1);
                }
                _ => panic!("Expected constructor pattern"),
            }
        }
        _ => panic!("Expected Match expression"),
    }
}

#[test]
fn test_parse_tuple_pattern() {
    let mut parser = Parser::new("Match[pair, [(x, y), x]]".to_string());
    let expr = parser.parse_expression().unwrap();

    match expr {
        Expression::Match { value: _, arms } => {
            assert_eq!(arms.len(), 1);
            match &arms[0].0 {
                Pattern::Tuple(patterns) => {
                    assert_eq!(patterns.len(), 2);
                    match &patterns[0] {
                        Pattern::Variable(v) => assert_eq!(v, "x"),
                        _ => panic!("Expected variable in tuple pattern"),
                    }
                    match &patterns[1] {
                        Pattern::Variable(v) => assert_eq!(v, "y"),
                        _ => panic!("Expected variable in tuple pattern"),
                    }
                }
                _ => panic!("Expected tuple pattern"),
            }
        }
        _ => panic!("Expected Match expression"),
    }
}

#[test]
fn test_parse_list_pattern() {
    let mut parser = Parser::new("Match[list, [[a, b], a]]".to_string());
    let expr = parser.parse_expression().unwrap();

    match expr {
        Expression::Match { value: _, arms } => {
            assert_eq!(arms.len(), 1);
            match &arms[0].0 {
                Pattern::List(patterns) => {
                    assert_eq!(patterns.len(), 2);
                }
                _ => panic!("Expected list pattern"),
            }
        }
        _ => panic!("Expected Match expression"),
    }
}

#[test]
fn test_parse_match_multiple_arms() {
    let mut parser = Parser::new("Match[x, [0, \"zero\"], [1, \"one\"], [_, \"other\"]]".to_string());
    let expr = parser.parse_expression().unwrap();

    match expr {
        Expression::Match { value: _, arms } => {
            assert_eq!(arms.len(), 3);
        }
        _ => panic!("Expected Match expression"),
    }
}

// ============================================
// Code Generation Tests - Pattern Matching
// ============================================

#[test]
fn test_codegen_wildcard_pattern() {
    let mut parser = Parser::new("Match[x, [_, 0]]".to_string());
    let expr = parser.parse_expression().unwrap();

    let mut codegen = RustCodeGenerator::new();
    let rust_code = codegen.generate(&expr).unwrap();

    assert!(rust_code.contains("match") && rust_code.contains("_ =>"),
        "Should generate match with wildcard pattern, got: {}", rust_code);
}

#[test]
fn test_codegen_literal_pattern() {
    let mut parser = Parser::new("Match[x, [42, \"found\"]]".to_string());
    let expr = parser.parse_expression().unwrap();

    let mut codegen = RustCodeGenerator::new();
    let rust_code = codegen.generate(&expr).unwrap();

    assert!(rust_code.contains("42 =>"),
        "Should generate literal pattern, got: {}", rust_code);
}

#[test]
fn test_codegen_variable_pattern() {
    let mut parser = Parser::new("Match[x, [value, value]]".to_string());
    let expr = parser.parse_expression().unwrap();

    let mut codegen = RustCodeGenerator::new();
    let rust_code = codegen.generate(&expr).unwrap();

    assert!(rust_code.contains("value =>"),
        "Should generate variable pattern, got: {}", rust_code);
}

#[test]
fn test_codegen_some_pattern() {
    let mut parser = Parser::new("Match[opt, [Some[x], x], [None, 0]]".to_string());
    let expr = parser.parse_expression().unwrap();

    let mut codegen = RustCodeGenerator::new();
    let rust_code = codegen.generate(&expr).unwrap();

    assert!(rust_code.contains("Some(x) =>") && rust_code.contains("None =>"),
        "Should generate Some and None patterns, got: {}", rust_code);
}

#[test]
fn test_codegen_ok_err_pattern() {
    let mut parser = Parser::new("Match[result, [Ok[v], v], [Err[e], e]]".to_string());
    let expr = parser.parse_expression().unwrap();

    let mut codegen = RustCodeGenerator::new();
    let rust_code = codegen.generate(&expr).unwrap();

    assert!(rust_code.contains("Ok(v) =>") && rust_code.contains("Err(e) =>"),
        "Should generate Ok and Err patterns, got: {}", rust_code);
}

#[test]
fn test_codegen_tuple_pattern() {
    let mut parser = Parser::new("Match[pair, [(x, y), x]]".to_string());
    let expr = parser.parse_expression().unwrap();

    let mut codegen = RustCodeGenerator::new();
    let rust_code = codegen.generate(&expr).unwrap();

    assert!(rust_code.contains("(x, y) =>"),
        "Should generate tuple pattern, got: {}", rust_code);
}

#[test]
fn test_codegen_nested_pattern() {
    let mut parser = Parser::new("Match[opt, [Some[(x, y)], x]]".to_string());
    let expr = parser.parse_expression().unwrap();

    let mut codegen = RustCodeGenerator::new();
    let rust_code = codegen.generate(&expr).unwrap();

    assert!(rust_code.contains("Some((x, y)) =>"),
        "Should generate nested pattern, got: {}", rust_code);
}

// ============================================
// Integration Tests - Pattern Matching
// ============================================

#[test]
fn test_match_with_option_type() {
    let input = "Match[Some[42], [Some[x], x], [None, 0]]";
    let mut parser = Parser::new(input.to_string());
    let expr = parser.parse_expression().unwrap();

    let mut codegen = RustCodeGenerator::new();
    let rust_code = codegen.generate(&expr).unwrap();

    assert!(rust_code.contains("match Some(42)"),
        "Should generate match on Some value, got: {}", rust_code);
    assert!(rust_code.contains("Some(x) =>"),
        "Should generate Some pattern, got: {}", rust_code);
    assert!(rust_code.contains("None =>"),
        "Should generate None pattern, got: {}", rust_code);
}

#[test]
fn test_match_with_result_type() {
    let input = "Match[Ok[100], [Ok[val], val], [Err[e], e]]";
    let mut parser = Parser::new(input.to_string());
    let expr = parser.parse_expression().unwrap();

    let mut codegen = RustCodeGenerator::new();
    let rust_code = codegen.generate(&expr).unwrap();

    assert!(rust_code.contains("match Ok(100)"),
        "Should generate match on Ok value, got: {}", rust_code);
    assert!(rust_code.contains("Ok(val) =>"),
        "Should generate Ok pattern, got: {}", rust_code);
    assert!(rust_code.contains("Err(e) =>"),
        "Should generate Err pattern, got: {}", rust_code);
}

#[test]
fn test_match_with_number() {
    let input = "Match[5, [1, \"one\"], [2, \"two\"], [_, \"other\"]]";
    let mut parser = Parser::new(input.to_string());
    let expr = parser.parse_expression().unwrap();

    let mut codegen = RustCodeGenerator::new();
    let rust_code = codegen.generate(&expr).unwrap();

    assert!(rust_code.contains("match 5"),
        "Should generate match on number, got: {}", rust_code);
    assert!(rust_code.contains("1 =>"),
        "Should generate literal pattern 1, got: {}", rust_code);
    assert!(rust_code.contains("2 =>"),
        "Should generate literal pattern 2, got: {}", rust_code);
    assert!(rust_code.contains("_ =>"),
        "Should generate wildcard pattern, got: {}", rust_code);
}
