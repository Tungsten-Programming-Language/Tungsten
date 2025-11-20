use w::parser::Parser;
use w::ast::Expression;
use w::rust_codegen::RustCodeGenerator;

// ============================================
// Parser Tests - List Values
// ============================================

#[test]
fn test_parse_empty_list() {
    let mut parser = Parser::new("[]".to_string());
    let expr = parser.parse_expression().unwrap();

    match expr {
        Expression::List(elements) => {
            assert_eq!(elements.len(), 0, "Empty list should have 0 elements");
        }
        _ => panic!("Expected List expression, got {:?}", expr),
    }
}

#[test]
fn test_parse_list_with_numbers() {
    let mut parser = Parser::new("[1, 2, 3]".to_string());
    let expr = parser.parse_expression().unwrap();

    match expr {
        Expression::List(elements) => {
            assert_eq!(elements.len(), 3);
            for (i, elem) in elements.iter().enumerate() {
                match elem {
                    Expression::Number(n) => assert_eq!(*n, (i + 1) as i32),
                    _ => panic!("Expected number in list"),
                }
            }
        }
        _ => panic!("Expected List expression"),
    }
}

#[test]
fn test_parse_list_with_strings() {
    let mut parser = Parser::new("[\"a\", \"b\", \"c\"]".to_string());
    let expr = parser.parse_expression().unwrap();

    match expr {
        Expression::List(elements) => {
            assert_eq!(elements.len(), 3);
            let expected = vec!["a", "b", "c"];
            for (i, elem) in elements.iter().enumerate() {
                match elem {
                    Expression::String(s) => assert_eq!(s, expected[i]),
                    _ => panic!("Expected string in list"),
                }
            }
        }
        _ => panic!("Expected List expression"),
    }
}

#[test]
fn test_parse_nested_list() {
    let mut parser = Parser::new("[[1, 2], [3, 4]]".to_string());
    let expr = parser.parse_expression().unwrap();

    match expr {
        Expression::List(elements) => {
            assert_eq!(elements.len(), 2);
            for elem in &elements {
                match elem {
                    Expression::List(inner) => {
                        assert_eq!(inner.len(), 2);
                    }
                    _ => panic!("Expected nested list"),
                }
            }
        }
        _ => panic!("Expected List expression"),
    }
}

// ============================================
// Parser Tests - Map Values
// ============================================

#[test]
fn test_parse_empty_map() {
    let mut parser = Parser::new("{}".to_string());
    let expr = parser.parse_expression().unwrap();

    match expr {
        Expression::Map(entries) => {
            assert_eq!(entries.len(), 0, "Empty map should have 0 entries");
        }
        _ => panic!("Expected Map expression, got {:?}", expr),
    }
}

#[test]
fn test_parse_map_with_string_keys() {
    let mut parser = Parser::new("{\"name\": \"Alice\", \"city\": \"NYC\"}".to_string());
    let expr = parser.parse_expression().unwrap();

    match expr {
        Expression::Map(entries) => {
            assert_eq!(entries.len(), 2);
            // Check first entry
            match &entries[0] {
                (Expression::String(k), Expression::String(v)) => {
                    assert_eq!(k, "name");
                    assert_eq!(v, "Alice");
                }
                _ => panic!("Expected string key-value pair"),
            }
        }
        _ => panic!("Expected Map expression"),
    }
}

#[test]
fn test_parse_map_with_number_values() {
    let mut parser = Parser::new("{\"age\": 30, \"score\": 100}".to_string());
    let expr = parser.parse_expression().unwrap();

    match expr {
        Expression::Map(entries) => {
            assert_eq!(entries.len(), 2);
            match &entries[0] {
                (Expression::String(k), Expression::Number(v)) => {
                    assert_eq!(k, "age");
                    assert_eq!(*v, 30);
                }
                _ => panic!("Expected string key with number value"),
            }
        }
        _ => panic!("Expected Map expression"),
    }
}

// ============================================
// Code Generation Tests - Lists
// ============================================

