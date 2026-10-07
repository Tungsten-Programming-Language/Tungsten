use w::ast::Type;
use w::parser::Parser;
use w::rust_codegen::RustCodeGenerator;
use w::type_inference::TypeInference;

// ============================================================================
// If[cond, then, else] - was completely broken (emitted nonexistent if_ identifier)
// ============================================================================

#[test]
fn test_if_codegen_uses_rust_if() {
    // Regression: If used to generate "if_(true, ...)" which never compiled
    let input = r#"If[1 == 1, Print["yes"], Print["no"]]"#;
    let mut parser = Parser::new(input.to_string());
    let expr = parser.parse().unwrap();

    let mut codegen = RustCodeGenerator::new();
    let rust_code = codegen.generate(&expr).unwrap();
    assert!(
        !rust_code.contains("if_("),
        "If must not generate if_() identifier, got: {}",
        rust_code
    );
    assert!(
        rust_code.contains("if (1 == 1) {"),
        "If should generate a Rust if expression, got: {}",
        rust_code
    );
    assert!(
        rust_code.contains(" else {"),
        "If should generate an else branch, got: {}",
        rust_code
    );
}

#[test]
fn test_if_two_arg_codegen() {
    // If[cond, then] - no else arm, should generate "else { () }"
    let input = r#"If[1 == 2, Print["never"]]"#;
    let mut parser = Parser::new(input.to_string());
    let expr = parser.parse().unwrap();

    let mut codegen = RustCodeGenerator::new();
    let rust_code = codegen.generate(&expr).unwrap();
    assert!(
        rust_code.contains("else { () }"),
        "Two-arg If should emit an else unit block, got: {}",
        rust_code
    );
}

#[test]
fn test_if_nested_codegen() {
    let input = r#"If[2 > 1, If[3 > 2, Print["both"]], Print["no"]]"#;
    let mut parser = Parser::new(input.to_string());
    let expr = parser.parse().unwrap();

    let mut codegen = RustCodeGenerator::new();
    let rust_code = codegen.generate(&expr).unwrap();
    assert!(
        rust_code.matches("if (").count() >= 2,
        "Nested If should emit multiple if expressions, got: {}",
        rust_code
    );
}

#[test]
fn test_if_type_inference() {
    let input = r#"If[true, 42, 0]"#;
    let mut parser = Parser::new(input.to_string());
    let expr = parser.parse_expression().unwrap();

    let mut inferencer = TypeInference::new();
    let result_type = inferencer.infer_expression(&expr).unwrap();
    assert_eq!(
        result_type,
        Type::Int32,
        "If should infer the type of its then-branch, got {:?}",
        result_type
    );
}

#[test]
fn test_if_arity_error() {
    let input = r#"If[true]"#;
    let mut parser = Parser::new(input.to_string());
    let expr = parser.parse_expression().unwrap();

    let mut inferencer = TypeInference::new();
    assert!(
        inferencer.infer_expression(&expr).is_err(),
        "If should require at least 2 arguments"
    );
}

// ============================================================================
// Cond/If arms with mixed Rust types (WriteFile -> Result vs Print -> ())
// Regression: arms had to share a Rust type; unit-normalization appends ";"
// ============================================================================

#[test]
fn test_cond_mixed_arms_unit_normalized() {
    // Cond with a unit arm (Print) and a Result arm (WriteFile) must compile:
    // the inferred Cond type is unit, so arms get ";" appended
    let input = r#"Cond[[1 == 1 Print["x"]][true WriteFile["a.txt", "y"]]]"#;
    let mut parser = Parser::new(input.to_string());
    let expr = parser.parse().unwrap();

    let mut codegen = RustCodeGenerator::new();
    let rust_code = codegen.generate(&expr).unwrap();
    assert!(
        rust_code.contains("std::fs::write"),
        "Cond should contain the WriteFile call, got: {}",
        rust_code
    );
    // Unit-normalized arms must end in ";"
    assert!(
        rust_code.contains("println!(\"{}\", \"x\".to_string());"),
        "Unit-typed Cond arm (Print) should end with ;, got: {}",
        rust_code
    );
    assert!(
        rust_code.contains(".map_err(|e| e.to_string());"),
        "Result-typed Cond arm (WriteFile) should end with ;, got: {}",
        rust_code
    );
}

