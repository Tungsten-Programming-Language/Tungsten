use w::parser::Parser;
use w::rust_codegen::RustCodeGenerator;

// ============================================================================
// infer_return_type fixes
// ============================================================================

#[test]
fn test_function_returning_match_infers_concrete_type() {
    // Regression: fn bodies whose top-level expr is Match inferred "()",
    // generating fn foo() -> () { <match returning String> } which failed rustc
    let input = r#"LoadTodos[] := Match[ReadFile["todos.txt"], [Some[c], c], [None, ""]]
Print[LoadTodos[]]"#;
    let mut parser = Parser::new(input.to_string());
    let expr = parser.parse().unwrap();

    let mut codegen = RustCodeGenerator::new();
    let rust_code = codegen.generate(&expr).unwrap();
    assert!(
        rust_code.contains("fn load_todos() -> String"),
        "Match-bodied fn should infer String return type, got: {}",
        rust_code
    );
}

#[test]
fn test_param_aware_rest_inference() {
    // Regression: Rest[lines] on List[String] inferred Vec<i64>,
    // generating fn clean_lines(...) -> Vec<i64> with a Vec<String> body (E0308)
    let input = r#"CleanLines[lines: List[String]] := Cond[[true Rest[lines]][true lines]]
Print[CleanLines[["a"]]]"#;
    let mut parser = Parser::new(input.to_string());
    let expr = parser.parse().unwrap();

    let mut codegen = RustCodeGenerator::new();
    let rust_code = codegen.generate(&expr).unwrap();
    assert!(
        rust_code.contains("fn clean_lines(lines: Vec<String>) -> Vec<String>"),
        "Rest on typed param should infer Vec<String>, got: {}",
        rust_code
    );
}

#[test]
fn test_param_aware_reverse_inference() {
    let input = r#"Rev[items: List[String]] := Reverse[items]
Print[Rev[["a"]]]"#;
    let mut parser = Parser::new(input.to_string());
    let expr = parser.parse().unwrap();

    let mut codegen = RustCodeGenerator::new();
    let rust_code = codegen.generate(&expr).unwrap();
    assert!(
        rust_code.contains("fn rev(items: Vec<String>) -> Vec<String>"),
        "Reverse on typed param should infer Vec<String>, got: {}",
        rust_code
    );
}

#[test]
fn test_param_aware_set_inference() {
    let input = r#"UpdateAt[items: List[String]] := Set[items, 0, "z"]
Print[UpdateAt[["a"]]]"#;
    let mut parser = Parser::new(input.to_string());
    let expr = parser.parse().unwrap();

    let mut codegen = RustCodeGenerator::new();
    let rust_code = codegen.generate(&expr).unwrap();
    assert!(
        rust_code.contains("fn update_at(items: Vec<String>) -> Vec<String>"),
        "Set on typed param should infer Vec<String>, got: {}",
        rust_code
    );
}

// ============================================================================
// Nth/First/Set clone fixes (E0507 move-out-of-Vec, E0382 use-after-move)
// ============================================================================

#[test]
fn test_nth_on_named_binding_clones_element() {
    // Regression: Nth[lines, i] -> lines[i] moved a String out of the Vec (E0507)
    let input = r#"With[{lines = ["a", "b"]}, Nth[lines, 0]]"#;
    let mut parser = Parser::new(input.to_string());
    let expr = parser.parse().unwrap();

    let mut codegen = RustCodeGenerator::new();
    let rust_code = codegen.generate(&expr).unwrap();
    assert!(
        rust_code.contains("lines[0 as usize].clone()"),
        "Nth on a named binding must clone (E0507 fix), got: {}",
        rust_code
    );
    assert!(
        !rust_code.contains("lines[0 as usize]\n"),
        "Nth must not emit a bare index without clone, got: {}",
        rust_code
    );
}

#[test]
fn test_first_on_named_binding_clones_element() {
    let input = r#"With[{lines = ["a", "b"]}, First[lines]]"#;
    let mut parser = Parser::new(input.to_string());
    let expr = parser.parse().unwrap();

    let mut codegen = RustCodeGenerator::new();
    let rust_code = codegen.generate(&expr).unwrap();
    assert!(
        rust_code.contains("lines[0].clone()"),
        "First on a named binding must clone (E0507 fix), got: {}",
        rust_code
    );
}

#[test]
fn test_set_clones_input_list() {
    // Regression: Set emitted "let mut l = lines" which moved the list before the
    // replacement value (referencing lines) evaluated (E0382)
    let input = r#"With[{lines = ["a", "b"]}, Set[lines, 0, StringJoin[["[x] ", Nth[lines, 0]], ""]]]"#;
    let mut parser = Parser::new(input.to_string());
    let expr = parser.parse().unwrap();

    let mut codegen = RustCodeGenerator::new();
    let rust_code = codegen.generate(&expr).unwrap();
    assert!(
        rust_code.contains("let mut l = lines.clone();"),
        "Set must clone the input list (E0382 fix), got: {}",
        rust_code
    );
    // The replacement value may reference the original list: both `lines` uses present
    assert!(
        rust_code.matches("lines").count() >= 2,
        "Set with value referencing the list should keep both uses, got: {}",
        rust_code
    );
}

#[test]
fn test_set_functional_update() {
    let input = r#"With[{lines = ["a", "b"]}, Set[lines, 1, "z"]]"#;
    let mut parser = Parser::new(input.to_string());
    let expr = parser.parse().unwrap();

    let mut codegen = RustCodeGenerator::new();
    let rust_code = codegen.generate(&expr).unwrap();
    assert!(
        rust_code.contains("l[1 as usize] = \"z\".to_string();"),
        "Set should assign at index with cloned list, got: {}",
        rust_code
    );
}

// ============================================================================
// StringSplit/Join roundtrip (todo app core)
// ============================================================================

#[test]
fn test_split_join_roundtrip_codegen() {
    let input = r#"StringJoin[StringSplit["x,y,z", ","], "-"]"#;
    let mut parser = Parser::new(input.to_string());
    let expr = parser.parse().unwrap();

    let mut codegen = RustCodeGenerator::new();
    let rust_code = codegen.generate(&expr).unwrap();
    assert!(
        rust_code.contains(".split("),
        "Roundtrip should contain split, got: {}",
        rust_code
    );
    assert!(
        rust_code.contains(".join("),
        "Roundtrip should contain join, got: {}",
        rust_code
    );
}