use w::parser::Parser;
use w::type_inference::{TypeInference, TypeError};
use w::ast::Type;

// ============================================================================
// Basic Type Inference Tests
// ============================================================================

#[test]
fn test_infer_number_literal() {
    let input = "42";
    let mut parser = Parser::new(input.to_string());
    let expr = parser.parse_expression().unwrap();

    let mut inference = TypeInference::new();
    let result = inference.infer_expression(&expr);

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Type::Int32);
}

#[test]
fn test_infer_string_literal() {
    let input = r#""Hello""#;
    let mut parser = Parser::new(input.to_string());
    let expr = parser.parse_expression().unwrap();

    let mut inference = TypeInference::new();
    let result = inference.infer_expression(&expr);

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Type::String);
}

#[test]
fn test_infer_boolean_literal() {
    let input = "true";
    let mut parser = Parser::new(input.to_string());
    let expr = parser.parse_expression().unwrap();

    let mut inference = TypeInference::new();
    let result = inference.infer_expression(&expr);

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Type::Bool);
}

#[test]
fn test_infer_tuple() {
    let input = "(1, \"hello\", true)";
    let mut parser = Parser::new(input.to_string());
    let expr = parser.parse_expression().unwrap();

    let mut inference = TypeInference::new();
    let result = inference.infer_expression(&expr);

    assert!(result.is_ok());
    assert_eq!(
        result.unwrap(),
        Type::Tuple(vec![Type::Int32, Type::String, Type::Bool])
    );
}

#[test]
fn test_infer_list_homogeneous() {
    let input = "[1, 2, 3]";
    let mut parser = Parser::new(input.to_string());
    let expr = parser.parse_expression().unwrap();

    let mut inference = TypeInference::new();
    let result = inference.infer_expression(&expr);

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Type::List(Box::new(Type::Int32)));
}

#[test]
fn test_infer_list_type_mismatch() {
    let input = r#"[1, "hello", 3]"#;
    let mut parser = Parser::new(input.to_string());
    let expr = parser.parse_expression().unwrap();

    let mut inference = TypeInference::new();
    let result = inference.infer_expression(&expr);

    assert!(result.is_err());
    match result.unwrap_err() {
        TypeError::TypeMismatch { expected, actual, .. } => {
            assert_eq!(expected, Type::Int32);
            assert_eq!(actual, Type::String);
        }
        _ => panic!("Expected TypeMismatch error"),
    }
}

// ============================================================================
// Binary Operation Type Inference
// ============================================================================

#[test]
fn test_infer_arithmetic_addition() {
    let input = "1 + 2";
    let mut parser = Parser::new(input.to_string());
    let expr = parser.parse_expression().unwrap();

    let mut inference = TypeInference::new();
    let result = inference.infer_expression(&expr);

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Type::Int32);
}

#[test]
fn test_infer_comparison_operation() {
    let input = "5 > 3";
    let mut parser = Parser::new(input.to_string());
    let expr = parser.parse_expression().unwrap();

    let mut inference = TypeInference::new();
    let result = inference.infer_expression(&expr);

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Type::Bool);
}

#[test]
fn test_infer_arithmetic_type_mismatch() {
    let input = r#"1 + "hello""#;
    let mut parser = Parser::new(input.to_string());
    let expr = parser.parse_expression().unwrap();

    let mut inference = TypeInference::new();
    let result = inference.infer_expression(&expr);

    assert!(result.is_err());
    match result.unwrap_err() {
        TypeError::TypeMismatch { .. } => {},
        _ => panic!("Expected TypeMismatch error"),
    }
}

// ============================================================================
// Function Type Inference
// ============================================================================

#[test]
fn test_infer_function_definition() {
    let input = "Square[x: Int32] := x * x";
    let mut parser = Parser::new(input.to_string());
    let expr = parser.parse_expression().unwrap();

    let mut inference = TypeInference::new();
    let result = inference.infer_expression(&expr);

    assert!(result.is_ok());
    match result.unwrap() {
        Type::Function(param_types, return_type) => {
            assert_eq!(param_types, vec![Type::Int32]);
            assert_eq!(*return_type, Type::Int32);
        }
        _ => panic!("Expected Function type"),
    }
}

#[test]
fn test_infer_function_call() {
    let input = r#"
Square[x: Int32] := x * x
Square[5]
"#;
    let mut parser = Parser::new(input.to_string());
    let program = parser.parse().unwrap();

    let mut inference = TypeInference::new();

    // First, type check the function definition
    if let w::ast::Expression::Program(expressions) = program {
        inference.infer_expression(&expressions[0]).unwrap();

        // Then check the function call
        let call_result = inference.infer_expression(&expressions[1]);
        assert!(call_result.is_ok());
        assert_eq!(call_result.unwrap(), Type::Int32);
    } else {
        panic!("Expected Program expression");
    }
}