#[test]
fn test_cond_value_arms_not_normalized() {
    // A Cond whose first arm infers a non-unit value must NOT get ";" appended
    // (otherwise value-returning Conds like CountNodes break)
    let input = r#"Cond[[depth <= 0 1][true 2]]"#;
    let mut parser = Parser::new(input.to_string());
    let expr = parser.parse().unwrap();

    let mut codegen = RustCodeGenerator::new();
    let rust_code = codegen.generate(&expr).unwrap();
    // Both arms must remain values: "1" and "2" not followed by ";"
    assert!(
        rust_code.contains("if (depth <= 0) {"),
        "Value-typed Cond should emit an if, got: {}",
        rust_code
    );
    assert!(
        !rust_code.contains("1;") && !rust_code.contains("2;"),
        "Value-typed Cond arms must not be semicolon-terminated, got: {}",
        rust_code
    );
}

#[test]
fn test_if_mixed_arms_unit_normalized() {
    // If[cond, WriteFile, Print] - then is Result, else is unit
    let input = r#"If[true, WriteFile["a.txt", "x"], Print["y"]]"#;
    let mut parser = Parser::new(input.to_string());
    let expr = parser.parse().unwrap();

    let mut codegen = RustCodeGenerator::new();
    let rust_code = codegen.generate(&expr).unwrap();
    assert!(
        rust_code.contains(".map_err(|e| e.to_string());"),
        "If then-arm (WriteFile) should be unit-normalized with ;, got: {}",
        rust_code
    );
    assert!(
        rust_code.contains("println!(\"{}\", \"y\".to_string());"),
        "If else-arm (Print) should be unit-normalized with ;, got: {}",
        rust_code
    );
}

#[test]
fn test_cond_parse_no_commas_between_pairs() {
    // Parser quirk: Cond pairs are space-separated, not comma-separated
    let input = r#"Cond[[true Print["a"]][true Print["b"]]]"#;
    let mut parser = Parser::new(input.to_string());
    assert!(parser.parse().is_some(), "Cond pairs must parse without commas");
}

#[test]
fn test_cond_with_comma_pairs_fails() {
    // Commas between Cond pairs must fail to parse (documented syntax)
    let input = r#"Cond[[true Print["a"]], [true Print["b"]]]"#;
    let mut parser = Parser::new(input.to_string());
    assert!(parser.parse().is_none(), "Comma-separated Cond pairs should not parse");
}

// ============================================================================
// While loop with single-expression body
// ============================================================================

#[test]
fn test_cond_with_bound_identifier_not_normalized() {
    // Regression: binary_trees uses Cond[[minDepth + 2 > n minDepth + 2][true n]]
    // where n/minDepth are With-bound. Bare identifiers not in parameters used
    // to infer "()" and falsely trigger unit normalization (n;), breaking the
    // value-typed Cond. With-bound vars are Unknown and must not normalize.
    let input = r#"With[{n = 7},
    With[{minDepth = 4},
        Cond[[minDepth + 2 > n minDepth + 2][true n]]
    ]
]"#;
    let mut parser = Parser::new(input.to_string());
    let expr = parser.parse().unwrap();

    let mut codegen = RustCodeGenerator::new();
    let rust_code = codegen.generate(&expr).unwrap();
    assert!(
        !rust_code.contains("n;"),
        "With-bound identifier arm must not be semicolon-terminated, got: {}",
        rust_code
    );
    assert!(
        rust_code.contains("if ((min_depth + 2) > n) {"),
        "Value-typed Cond should still emit an if, got: {}",
        rust_code
    );
}

#[test]
fn test_while_body_must_be_single_expression() {
    // Regression guide: While[cond, body] takes exactly one body expression
    let input = r#"While[true, Match[ReadLine[], [Some[c], Print[c]], [None, Break[]]]]"#;
    let mut parser = Parser::new(input.to_string());
    let expr = parser.parse().unwrap();

    let mut codegen = RustCodeGenerator::new();
    let rust_code = codegen.generate(&expr).unwrap();
    assert!(
        rust_code.contains("while true {"),
        "While should generate a Rust while loop, got: {}",
        rust_code
    );
    assert!(
        rust_code.contains("break"),
        "While body Match with Break should generate break, got: {}",
        rust_code
    );
}