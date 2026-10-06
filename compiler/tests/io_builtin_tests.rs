use w::ast::Type;
use w::parser::Parser;
use w::rust_codegen::RustCodeGenerator;
use w::type_inference::TypeInference;

#[test]
fn test_readline_type_inference() {
    let mut parser = Parser::new("ReadLine[]".to_string());
    let expr = parser.parse_expression().unwrap();

    let mut inferencer = TypeInference::new();
    let result_type = inferencer.infer_expression(&expr).unwrap();

    match result_type {
        Type::Option(inner) => match *inner {
            Type::String => {}
            _ => panic!("Expected Option[String], got Option[{:?}]", inner),
        },
        _ => panic!("Expected Option type, got {:?}", result_type),
    }
}

#[test]
fn test_readline_arity_error() {
    let mut parser = Parser::new("ReadLine[1]".to_string());
    let expr = parser.parse_expression().unwrap();

    let mut inferencer = TypeInference::new();
    let result = inferencer.infer_expression(&expr);
    assert!(result.is_err(), "ReadLine should reject arguments");
}

#[test]
fn test_readline_codegen() {
    let mut parser = Parser::new("ReadLine[]".to_string());
    let expr = parser.parse_expression().unwrap();

    let mut codegen = RustCodeGenerator::new();
    let rust_code = codegen.generate(&expr).unwrap();

    assert!(
        rust_code.contains("std::io::stdin()"),
        "ReadLine should use stdin, got: {}",
        rust_code
    );
    assert!(
        rust_code.contains("read_line"),
        "ReadLine should call read_line, got: {}",
        rust_code
    );
    assert!(
        rust_code.contains("None"),
        "ReadLine should return None on EOF/error, got: {}",
        rust_code
    );
    assert!(
        rust_code.contains("Some"),
        "ReadLine should return Some on success, got: {}",
        rust_code
    );
}

#[test]
fn test_append_type_inference() {
    let mut parser = Parser::new("Append[[1, 2, 3], 4]".to_string());
    let expr = parser.parse_expression().unwrap();

    let mut inferencer = TypeInference::new();
    let result_type = inferencer.infer_expression(&expr).unwrap();

    match result_type {
        Type::List(_) => {}
        _ => panic!("Expected List type, got {:?}", result_type),
    }
}

#[test]
fn test_append_arity_error() {
    let mut parser = Parser::new("Append[[1, 2]]".to_string());
    let expr = parser.parse_expression().unwrap();

    let mut inferencer = TypeInference::new();
    let result = inferencer.infer_expression(&expr);
    assert!(result.is_err(), "Append should require 2 arguments");
}

#[test]
fn test_append_codegen() {
    let mut parser = Parser::new("Append[[1, 2, 3], 4]".to_string());
    let expr = parser.parse_expression().unwrap();

    let mut codegen = RustCodeGenerator::new();
    let rust_code = codegen.generate(&expr).unwrap();

    assert!(
        rust_code.contains("let mut l ="),
        "Append should create mutable binding, got: {}",
        rust_code
    );
    assert!(
        rust_code.contains(".push("),
        "Append should use push, got: {}",
        rust_code
    );
}

#[test]
fn test_append_in_print() {
    let mut parser = Parser::new("Print[Append[[1, 2], 3]]".to_string());
    let expr = parser.parse_expression().unwrap();

    let mut codegen = RustCodeGenerator::new();
    let rust_code = codegen.generate(&expr).unwrap();

    assert!(
        rust_code.contains("{:?}"),
        "Print with Append should use debug formatter, got: {}",
        rust_code
    );
}

#[test]
fn test_readline_in_match() {
    let input = r#"Match[ReadLine[], [Some[line], Print[line]], [None, Print["EOF"]]]"#;
    let mut parser = Parser::new(input.to_string());
    let expr = parser.parse_expression().unwrap();

    let mut inferencer = TypeInference::new();
    let result = inferencer.infer_expression(&expr);
    assert!(
        result.is_ok(),
        "ReadLine in Match should type check: {:?}",
        result.err()
    );
}

#[test]
fn test_append_chained() {
    let input = "Append[Append[[1], 2], 3]";
    let mut parser = Parser::new(input.to_string());
    let expr = parser.parse_expression().unwrap();

    let mut inferencer = TypeInference::new();
    let result_type = inferencer.infer_expression(&expr).unwrap();

    match result_type {
        Type::List(_) => {}
        _ => panic!(
            "Expected List type for chained Append, got {:?}",
            result_type
        ),
    }
}
