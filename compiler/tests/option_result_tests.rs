use w::lexer::Lexer;
use w::parser::Parser;
use w::ast::Expression;
use w::rust_codegen::RustCodeGenerator;

#[test]
fn test_none_parsing() {
    let mut parser = Parser::new("None".to_string());
    let expr = parser.parse_expression().unwrap();

    match expr {
        Expression::None => {
            // Success - None parsed correctly
        }
        _ => panic!("Expected None expression"),
    }
}

#[test]
fn test_some_parsing() {
    let mut parser = Parser::new("Some[42]".to_string());
    let expr = parser.parse_expression().unwrap();

    match expr {
        Expression::Some { value } => {
            match *value {
                Expression::Number(n) => assert_eq!(n, 42),
                _ => panic!("Expected number in Some"),
            }
        }
        _ => panic!("Expected Some expression"),
    }
}

#[test]
fn test_ok_parsing() {
    let mut parser = Parser::new("Ok[\"success\"]".to_string());
    let expr = parser.parse_expression().unwrap();

    match expr {
        Expression::Ok { value } => {
            match *value {
                Expression::String(ref s) => assert_eq!(s, "success"),
                _ => panic!("Expected string in Ok"),
            }
        }
        _ => panic!("Expected Ok expression"),
    }
}

#[test]
fn test_err_parsing() {
    let mut parser = Parser::new("Err[\"error message\"]".to_string());
    let expr = parser.parse_expression().unwrap();

    match expr {
        Expression::Err { error } => {
            match *error {
                Expression::String(ref s) => assert_eq!(s, "error message"),
                _ => panic!("Expected string in Err"),
            }
        }
        _ => panic!("Expected Err expression"),
    }
}

#[test]
fn test_none_codegen() {
    let mut parser = Parser::new("None".to_string());
    let expr = parser.parse_expression().unwrap();

    let mut codegen = RustCodeGenerator::new();
    let rust_code = codegen.generate(&expr).unwrap();

    assert!(rust_code.contains("None"), "Generated code should contain None");
}

#[test]
fn test_some_codegen() {
    let mut parser = Parser::new("Some[42]".to_string());
    let expr = parser.parse_expression().unwrap();

    let mut codegen = RustCodeGenerator::new();
    let rust_code = codegen.generate(&expr).unwrap();

    assert!(rust_code.contains("Some(42)"), "Generated code should contain Some(42), got: {}", rust_code);
}

#[test]
fn test_ok_codegen() {
    let mut parser = Parser::new("Ok[100]".to_string());
    let expr = parser.parse_expression().unwrap();

    let mut codegen = RustCodeGenerator::new();
    let rust_code = codegen.generate(&expr).unwrap();

    assert!(rust_code.contains("Ok(100)"), "Generated code should contain Ok(100), got: {}", rust_code);
}

#[test]
fn test_err_codegen() {
    let mut parser = Parser::new("Err[404]".to_string());
    let expr = parser.parse_expression().unwrap();

    let mut codegen = RustCodeGenerator::new();
    let rust_code = codegen.generate(&expr).unwrap();

    assert!(rust_code.contains("Err(404)"), "Generated code should contain Err(404), got: {}", rust_code);
}

#[test]
fn test_nested_some() {
    let mut parser = Parser::new("Some[Some[42]]".to_string());
    let expr = parser.parse_expression().unwrap();

    match expr {
        Expression::Some { value } => {
            match *value {
                Expression::Some { value: inner } => {
                    match *inner {
                        Expression::Number(n) => assert_eq!(n, 42),
                        _ => panic!("Expected nested number"),
                    }
                }
                _ => panic!("Expected nested Some"),
            }
        }
        _ => panic!("Expected Some expression"),
    }
}

#[test]
fn test_lexer_none_token() {
    let mut lexer = Lexer::new("None".to_string());
    let token = lexer.next_token().unwrap();
    assert_eq!(token, w::lexer::Token::None);
}

#[test]
fn test_lexer_some_token() {
    let mut lexer = Lexer::new("Some".to_string());
    let token = lexer.next_token().unwrap();
    assert_eq!(token, w::lexer::Token::Some);
}

#[test]
fn test_lexer_ok_token() {
    let mut lexer = Lexer::new("Ok".to_string());
    let token = lexer.next_token().unwrap();
    assert_eq!(token, w::lexer::Token::Ok);
}

#[test]
fn test_lexer_err_token() {
    let mut lexer = Lexer::new("Err".to_string());
    let token = lexer.next_token().unwrap();
    assert_eq!(token, w::lexer::Token::Err);
}