#[test]
fn test_codegen_empty_list() {
    let mut parser = Parser::new("[]".to_string());
    let expr = parser.parse_expression().unwrap();

    let mut codegen = RustCodeGenerator::new();
    let rust_code = codegen.generate(&expr).unwrap();

    assert!(rust_code.contains("vec![]"),
        "Empty list should generate vec![], got: {}", rust_code);
}

#[test]
fn test_codegen_list_with_numbers() {
    let mut parser = Parser::new("[1, 2, 3]".to_string());
    let expr = parser.parse_expression().unwrap();

    let mut codegen = RustCodeGenerator::new();
    let rust_code = codegen.generate(&expr).unwrap();

    assert!(rust_code.contains("vec![1, 2, 3]"),
        "List should generate vec![1, 2, 3], got: {}", rust_code);
}

#[test]
fn test_codegen_nested_list() {
    let mut parser = Parser::new("[[1, 2], [3, 4]]".to_string());
    let expr = parser.parse_expression().unwrap();

    let mut codegen = RustCodeGenerator::new();
    let rust_code = codegen.generate(&expr).unwrap();

    assert!(rust_code.contains("vec![vec![1, 2], vec![3, 4]]"),
        "Nested list should generate nested vec!, got: {}", rust_code);
}

#[test]
fn test_codegen_list_in_print() {
    let mut parser = Parser::new("Print[[1, 2, 3]]".to_string());
    let expr = parser.parse_expression().unwrap();

    let mut codegen = RustCodeGenerator::new();
    let rust_code = codegen.generate(&expr).unwrap();

    // Lists should use {:?} formatter
    assert!(rust_code.contains("{:?}"),
        "Print with list should use debug formatter, got: {}", rust_code);
}

// ============================================
// Code Generation Tests - Maps
// ============================================

#[test]
fn test_codegen_empty_map() {
    let mut parser = Parser::new("{}".to_string());
    let expr = parser.parse_expression().unwrap();

    let mut codegen = RustCodeGenerator::new();
    let rust_code = codegen.generate(&expr).unwrap();

    assert!(rust_code.contains("HashMap::new()"),
        "Empty map should generate HashMap::new(), got: {}", rust_code);
}

#[test]
fn test_codegen_map_with_entries() {
    let mut parser = Parser::new("{\"key\": \"value\"}".to_string());
    let expr = parser.parse_expression().unwrap();

    let mut codegen = RustCodeGenerator::new();
    let rust_code = codegen.generate(&expr).unwrap();

    assert!(rust_code.contains("HashMap::new()") &&
            rust_code.contains("map.insert"),
        "Map should generate HashMap with insert, got: {}", rust_code);
}

#[test]
fn test_codegen_map_in_print() {
    let mut parser = Parser::new("Print[{\"a\": 1}]".to_string());
    let expr = parser.parse_expression().unwrap();

    let mut codegen = RustCodeGenerator::new();
    let rust_code = codegen.generate(&expr).unwrap();

    // Maps should use {:?} formatter
    assert!(rust_code.contains("{:?}"),
        "Print with map should use debug formatter, got: {}", rust_code);
}

// ============================================
// Integration Tests - Container Type Annotations
// ============================================

#[test]
fn test_function_with_list_parameter() {
    let input = "ProcessItems[items: List[Int32]] := items";
    let mut parser = Parser::new(input.to_string());
    let expr = parser.parse_expression().unwrap();

    let mut codegen = RustCodeGenerator::new();
    let rust_code = codegen.generate(&expr).unwrap();

    assert!(rust_code.contains("fn process_items(items: Vec<i32>)"),
        "Function with List parameter should generate Vec<i32>, got: {}", rust_code);
}

#[test]
fn test_function_with_array_parameter() {
    let input = "UseBuffer[buffer: Array[UInt8, 256]] := buffer";
    let mut parser = Parser::new(input.to_string());
    let expr = parser.parse_expression().unwrap();

    let mut codegen = RustCodeGenerator::new();
    let rust_code = codegen.generate(&expr).unwrap();

    assert!(rust_code.contains("fn use_buffer(buffer: [u8; 256])"),
        "Function with Array parameter should generate [u8; 256], got: {}", rust_code);
}

