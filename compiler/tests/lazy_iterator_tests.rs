use w::parser::Parser;
use w::rust_codegen::RustCodeGenerator;

#[test]
fn test_codegen_lazy_map() {
    let mut parser = Parser::new("LazyMap[Function[{x}, x * 2], [1, 2, 3]]".to_string());
    let expr = parser.parse_expression().unwrap();

    let mut codegen = RustCodeGenerator::new();
    let rust_code = codegen.generate(&expr).unwrap();

    assert!(
        rust_code.contains(".into_iter().map("),
        "LazyMap should generate iterator without collect, got: {}",
        rust_code
    );
    assert!(
        !rust_code.contains(".collect::<Vec<_>>()"),
        "LazyMap should NOT contain collect, got: {}",
        rust_code
    );
}

#[test]
fn test_codegen_lazy_map_with_shorthand() {
    let mut parser = Parser::new("LazyMap[x -> x * 2, [1, 2, 3]]".to_string());
    let expr = parser.parse_expression().unwrap();

    let mut codegen = RustCodeGenerator::new();
    let rust_code = codegen.generate(&expr).unwrap();

    assert!(
        rust_code.contains(".into_iter().map(|x|"),
        "LazyMap with shorthand should work, got: {}",
        rust_code
    );
    assert!(
        !rust_code.contains(".collect::<Vec<_>>()"),
        "LazyMap should NOT contain collect, got: {}",
        rust_code
    );
}

#[test]
fn test_codegen_lazy_filter() {
    let mut parser = Parser::new("LazyFilter[Function[{x}, x > 2], [1, 2, 3, 4, 5]]".to_string());
    let expr = parser.parse_expression().unwrap();

    let mut codegen = RustCodeGenerator::new();
    let rust_code = codegen.generate(&expr).unwrap();

    assert!(
        rust_code.contains(".into_iter().filter("),
        "LazyFilter should generate iterator without collect, got: {}",
        rust_code
    );
    assert!(
        !rust_code.contains(".collect::<Vec<_>>()"),
        "LazyFilter should NOT contain collect, got: {}",
        rust_code
    );
}

#[test]
fn test_codegen_lazy_filter_with_shorthand() {
    let mut parser = Parser::new("LazyFilter[x -> x > 2, [1, 2, 3, 4, 5]]".to_string());
    let expr = parser.parse_expression().unwrap();

    let mut codegen = RustCodeGenerator::new();
    let rust_code = codegen.generate(&expr).unwrap();

    assert!(
        rust_code.contains(".into_iter().filter(|&x|"),
        "LazyFilter with shorthand should work, got: {}",
        rust_code
    );
    assert!(
        !rust_code.contains(".collect::<Vec<_>>()"),
        "LazyFilter should NOT contain collect, got: {}",
        rust_code
    );
}

#[test]
fn test_codegen_collect() {
    let mut parser = Parser::new("Collect[LazyMap[x -> x * 2, [1, 2, 3]]]".to_string());
    let expr = parser.parse_expression().unwrap();

    let mut codegen = RustCodeGenerator::new();
    let rust_code = codegen.generate(&expr).unwrap();

    assert!(
        rust_code.contains(".collect::<Vec<_>>()"),
        "Collect should generate collect call, got: {}",
        rust_code
    );
    assert!(
        rust_code.contains(".into_iter().map("),
        "Collect with LazyMap should have map, got: {}",
        rust_code
    );
}

#[test]
fn test_codegen_lazy_chain_map_filter_collect() {
    let input = r#"[1, 2, 3, 4, 5] |> LazyMap[x -> x * 2] |> LazyFilter[x -> x > 5] |> Collect[]"#;
    let mut parser = Parser::new(input.to_string());
    let expr = parser.parse_expression().unwrap();

    let mut codegen = RustCodeGenerator::new();
    let rust_code = codegen.generate(&expr).unwrap();

    assert!(
        rust_code.contains(".into_iter().map("),
        "Should contain map, got: {}",
        rust_code
    );
    assert!(
        rust_code.contains(".filter("),
        "Should contain filter, got: {}",
        rust_code
    );
    assert!(
        rust_code.contains(".collect::<Vec<_>>()"),
        "Should end with collect, got: {}",
        rust_code
    );
}

#[test]
fn test_codegen_lazy_chain_with_print() {
    let input = r#"Print[Collect[[1, 2, 3] |> LazyMap[x -> x * 2]]]"#;
    let mut parser = Parser::new(input.to_string());
    let expr = parser.parse_expression().unwrap();

    let mut codegen = RustCodeGenerator::new();
    let rust_code = codegen.generate(&expr).unwrap();

    assert!(
        rust_code.contains(".into_iter().map("),
        "Should contain map, got: {}",
        rust_code
    );
    assert!(
        rust_code.contains(".collect::<Vec<_>>()"),
        "Should contain collect, got: {}",
        rust_code
    );
    assert!(
        rust_code.contains("{:?}"),
        "Print should use debug format for Vec, got: {}",
        rust_code
    );
}

