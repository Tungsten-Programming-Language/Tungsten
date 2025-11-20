use w::parser::Parser;
use w::ast::{Expression, Type};
use w::rust_codegen::RustCodeGenerator;

// ============================================================================
// Parser Tests for Struct Definitions
// ============================================================================

#[test]
fn test_parse_struct_definition_simple() {
    let input = "Struct[Point, [x: Int32, y: Int32]]";
    let mut parser = Parser::new(input.to_string());
    let result = parser.parse_expression();

    assert!(result.is_some(), "Failed to parse struct definition");

    match result.unwrap() {
        Expression::StructDefinition { name, fields } => {
            assert_eq!(name, "Point");
            assert_eq!(fields.len(), 2);
            assert_eq!(fields[0].name, "x");
            assert_eq!(fields[0].type_, Type::Int32);
            assert_eq!(fields[1].name, "y");
            assert_eq!(fields[1].type_, Type::Int32);
        }
        _ => panic!("Expected StructDefinition"),
    }
}

#[test]
fn test_parse_struct_definition_with_string() {
    let input = "Struct[Person, [name: String, age: Int32]]";
    let mut parser = Parser::new(input.to_string());
    let result = parser.parse_expression();

    assert!(result.is_some());

    match result.unwrap() {
        Expression::StructDefinition { name, fields } => {
            assert_eq!(name, "Person");
            assert_eq!(fields.len(), 2);
            assert_eq!(fields[0].name, "name");
            assert_eq!(fields[0].type_, Type::String);
            assert_eq!(fields[1].name, "age");
            assert_eq!(fields[1].type_, Type::Int32);
        }
        _ => panic!("Expected StructDefinition"),
    }
}

#[test]
fn test_parse_struct_definition_with_float() {
    let input = "Struct[Circle, [radius: Float64]]";
    let mut parser = Parser::new(input.to_string());
    let result = parser.parse_expression();

    assert!(result.is_some());

    match result.unwrap() {
        Expression::StructDefinition { name, fields } => {
            assert_eq!(name, "Circle");
            assert_eq!(fields.len(), 1);
            assert_eq!(fields[0].name, "radius");
            assert_eq!(fields[0].type_, Type::Float64);
        }
        _ => panic!("Expected StructDefinition"),
    }
}

#[test]
fn test_parse_struct_instantiation() {
    // Struct instantiation is parsed as a function call
    let input = "Point[10, 20]";
    let mut parser = Parser::new(input.to_string());
    let result = parser.parse_expression();

    assert!(result.is_some());

    match result.unwrap() {
        Expression::FunctionCall { function, arguments } => {
            match *function {
                Expression::Identifier(name) => assert_eq!(name, "Point"),
                _ => panic!("Expected identifier"),
            }
            assert_eq!(arguments.len(), 2);
        }
        _ => panic!("Expected FunctionCall"),
    }
}

// ============================================================================
// Code Generation Tests for Structs
// ============================================================================

#[test]
fn test_codegen_struct_definition() {
    let input = "Struct[Point, [x: Int32, y: Int32]]";
    let mut parser = Parser::new(input.to_string());
    let expr = parser.parse_expression().unwrap();

    let mut codegen = RustCodeGenerator::new();
    let rust_code = codegen.generate(&expr).unwrap();

    assert!(rust_code.contains("#[derive(Debug, Clone, PartialEq)]"),
        "Should have derive macros, got: {}", rust_code);
    assert!(rust_code.contains("pub struct Point {"),
        "Should have struct definition, got: {}", rust_code);
    assert!(rust_code.contains("pub x: i32,"),
        "Should have x field, got: {}", rust_code);
    assert!(rust_code.contains("pub y: i32,"),
        "Should have y field, got: {}", rust_code);
}

#[test]
fn test_codegen_struct_with_different_types() {
    let input = "Struct[Person, [name: String, age: Int32, height: Float64]]";
    let mut parser = Parser::new(input.to_string());
    let expr = parser.parse_expression().unwrap();

    let mut codegen = RustCodeGenerator::new();
    let rust_code = codegen.generate(&expr).unwrap();

    assert!(rust_code.contains("pub struct Person {"));
    assert!(rust_code.contains("pub name: String,"));
    assert!(rust_code.contains("pub age: i32,"));
    assert!(rust_code.contains("pub height: f64,"));
}

#[test]
fn test_codegen_struct_instantiation() {
    // Parse struct definition first, then instantiation
    let input = r#"
Struct[Point, [x: Int32, y: Int32]]
Point[10, 20]
"#;

    let mut parser = Parser::new(input.to_string());
    let expr = parser.parse().unwrap();

    let mut codegen = RustCodeGenerator::new();
    let rust_code = codegen.generate(&expr).unwrap();

    assert!(rust_code.contains("pub struct Point {"),
        "Should have struct definition, got: {}", rust_code);
    assert!(rust_code.contains("Point { x: 10, y: 20 }"),
        "Should have struct instantiation, got: {}", rust_code);
}

