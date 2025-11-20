use w::parser::Parser;
use w::ast::Expression;
use w::rust_codegen::RustCodeGenerator;

// ============================================
// Parser Tests - Lambda/Closure Expressions
// ============================================

#[test]
fn test_parse_simple_lambda() {
    let mut parser = Parser::new("Function[{x}, x * 2]".to_string());
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
fn test_parse_lambda_with_multiple_params() {
    let mut parser = Parser::new("Function[{x, y}, x + y]".to_string());
    let expr = parser.parse_expression().unwrap();

    match expr {
        Expression::Lambda { parameters, body: _ } => {
            assert_eq!(parameters.len(), 2);
            assert_eq!(parameters[0].name, "x");
            assert_eq!(parameters[1].name, "y");
        }
        _ => panic!("Expected Lambda expression"),
    }
}

#[test]
fn test_parse_lambda_with_type_annotation() {
    let mut parser = Parser::new("Function[{x: Int32}, x * x]".to_string());
    let expr = parser.parse_expression().unwrap();

    match expr {
        Expression::Lambda { parameters, body: _ } => {
            assert_eq!(parameters.len(), 1);
            assert_eq!(parameters[0].name, "x");
        }
        _ => panic!("Expected Lambda expression"),
    }
}

#[test]
fn test_parse_lambda_in_map() {
    let mut parser = Parser::new("Map[Function[{x}, x * 2], [1, 2, 3]]".to_string());
    let expr = parser.parse_expression().unwrap();

    match expr {
        Expression::FunctionCall { function, arguments } => {
            match function.as_ref() {
                Expression::Identifier(name) => assert_eq!(name, "Map"),
                _ => panic!("Expected Map identifier"),
            }
            assert_eq!(arguments.len(), 2);
            match &arguments[0] {
                Expression::Lambda { .. } => {},
                _ => panic!("Expected lambda as first argument"),
            }
        }
        _ => panic!("Expected FunctionCall expression"),
    }
}

// ============================================
// Code Generation Tests - Lambdas
// ============================================

#[test]
fn test_codegen_simple_lambda() {
    let mut parser = Parser::new("Function[{x}, x * 2]".to_string());
    let expr = parser.parse_expression().unwrap();

    let mut codegen = RustCodeGenerator::new();
    let rust_code = codegen.generate(&expr).unwrap();

    assert!(rust_code.contains("|x|"),
        "Should generate Rust closure, got: {}", rust_code);
    assert!(rust_code.contains("x * 2"),
        "Should contain closure body, got: {}", rust_code);
}

#[test]
fn test_codegen_lambda_with_multiple_params() {
    let mut parser = Parser::new("Function[{x, y}, x + y]".to_string());
    let expr = parser.parse_expression().unwrap();

    let mut codegen = RustCodeGenerator::new();
    let rust_code = codegen.generate(&expr).unwrap();

    assert!(rust_code.contains("|x, y|"),
        "Should generate closure with multiple params, got: {}", rust_code);
}

// ============================================
// Code Generation Tests - Map
// ============================================

#[test]
fn test_codegen_map() {
    let mut parser = Parser::new("Map[Function[{x}, x * 2], [1, 2, 3]]".to_string());
    let expr = parser.parse_expression().unwrap();

    let mut codegen = RustCodeGenerator::new();
    let rust_code = codegen.generate(&expr).unwrap();

    assert!(rust_code.contains(".into_iter().map("),
        "Should generate iterator map, got: {}", rust_code);
    assert!(rust_code.contains("|x| (x * 2)"),
        "Should inline lambda in map, got: {}", rust_code);
    assert!(rust_code.contains(".collect::<Vec<_>>()"),
        "Should collect into Vec, got: {}", rust_code);
}

#[test]
fn test_codegen_map_with_print() {
    let mut parser = Parser::new("Print[Map[Function[{x}, x * 2], [1, 2, 3]]]".to_string());
    let expr = parser.parse_expression().unwrap();

    let mut codegen = RustCodeGenerator::new();
    let rust_code = codegen.generate(&expr).unwrap();

    assert!(rust_code.contains("{:?}"),
        "Map result should use debug formatter in print, got: {}", rust_code);
}

// ============================================
// Code Generation Tests - Filter
// ============================================

#[test]
fn test_codegen_filter() {
    let mut parser = Parser::new("Filter[Function[{x}, x > 5], [1, 10, 3, 8]]".to_string());
    let expr = parser.parse_expression().unwrap();

    let mut codegen = RustCodeGenerator::new();
    let rust_code = codegen.generate(&expr).unwrap();

    assert!(rust_code.contains(".into_iter().filter("),
        "Should generate iterator filter, got: {}", rust_code);
    assert!(rust_code.contains("|&x| (x > 5)"),
        "Should use pattern matching in filter, got: {}", rust_code);
    assert!(rust_code.contains(".collect::<Vec<_>>()"),
        "Should collect into Vec, got: {}", rust_code);
}

#[test]
fn test_codegen_filter_with_print() {
    let mut parser = Parser::new("Print[Filter[Function[{x}, x > 5], [1, 10]]]".to_string());
    let expr = parser.parse_expression().unwrap();

    let mut codegen = RustCodeGenerator::new();
    let rust_code = codegen.generate(&expr).unwrap();

    assert!(rust_code.contains("{:?}"),
        "Filter result should use debug formatter in print, got: {}", rust_code);
}

// ============================================
// Code Generation Tests - Fold
// ============================================

#[test]
fn test_codegen_fold() {
    let mut parser = Parser::new("Fold[Function[{acc, x}, acc + x], 0, [1, 2, 3]]".to_string());
    let expr = parser.parse_expression().unwrap();

    let mut codegen = RustCodeGenerator::new();
    let rust_code = codegen.generate(&expr).unwrap();

    assert!(rust_code.contains(".into_iter().fold("),
        "Should generate iterator fold, got: {}", rust_code);
    assert!(rust_code.contains("|acc, x| (acc + x)"),
        "Should inline lambda in fold, got: {}", rust_code);
}

#[test]
fn test_codegen_fold_with_init() {
    let mut parser = Parser::new("Fold[Function[{acc, x}, acc + x], 10, [1, 2, 3]]".to_string());
    let expr = parser.parse_expression().unwrap();

    let mut codegen = RustCodeGenerator::new();
    let rust_code = codegen.generate(&expr).unwrap();

    assert!(rust_code.contains("fold(10,"),
        "Should use initial value in fold, got: {}", rust_code);
}

// ============================================
// Integration Tests - Full Compilation
// ============================================

#[test]
fn test_map_doubles_list() {
    let input = "Map[Function[{x}, x * 2], [1, 2, 3]]";
    let mut parser = Parser::new(input.to_string());
    let expr = parser.parse_expression().unwrap();

    let mut codegen = RustCodeGenerator::new();
    let rust_code = codegen.generate(&expr).unwrap();

    assert!(rust_code.contains("vec![1, 2, 3].into_iter().map(|x| (x * 2)).collect::<Vec<_>>()"),
        "Should generate complete map expression, got: {}", rust_code);
}

#[test]
fn test_filter_greater_than() {
    let input = "Filter[Function[{x}, x > 5], [1, 10, 3, 8, 2]]";
    let mut parser = Parser::new(input.to_string());
    let expr = parser.parse_expression().unwrap();

    let mut codegen = RustCodeGenerator::new();
    let rust_code = codegen.generate(&expr).unwrap();

    assert!(rust_code.contains(".filter(|&x| (x > 5))"),
        "Should generate filter with comparison, got: {}", rust_code);
}

#[test]
fn test_fold_sum() {
    let input = "Fold[Function[{acc, x}, acc + x], 0, [1, 2, 3, 4, 5]]";
    let mut parser = Parser::new(input.to_string());
    let expr = parser.parse_expression().unwrap();

    let mut codegen = RustCodeGenerator::new();
    let rust_code = codegen.generate(&expr).unwrap();

    assert!(rust_code.contains(".fold(0, |acc, x| (acc + x))"),
        "Should generate fold with addition, got: {}", rust_code);
}

#[test]
fn test_nested_map() {
    let input = "Map[Function[{x}, x + 1], Map[Function[{y}, y * 2], [1, 2, 3]]]";
    let mut parser = Parser::new(input.to_string());
    let expr = parser.parse_expression().unwrap();

    let mut codegen = RustCodeGenerator::new();
    let rust_code = codegen.generate(&expr).unwrap();

    assert!(rust_code.contains(".into_iter().map("),
        "Should generate nested map operations, got: {}", rust_code);
}

#[test]
fn test_lambda_with_complex_body() {
    let input = "Map[Function[{x}, x * x + x], [1, 2, 3]]";
    let mut parser = Parser::new(input.to_string());
    let expr = parser.parse_expression().unwrap();

    let mut codegen = RustCodeGenerator::new();
    let rust_code = codegen.generate(&expr).unwrap();

    assert!(rust_code.contains("|x| ((x * x) + x)"),
        "Should handle complex lambda body, got: {}", rust_code);
}