#[test]
fn test_infer_function_arity_mismatch() {
    let input = r#"
Add[x: Int32, y: Int32] := x + y
Add[5]
"#;
    let mut parser = Parser::new(input.to_string());
    let program = parser.parse().unwrap();

    let mut inference = TypeInference::new();

    if let w::ast::Expression::Program(expressions) = program {
        inference.infer_expression(&expressions[0]).unwrap();

        let call_result = inference.infer_expression(&expressions[1]);
        assert!(call_result.is_err());
        match call_result.unwrap_err() {
            TypeError::ArityMismatch { expected, actual, .. } => {
                assert_eq!(expected, 2);
                assert_eq!(actual, 1);
            }
            _ => panic!("Expected ArityMismatch error"),
        }
    }
}

// ============================================================================
// Struct Type Inference
// ============================================================================

#[test]
fn test_infer_struct_definition() {
    let input = "Struct[Point, [x: Int32, y: Int32]]";
    let mut parser = Parser::new(input.to_string());
    let expr = parser.parse_expression().unwrap();

    let mut inference = TypeInference::new();
    let result = inference.infer_expression(&expr);

    assert!(result.is_ok());
    // Struct definition returns unit type
    assert_eq!(result.unwrap(), Type::Tuple(vec![]));
}

#[test]
fn test_infer_struct_instantiation() {
    let input = r#"
Struct[Point, [x: Int32, y: Int32]]
Point[10, 20]
"#;
    let mut parser = Parser::new(input.to_string());
    let program = parser.parse().unwrap();

    let mut inference = TypeInference::new();

    if let w::ast::Expression::Program(expressions) = program {
        // Type check struct definition
        inference.infer_expression(&expressions[0]).unwrap();

        // Type check struct instantiation
        let inst_result = inference.infer_expression(&expressions[1]);
        assert!(inst_result.is_ok());
        assert_eq!(inst_result.unwrap(), Type::Custom("Point".to_string()));
    } else {
        panic!("Expected Program expression");
    }
}

#[test]
fn test_infer_struct_field_type_mismatch() {
    let input = r#"
Struct[Point, [x: Int32, y: Int32]]
Point[10, "hello"]
"#;
    let mut parser = Parser::new(input.to_string());
    let program = parser.parse().unwrap();

    let mut inference = TypeInference::new();

    if let w::ast::Expression::Program(expressions) = program {
        inference.infer_expression(&expressions[0]).unwrap();

        let inst_result = inference.infer_expression(&expressions[1]);
        assert!(inst_result.is_err());
        match inst_result.unwrap_err() {
            TypeError::TypeMismatch { expected, actual, .. } => {
                assert_eq!(expected, Type::Int32);
                assert_eq!(actual, Type::String);
            }
            _ => panic!("Expected TypeMismatch error"),
        }
    }
}

#[test]
fn test_infer_struct_field_count_mismatch() {
    let input = r#"
Struct[Point, [x: Int32, y: Int32]]
Point[10]
"#;
    let mut parser = Parser::new(input.to_string());
    let program = parser.parse().unwrap();

    let mut inference = TypeInference::new();

    if let w::ast::Expression::Program(expressions) = program {
        inference.infer_expression(&expressions[0]).unwrap();

        let inst_result = inference.infer_expression(&expressions[1]);
        assert!(inst_result.is_err());
        match inst_result.unwrap_err() {
            TypeError::FieldCountMismatch { expected, actual, .. } => {
                assert_eq!(expected, 2);
                assert_eq!(actual, 1);
            }
            _ => panic!("Expected FieldCountMismatch error"),
        }
    }
}

// ============================================================================
// Option and Result Type Inference
// ============================================================================

#[test]
fn test_infer_some() {
    let input = "Some[42]";
    let mut parser = Parser::new(input.to_string());
    let expr = parser.parse_expression().unwrap();

    let mut inference = TypeInference::new();
    let result = inference.infer_expression(&expr);

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Type::Option(Box::new(Type::Int32)));
}

#[test]
fn test_infer_ok() {
    let input = r#"Ok["success"]"#;
    let mut parser = Parser::new(input.to_string());
    let expr = parser.parse_expression().unwrap();

    let mut inference = TypeInference::new();
    let result = inference.infer_expression(&expr);

    assert!(result.is_ok());
    assert_eq!(
        result.unwrap(),
        Type::Result(Box::new(Type::String), Box::new(Type::String))
    );
}

// ============================================================================
// Built-in Function Type Inference
// ============================================================================

#[test]
fn test_infer_print() {
    let input = r#"Print["Hello"]"#;
    let mut parser = Parser::new(input.to_string());
    let expr = parser.parse_expression().unwrap();

    let mut inference = TypeInference::new();
    let result = inference.infer_expression(&expr);

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Type::Tuple(vec![])); // Unit type
}