#[test]
fn test_lazy_map_with_pipe() {
    let input = "[10, 20, 30] |> LazyMap[x -> x + 5]";
    let mut parser = Parser::new(input.to_string());
    let expr = parser.parse_expression().unwrap();

    let mut codegen = RustCodeGenerator::new();
    let rust_code = codegen.generate(&expr).unwrap();

    assert!(
        rust_code.contains("vec![10, 20, 30].into_iter().map(|x|"),
        "LazyMap with pipe should work, got: {}",
        rust_code
    );
    assert!(
        !rust_code.contains(".collect::<Vec<_>>()"),
        "LazyMap should NOT contain collect, got: {}",
        rust_code
    );
}

#[test]
fn test_lazy_filter_with_pipe() {
    let input = "[1, 2, 3, 4] |> LazyFilter[x -> x > 2]";
    let mut parser = Parser::new(input.to_string());
    let expr = parser.parse_expression().unwrap();

    let mut codegen = RustCodeGenerator::new();
    let rust_code = codegen.generate(&expr).unwrap();

    assert!(
        rust_code.contains("vec![1, 2, 3, 4].into_iter().filter(|&x|"),
        "LazyFilter with pipe should work, got: {}",
        rust_code
    );
    assert!(
        !rust_code.contains(".collect::<Vec<_>>()"),
        "LazyFilter should NOT contain collect, got: {}",
        rust_code
    );
}

#[test]
fn test_compare_eager_vs_lazy_map() {
    let eager_input = "[1, 2, 3] |> Map[x -> x * 2]";
    let lazy_input = "[1, 2, 3] |> LazyMap[x -> x * 2]";

    let mut parser = Parser::new(eager_input.to_string());
    let eager_expr = parser.parse_expression().unwrap();
    let mut codegen = RustCodeGenerator::new();
    let eager_code = codegen.generate(&eager_expr).unwrap();

    let mut parser = Parser::new(lazy_input.to_string());
    let lazy_expr = parser.parse_expression().unwrap();
    let mut codegen = RustCodeGenerator::new();
    let lazy_code = codegen.generate(&lazy_expr).unwrap();

    assert!(
        eager_code.contains(".collect::<Vec<_>>()"),
        "Eager Map should contain collect, got: {}",
        eager_code
    );
    assert!(
        !lazy_code.contains(".collect::<Vec<_>>()"),
        "LazyMap should NOT contain collect, got: {}",
        lazy_code
    );
}

#[test]
fn test_compare_eager_vs_lazy_filter() {
    let eager_input = "[1, 2, 3, 4] |> Filter[x -> x > 2]";
    let lazy_input = "[1, 2, 3, 4] |> LazyFilter[x -> x > 2]";

    let mut parser = Parser::new(eager_input.to_string());
    let eager_expr = parser.parse_expression().unwrap();
    let mut codegen = RustCodeGenerator::new();
    let eager_code = codegen.generate(&eager_expr).unwrap();

    let mut parser = Parser::new(lazy_input.to_string());
    let lazy_expr = parser.parse_expression().unwrap();
    let mut codegen = RustCodeGenerator::new();
    let lazy_code = codegen.generate(&lazy_expr).unwrap();

    assert!(
        eager_code.contains(".collect::<Vec<_>>()"),
        "Eager Filter should contain collect, got: {}",
        eager_code
    );
    assert!(
        !lazy_code.contains(".collect::<Vec<_>>()"),
        "LazyFilter should NOT contain collect, got: {}",
        lazy_code
    );
}

#[test]
fn test_multiple_lazy_operations_before_collect() {
    let input = "[1, 2, 3, 4, 5, 6] |> LazyMap[x -> x * x] |> LazyFilter[x -> x > 10] |> LazyMap[x -> x / 2] |> Collect[]";
    let mut parser = Parser::new(input.to_string());
    let expr = parser.parse_expression().unwrap();

    let mut codegen = RustCodeGenerator::new();
    let rust_code = codegen.generate(&expr).unwrap();

    let map_count = rust_code.matches(".map(").count();
    let filter_count = rust_code.matches(".filter(").count();
    let collect_count = rust_code.matches(".collect::<Vec<_>>()").count();

    assert_eq!(
        map_count, 2,
        "Should have 2 map operations, got {} in: {}",
        map_count, rust_code
    );
    assert_eq!(
        filter_count, 1,
        "Should have 1 filter operation, got {} in: {}",
        filter_count, rust_code
    );
    assert_eq!(
        collect_count, 1,
        "Should have exactly 1 collect at the end, got {} in: {}",
        collect_count, rust_code
    );
}
