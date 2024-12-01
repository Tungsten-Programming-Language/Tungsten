use w::lexer::{Lexer, Token};

#[test]
fn test_function_call() {
    let mut lexer = Lexer::new("Print[123, hello]".to_string());
    
    assert_eq!(lexer.next_token(), Some(Token::Identifier("Print".to_string())));
    assert_eq!(lexer.next_token(), Some(Token::LeftBracket));
    assert_eq!(lexer.next_token(), Some(Token::Number(123)));
    assert_eq!(lexer.next_token(), Some(Token::Comma));
    assert_eq!(lexer.next_token(), Some(Token::Identifier("hello".to_string())));
    assert_eq!(lexer.next_token(), Some(Token::RightBracket));
    assert_eq!(lexer.next_token(), None);
}

#[test]
fn test_nested_function_calls() {
    let mut lexer = Lexer::new("Add[Multiply[2, 3], 4]".to_string());
    
    assert_eq!(lexer.next_token(), Some(Token::Identifier("Add".to_string())));
    assert_eq!(lexer.next_token(), Some(Token::LeftBracket));
    assert_eq!(lexer.next_token(), Some(Token::Identifier("Multiply".to_string())));
    assert_eq!(lexer.next_token(), Some(Token::LeftBracket));
    assert_eq!(lexer.next_token(), Some(Token::Number(2)));
    assert_eq!(lexer.next_token(), Some(Token::Comma));
    assert_eq!(lexer.next_token(), Some(Token::Number(3)));
    assert_eq!(lexer.next_token(), Some(Token::RightBracket));
    assert_eq!(lexer.next_token(), Some(Token::Comma));
    assert_eq!(lexer.next_token(), Some(Token::Number(4)));
    assert_eq!(lexer.next_token(), Some(Token::RightBracket));
    assert_eq!(lexer.next_token(), None);
}