#[test]
fn test_infer_fold() {
    let input = "Fold[Function[{acc, x}, acc + x], 0, [1, 2, 3]]";
    let mut parser = Parser::new(input.to_string());
    let expr = parser.parse_expression().unwrap();

    let mut inference = TypeInference::new();
    let result = inference.infer_expression(&expr);

    assert!(result.is_ok());
    // Fold returns the type of the initial value (Int32 in this case)
    assert_eq!(result.unwrap(), Type::Int32);
}

// ============================================================================
// Match Expression Type Inference
// ============================================================================

#[test]
fn test_match_simple_value() {
    let input = r#"
Match[42,
  [1, "one"],
  [2, "two"],
  [_, "other"]
]
"#;
    let mut parser = Parser::new(input.to_string());
    let expr = parser.parse_expression().unwrap();

    let mut inference = TypeInference::new();
    let result = inference.infer_expression(&expr);

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Type::String);
}

#[test]
fn test_match_option_type() {
    let input = r#"
Match[Some[42],
  [Some[x], x],
  [None, 0]
]
"#;
    let mut parser = Parser::new(input.to_string());
    let expr = parser.parse_expression().unwrap();

    let mut inference = TypeInference::new();
    let result = inference.infer_expression(&expr);

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Type::Int32);
}

#[test]
fn test_match_tuple_pattern() {
    let input = r#"
Match[(1, "hello"),
  [(x, y), x]
]
"#;
    let mut parser = Parser::new(input.to_string());
    let expr = parser.parse_expression().unwrap();

    let mut inference = TypeInference::new();
    let result = inference.infer_expression(&expr);

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Type::Int32);
}

#[test]
fn test_match_list_pattern() {
    let input = r#"
Match[[1, 2, 3],
  [[x, y, z], x],
  [_, 0]
]
"#;
    let mut parser = Parser::new(input.to_string());
    let expr = parser.parse_expression().unwrap();

    let mut inference = TypeInference::new();
    let result = inference.infer_expression(&expr);

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Type::Int32);
}

#[test]
fn test_match_list_pattern_variable_binding() {
    let input = r#"
Match[[10, 20, 30],
  [[first, second, third], first + second]
]
"#;
    let mut parser = Parser::new(input.to_string());
    let expr = parser.parse_expression().unwrap();

    let mut inference = TypeInference::new();
    let result = inference.infer_expression(&expr);

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Type::Int32);
}

#[test]
fn test_match_list_pattern_type_error() {
    let input = r#"
Match[["a", "b", "c"],
  [[x, y, z], x]
]
"#;
    let mut parser = Parser::new(input.to_string());
    let expr = parser.parse_expression().unwrap();

    let mut inference = TypeInference::new();
    let result = inference.infer_expression(&expr);

    assert!(result.is_ok());
    // Should infer String type from the list
    assert_eq!(result.unwrap(), Type::String);
}

#[test]
fn test_match_nested_list_pattern() {
    let input = r#"
Match[[[1, 2], [3, 4]],
  [[first, second], first]
]
"#;
    let mut parser = Parser::new(input.to_string());
    let expr = parser.parse_expression().unwrap();

    let mut inference = TypeInference::new();
    let result = inference.infer_expression(&expr);

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Type::List(Box::new(Type::Int32)));
}

#[test]
fn test_match_arm_type_mismatch() {
    let input = r#"
Match[42,
  [1, "one"],
  [2, 42],
  [_, "other"]
]
"#;
    let mut parser = Parser::new(input.to_string());
    let expr = parser.parse_expression().unwrap();

    let mut inference = TypeInference::new();
    let result = inference.infer_expression(&expr);

    assert!(result.is_err());
    match result.unwrap_err() {
        w::type_inference::TypeError::TypeMismatch { expected, actual, context } => {
            assert_eq!(expected, Type::String);
            assert_eq!(actual, Type::Int32);
            assert!(context.contains("match arm"));
        }
        _ => panic!("Expected TypeMismatch error"),
    }
}

#[test]
fn test_match_pattern_type_mismatch() {
    let input = r#"
Match[[1, 2, 3],
  [["a", "b", "c"], "string list"]
]
"#;
    let mut parser = Parser::new(input.to_string());
    let expr = parser.parse_expression().unwrap();

    let mut inference = TypeInference::new();
    let result = inference.infer_expression(&expr);

    assert!(result.is_err());
    match result.unwrap_err() {
        w::type_inference::TypeError::TypeMismatch { .. } => {},
        _ => panic!("Expected TypeMismatch error"),
    }
}

#[test]
fn test_match_wildcard_pattern() {
    let input = r#"
Match[[1, 2, 3],
  [_, 42]
]
"#;
    let mut parser = Parser::new(input.to_string());
    let expr = parser.parse_expression().unwrap();

    let mut inference = TypeInference::new();
    let result = inference.infer_expression(&expr);

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Type::Int32);
}

