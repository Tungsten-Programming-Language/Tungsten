use w::lexer::Lexer;
use w::parser::Parser;
use w::rust_codegen::RustCodeGenerator;
use w::type_inference::TypeInference;

#[test]
fn test_parse_do_simple() {
    let input = "Do[Print[1], 3]".to_string();
    let mut parser = Parser::new(input);
    let expr = parser.parse().expect("Failed to parse");
    let rust_code = RustCodeGenerator::new()
        .generate(&expr)
        .expect("Failed to generate");
    assert!(rust_code.contains("for _ in 1..=3"));
}

#[test]
fn test_parse_do_with_variable() {
    let input = "Do[Print[i], {i, 5}]".to_string();
    let mut parser = Parser::new(input);
    let expr = parser.parse().expect("Failed to parse");
    let rust_code = RustCodeGenerator::new()
        .generate(&expr)
        .expect("Failed to generate");
    assert!(rust_code.contains("for i in 1..=5"));
}

#[test]
fn test_parse_do_with_range() {
    let input = "Do[Print[i], {i, 2, 10}]".to_string();
    let mut parser = Parser::new(input);
    let expr = parser.parse().expect("Failed to parse");
    let rust_code = RustCodeGenerator::new()
        .generate(&expr)
        .expect("Failed to generate");
    assert!(rust_code.contains("for i in 2..=10"));
}

#[test]
fn test_parse_do_with_step() {
    let input = "Do[Print[i], {i, 1, 10, 2}]".to_string();
    let mut parser = Parser::new(input);
    let expr = parser.parse().expect("Failed to parse");
    let rust_code = RustCodeGenerator::new()
        .generate(&expr)
        .expect("Failed to generate");
    assert!(rust_code.contains("step_by"));
}

#[test]
fn test_parse_while() {
    let input = "While[true, Print[1]]".to_string();
    let mut parser = Parser::new(input);
    let expr = parser.parse().expect("Failed to parse");
    let rust_code = RustCodeGenerator::new()
        .generate(&expr)
        .expect("Failed to generate");
    assert!(rust_code.contains("while true"));
}

#[test]
fn test_parse_break() {
    let input = "Break[]".to_string();
    let mut parser = Parser::new(input);
    let expr = parser.parse().expect("Failed to parse");
    let rust_code = RustCodeGenerator::new()
        .generate(&expr)
        .expect("Failed to generate");
    assert!(rust_code.contains("break"));
}

#[test]
fn test_parse_continue() {
    let input = "Continue[]".to_string();
    let mut parser = Parser::new(input);
    let expr = parser.parse().expect("Failed to parse");
    let rust_code = RustCodeGenerator::new()
        .generate(&expr)
        .expect("Failed to generate");
    assert!(rust_code.contains("continue"));
}

#[test]
fn test_do_in_function() {
    let input = "Loop[n] := Do[Print[i], {i, n}]".to_string();
    let mut parser = Parser::new(input);
    let expr = parser.parse().expect("Failed to parse");
    let rust_code = RustCodeGenerator::new()
        .generate(&expr)
        .expect("Failed to generate");
    // 'loop' is a reserved keyword in Rust, so it becomes 'loop_'
    assert!(rust_code.contains("fn loop_"));
    assert!(rust_code.contains("for i in 1..=n"));
}

#[test]
fn test_nested_do() {
    let input = "Do[Do[Print[i + j], {j, 3}], {i, 2}]".to_string();
    let mut parser = Parser::new(input);
    let expr = parser.parse().expect("Failed to parse");
    let rust_code = RustCodeGenerator::new()
        .generate(&expr)
        .expect("Failed to generate");
    assert!(rust_code.contains("for i in 1..=2"));
    assert!(rust_code.contains("for j in 1..=3"));
}