#[test]
fn test_codegen_struct_instantiation_with_expressions() {
    let input = r#"
Struct[Point, [x: Int32, y: Int32]]
Point[5 + 5, 10 * 2]
"#;

    let mut parser = Parser::new(input.to_string());
    let expr = parser.parse().unwrap();

    let mut codegen = RustCodeGenerator::new();
    let rust_code = codegen.generate(&expr).unwrap();

    assert!(rust_code.contains("Point { x: (5 + 5), y: (10 * 2) }"),
        "Should have struct instantiation with expressions, got: {}", rust_code);
}

#[test]
fn test_codegen_struct_in_print() {
    let input = r#"
Struct[Point, [x: Int32, y: Int32]]
Print["Point:", Point[10, 20]]
"#;

    let mut parser = Parser::new(input.to_string());
    let expr = parser.parse().unwrap();

    let mut codegen = RustCodeGenerator::new();
    let rust_code = codegen.generate(&expr).unwrap();

    assert!(rust_code.contains("pub struct Point {"));
    assert!(rust_code.contains("Point { x: 10, y: 20 }"));
    assert!(rust_code.contains("println!"));
}

// ============================================================================
// Integration Tests
// ============================================================================

#[test]
fn test_struct_definition_and_usage() {
    let input = r#"
Struct[Rectangle, [width: Int32, height: Int32]]
Print["Rectangle:", Rectangle[100, 50]]
"#;

    let mut parser = Parser::new(input.to_string());
    let expr = parser.parse().unwrap();

    let mut codegen = RustCodeGenerator::new();
    let rust_code = codegen.generate(&expr).unwrap();

    // Verify struct definition
    assert!(rust_code.contains("#[derive(Debug, Clone, PartialEq)]"));
    assert!(rust_code.contains("pub struct Rectangle {"));
    assert!(rust_code.contains("pub width: i32,"));
    assert!(rust_code.contains("pub height: i32,"));

    // Verify struct usage
    assert!(rust_code.contains("Rectangle { width: 100, height: 50 }"));
    assert!(rust_code.contains("fn main() {"));
}

#[test]
fn test_multiple_struct_definitions() {
    let input = r#"
Struct[Point, [x: Int32, y: Int32]]
Struct[Circle, [center: Point, radius: Float64]]
Print["Done"]
"#;

    let mut parser = Parser::new(input.to_string());
    let expr = parser.parse().unwrap();

    let mut codegen = RustCodeGenerator::new();
    let rust_code = codegen.generate(&expr).unwrap();

    assert!(rust_code.contains("pub struct Point {"));
    assert!(rust_code.contains("pub struct Circle {"));
    // Note: Custom type "Point" will appear as-is in the Circle struct
    assert!(rust_code.contains("pub center: Point,"));
    assert!(rust_code.contains("pub radius: f64,"));
}

#[test]
fn test_struct_with_bool_field() {
    let input = "Struct[Flag, [isActive: Bool]]";
    let mut parser = Parser::new(input.to_string());
    let expr = parser.parse_expression().unwrap();

    let mut codegen = RustCodeGenerator::new();
    let rust_code = codegen.generate(&expr).unwrap();

    assert!(rust_code.contains("pub is_active: bool,"));
}

#[test]
fn test_struct_field_name_conversion() {
    // Test that PascalCase/camelCase field names are converted to snake_case
    let input = "Struct[Data, [firstName: String, LastName: String]]";
    let mut parser = Parser::new(input.to_string());
    let expr = parser.parse_expression().unwrap();

    let mut codegen = RustCodeGenerator::new();
    let rust_code = codegen.generate(&expr).unwrap();

    assert!(rust_code.contains("pub first_name: String,"));
    assert!(rust_code.contains("pub last_name: String,"));
}

#[test]
fn test_empty_struct() {
    let input = "Struct[Empty, []]";
    let mut parser = Parser::new(input.to_string());
    let result = parser.parse_expression();

    assert!(result.is_some());

    match result.unwrap() {
        Expression::StructDefinition { name, fields } => {
            assert_eq!(name, "Empty");
            assert_eq!(fields.len(), 0);
        }
        _ => panic!("Expected StructDefinition"),
    }
}

#[test]
fn test_codegen_empty_struct() {
    let input = "Struct[Empty, []]";
    let mut parser = Parser::new(input.to_string());
    let expr = parser.parse_expression().unwrap();

    let mut codegen = RustCodeGenerator::new();
    let rust_code = codegen.generate(&expr).unwrap();

    assert!(rust_code.contains("pub struct Empty {"));
}