#[test]
fn test_function_with_slice_parameter() {
    let input = "ReadData[data: Slice[UInt8]] := data";
    let mut parser = Parser::new(input.to_string());
    let expr = parser.parse_expression().unwrap();

    let mut codegen = RustCodeGenerator::new();
    let rust_code = codegen.generate(&expr).unwrap();

    assert!(rust_code.contains("fn read_data(data: &[u8])"),
        "Function with Slice parameter should generate &[u8], got: {}", rust_code);
}

#[test]
fn test_function_with_hashmap_parameter() {
    let input = "UseMap[mapping: Map[String, Int32]] := mapping";
    let mut parser = Parser::new(input.to_string());
    let expr = parser.parse_expression().unwrap();

    let mut codegen = RustCodeGenerator::new();
    let rust_code = codegen.generate(&expr).unwrap();

    assert!(rust_code.contains("fn use_map(mapping: std::collections::HashMap<String, i32>)"),
        "Function with Map parameter should generate HashMap, got: {}", rust_code);
}

#[test]
fn test_function_with_hashset_parameter() {
    let input = "UniqueItems[items: HashSet[String]] := items";
    let mut parser = Parser::new(input.to_string());
    let expr = parser.parse_expression().unwrap();

    let mut codegen = RustCodeGenerator::new();
    let rust_code = codegen.generate(&expr).unwrap();

    assert!(rust_code.contains("fn unique_items(items: std::collections::HashSet<String>)"),
        "Function with HashSet parameter should generate HashSet, got: {}", rust_code);
}

#[test]
fn test_function_with_btreemap_parameter() {
    let input = "SortedMap[mapping: BTreeMap[Int32, String]] := mapping";
    let mut parser = Parser::new(input.to_string());
    let expr = parser.parse_expression().unwrap();

    let mut codegen = RustCodeGenerator::new();
    let rust_code = codegen.generate(&expr).unwrap();

    assert!(rust_code.contains("fn sorted_map(mapping: std::collections::BTreeMap<i32, String>)"),
        "Function with BTreeMap parameter should generate BTreeMap, got: {}", rust_code);
}

#[test]
fn test_function_with_btreeset_parameter() {
    let input = "OrderedSet[items: BTreeSet[Int64]] := items";
    let mut parser = Parser::new(input.to_string());
    let expr = parser.parse_expression().unwrap();

    let mut codegen = RustCodeGenerator::new();
    let rust_code = codegen.generate(&expr).unwrap();

    assert!(rust_code.contains("fn ordered_set(items: std::collections::BTreeSet<i64>)"),
        "Function with BTreeSet parameter should generate BTreeSet, got: {}", rust_code);
}

// ============================================
// Integration Tests - Nested Container Types
// ============================================

#[test]
fn test_function_with_nested_list() {
    let input = "Matrix[rows: List[List[Int32]]] := rows";
    let mut parser = Parser::new(input.to_string());
    let expr = parser.parse_expression().unwrap();

    let mut codegen = RustCodeGenerator::new();
    let rust_code = codegen.generate(&expr).unwrap();

    assert!(rust_code.contains("fn matrix(rows: Vec<Vec<i32>>)"),
        "Function with nested List should generate Vec<Vec<i32>>, got: {}", rust_code);
}

#[test]
fn test_function_with_map_of_lists() {
    let input = "Groups[data: Map[String, List[Int32]]] := data";
    let mut parser = Parser::new(input.to_string());
    let expr = parser.parse_expression().unwrap();

    let mut codegen = RustCodeGenerator::new();
    let rust_code = codegen.generate(&expr).unwrap();

    assert!(rust_code.contains("HashMap<String, Vec<i32>>"),
        "Function with Map of Lists should generate HashMap<String, Vec<i32>>, got: {}", rust_code);
}

#[test]
fn test_function_returning_list() {
    let input = "MakeList[x: Int32] := [x, x, x]";
    let mut parser = Parser::new(input.to_string());
    let expr = parser.parse_expression().unwrap();

    let mut codegen = RustCodeGenerator::new();
    let rust_code = codegen.generate(&expr).unwrap();

    assert!(rust_code.contains("vec![x, x, x]"),
        "Function returning list should generate vec!, got: {}", rust_code);
}
