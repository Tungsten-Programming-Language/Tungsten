use w::ast::Type;
use w::parser::Parser;
use w::rust_codegen::RustCodeGenerator;
use w::type_inference::TypeInference;

// ============================================================================
// ToString
// ============================================================================

#[test]
fn test_tostring_type_inference() {
    let mut parser = Parser::new("ToString[42]".to_string());
    let expr = parser.parse_expression().unwrap();

    let mut inferencer = TypeInference::new();
    let result_type = inferencer.infer_expression(&expr).unwrap();
    assert_eq!(result_type, Type::String, "ToString should infer String");
}

#[test]
fn test_tostring_arity_error() {
    let mut parser = Parser::new("ToString[]".to_string());
    let expr = parser.parse_expression().unwrap();

    let mut inferencer = TypeInference::new();
    assert!(
        inferencer.infer_expression(&expr).is_err(),
        "ToString should require exactly 1 argument"
    );
}

#[test]
fn test_tostring_codegen() {
    let mut parser = Parser::new("ToString[42]".to_string());
    let expr = parser.parse_expression().unwrap();

    let mut codegen = RustCodeGenerator::new();
    let rust_code = codegen.generate(&expr).unwrap();
    assert!(
        rust_code.contains("(42).to_string()"),
        "ToString should emit .to_string(), got: {}",
        rust_code
    );
}

#[test]
fn test_tostring_of_expression() {
    let mut parser = Parser::new("ToString[1 + 1]".to_string());
    let expr = parser.parse_expression().unwrap();

    let mut codegen = RustCodeGenerator::new();
    let rust_code = codegen.generate(&expr).unwrap();
    assert!(
        rust_code.contains("((1 + 1)).to_string()"),
        "ToString of an expression got: {}",
        rust_code
    );
}

// ============================================================================
// StringJoin
// ============================================================================

#[test]
fn test_stringjoin_type_inference() {
    let mut parser = Parser::new("StringJoin[[\"a\", \"b\"], \", \"]".to_string());
    let expr = parser.parse_expression().unwrap();

    let mut inferencer = TypeInference::new();
    let result_type = inferencer.infer_expression(&expr).unwrap();
    assert_eq!(result_type, Type::String, "StringJoin should infer String");
}

#[test]
fn test_stringjoin_arity_error() {
    let mut parser = Parser::new("StringJoin[[\"a\"]]".to_string());
    let expr = parser.parse_expression().unwrap();

    let mut inferencer = TypeInference::new();
    assert!(
        inferencer.infer_expression(&expr).is_err(),
        "StringJoin should require exactly 2 arguments"
    );
}

#[test]
fn test_stringjoin_codegen() {
    let mut parser = Parser::new("StringJoin[[\"a\", \"b\"], \"-\"]".to_string());
    let expr = parser.parse_expression().unwrap();

    let mut codegen = RustCodeGenerator::new();
    let rust_code = codegen.generate(&expr).unwrap();
    assert!(
        rust_code.contains(".join("),
        "StringJoin should emit .join(), got: {}",
        rust_code
    );
    assert!(
        rust_code.contains(".as_str()"),
        "StringJoin separator should use .as_str(), got: {}",
        rust_code
    );
}

#[test]
fn test_stringjoin_empty_separator_numbering() {
    // The exact pattern the todo app uses to render "1. buy milk"
    let input = r#"StringJoin[[ToString[1], ". ", "buy milk"], ""]"#;
    let mut parser = Parser::new(input.to_string());
    let expr = parser.parse_expression().unwrap();

    let mut inference = TypeInference::new();
    assert!(
        inference.infer_expression(&expr).is_ok(),
        "StringJoin with ToString and empty separator should type check"
    );
}

// ============================================================================
// StringSplit
// ============================================================================

#[test]
fn test_stringsplit_type_inference() {
    let mut parser = Parser::new("StringSplit[\"a,b\", \",\"]".to_string());
    let expr = parser.parse_expression().unwrap();

    let mut inferencer = TypeInference::new();
    let result_type = inferencer.infer_expression(&expr).unwrap();
    match result_type {
        Type::List(inner) => assert_eq!(
            *inner,
            Type::String,
            "StringSplit should infer List[String]"
        ),
        _ => panic!("StringSplit should infer List[String], got {:?}", result_type),
    }
}

#[test]
fn test_stringsplit_arity_error() {
    let mut parser = Parser::new("StringSplit[\"a,b\"]".to_string());
    let expr = parser.parse_expression().unwrap();

    let mut inferencer = TypeInference::new();
    assert!(
        inferencer.infer_expression(&expr).is_err(),
        "StringSplit should require exactly 2 arguments"
    );
}

#[test]
fn test_stringsplit_codegen() {
    let mut parser = Parser::new("StringSplit[\"a,b\", \",\"]".to_string());
    let expr = parser.parse_expression().unwrap();

    let mut codegen = RustCodeGenerator::new();
    let rust_code = codegen.generate(&expr).unwrap();
    assert!(
        rust_code.contains(".split("),
        "StringSplit should emit .split(), got: {}",
        rust_code
    );
    assert!(
        rust_code.contains("collect::<Vec<String>>()"),
        "StringSplit should collect into Vec<String>, got: {}",
        rust_code
    );
}

#[test]
fn test_split_then_nth_string_list() {
    // Splitting produces a Vec<String>; Nth on it must work without E0507
    let input = r#"Nth[StringSplit["a,b,c", ","], 0]"#;
    let mut parser = Parser::new(input.to_string());
    let expr = parser.parse_expression().unwrap();

    let mut codegen = RustCodeGenerator::new();
    let rust_code = codegen.generate(&expr).unwrap();
    assert!(
        rust_code.contains(".clone()"),
        "Nth on StringSplit result should clone element, got: {}",
        rust_code
    );
}

// ============================================================================
// Print integration: StringSplit returns Vec, needs {:?} formatting
// ============================================================================

#[test]
fn test_print_stringsplit_uses_debug_format() {
    let mut parser = Parser::new("Print[StringSplit[\"a,b\", \",\"]]".to_string());
    let expr = parser.parse_expression().unwrap();

    let mut codegen = RustCodeGenerator::new();
    let rust_code = codegen.generate(&expr).unwrap();
    assert!(
        rust_code.contains("println!(\"{:?}\""),
        "Print of StringSplit should use debug format, got: {}",
        rust_code
    );
}