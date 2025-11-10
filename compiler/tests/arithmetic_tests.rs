use w::lexer::{Lexer, Token};

fn evaluate(input: &str) -> i64 {
    let mut lexer = Lexer::new(input.to_string());
    let mut tokens = Vec::new();
    
    while let Some(token) = lexer.next_token() {
        tokens.push(token);
    }
    
    // Basic evaluation
    if tokens.len() >= 3 {
        match &tokens[0] {
            Token::Identifier(op) => {
                let mut numbers = Vec::new();
                for token in tokens.iter().skip(2) {
                    match token {
                        Token::Number(n) => numbers.push(*n),
                        Token::Comma => continue,
                        Token::RightBracket => break,
                        _ => panic!("Invalid token in arithmetic expression"),
                    }
                }
                
                match op.as_str() {
                    "Plus" => numbers.iter().map(|&n| n as i64).sum(),
                    "Minus" => {
                        if numbers.len() == 1 {
                            numbers[0] as i64
                        } else {
                            panic!("Minus operation requires exactly one argument")
                        }
                    },
                    "Power" => { // Add this block
                        if numbers.len() == 2 {
                            (numbers[0].pow(numbers[1] as u32)) as i64
                        } else {
                            panic!("Power operation requires exactly two arguments")
                        }
                    },
                    "Multiply" => numbers.iter().map(|&n| n as i64).product(),
                    "Divide" => {
                        if numbers.len() == 2 {
                            (numbers[0] / numbers[1]) as i64
                        } else {
                            panic!("Divide operation requires exactly two arguments")
                        }
                    },
                    _ => panic!("Unknown operation"),
                }
            },
            _ => panic!("Expected operation identifier"),
        }
    } else {
        panic!("Invalid expression");
    }
}

#[test]
fn test_power() {
    assert_eq!(evaluate("Power[2,3]"), 8);
}

#[test]
fn test_plus() {
    assert_eq!(evaluate("Plus[1,2,3]"), 6);
}

#[test]
fn test_minus() {
    assert_eq!(evaluate("Minus[42]"), 42);
}

#[test]
fn test_multiply() {
    assert_eq!(evaluate("Multiply[1,2,3]"), 6);
}

#[test]
fn test_divide() {
    assert_eq!(evaluate("Divide[6,3]"), 2);
}
