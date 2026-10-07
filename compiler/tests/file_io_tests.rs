use w::ast::Type;
use w::parser::Parser;
use w::rust_codegen::RustCodeGenerator;
use w::type_inference::TypeInference;

#[test]
fn test_readfile_type_inference() {
    let mut parser = Parser::new("ReadFile[\"todo.txt\"]".to_string());
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
fn test_readfile_arity_error() {
    let mut parser = Parser::new("ReadFile[]".to_string());
    let expr = parser.parse_expression().unwrap();

    let mut inferencer = TypeInference::new();
    let result = inferencer.infer_expression(&expr);
    assert!(result.is_err(), "ReadFile should require exactly 1 argument");
}

#[test]
fn test_readfile_codegen() {
    let mut parser = Parser::new("ReadFile[\"todo.txt\"]".to_string());
    let expr = parser.parse_expression().unwrap();

    let mut codegen = RustCodeGenerator::new();
    let rust_code = codegen.generate(&expr).unwrap();

    assert!(
        rust_code.contains("std::fs::read_to_string"),
        "ReadFile should use std::fs::read_to_string, got: {}",
        rust_code
    );
    assert!(
        rust_code.contains(".ok()"),
        "ReadFile should map to Option via .ok(), got: {}",
        rust_code
    );
}

#[test]
fn test_writefile_type_inference() {
    let mut parser = Parser::new("WriteFile[\"todo.txt\", \"hello\"]".to_string());
    let expr = parser.parse_expression().unwrap();

    let mut inferencer = TypeInference::new();
    let result_type = inferencer.infer_expression(&expr).unwrap();

    match result_type {
        Type::Result(ok_type, err_type) => {
            assert_eq!(*ok_type, Type::Tuple(vec![]), "Ok type should be unit");
            assert_eq!(*err_type, Type::String, "Err type should be String");
        }
        _ => panic!("Expected Result type, got {:?}", result_type),
    }
}

#[test]
fn test_writefile_arity_error() {
    let mut parser = Parser::new("WriteFile[\"todo.txt\"]".to_string());
    let expr = parser.parse_expression().unwrap();

    let mut inferencer = TypeInference::new();
    let result = inferencer.infer_expression(&expr);
    assert!(
        result.is_err(),
        "WriteFile should require exactly 2 arguments"
    );
}

#[test]
fn test_writefile_codegen() {
    let mut parser = Parser::new("WriteFile[\"todo.txt\", \"hello\"]".to_string());
    let expr = parser.parse_expression().unwrap();

    let mut codegen = RustCodeGenerator::new();
    let rust_code = codegen.generate(&expr).unwrap();

    assert!(
        rust_code.contains("std::fs::write"),
        "WriteFile should use std::fs::write, got: {}",
        rust_code
    );
    assert!(
        rust_code.contains("map_err"),
        "WriteFile should map errors to String via map_err, got: {}",
        rust_code
    );
}

#[test]
fn test_readfile_in_match() {
    let input = r#"Match[ReadFile["todo.txt"], [Some[content], Print[content]], [None, Print["not found"]]]"#;
    let mut parser = Parser::new(input.to_string());
    let expr = parser.parse_expression().unwrap();

    let mut inferencer = TypeInference::new();
    let result = inferencer.infer_expression(&expr);
    assert!(
        result.is_ok(),
        "ReadFile in Match should type check: {:?}",
        result.err()
    );
}