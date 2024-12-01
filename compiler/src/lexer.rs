#[allow(dead_code)]
#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    Identifier(String),
    LeftBracket,
    RightBracket,
    LeftBrace,
    RightBrace,
    Comma,
    Colon,
    Define,  // `:=` for function definition
    Number(i64),
    Float(f64),
    String(String),
    Boolean(bool),
    Plus,
    Minus,
    Multiply,
    Divide,
    Power,
    Equals,
    NotEquals,
    LessThan,
    GreaterThan,
}

pub struct Lexer {
    input: Vec<char>,
    position: usize,
}

impl Lexer {
    pub fn new(input: String) -> Self {
        Lexer {
            input: input.chars().collect(),
            position: 0,
        }
    }

    pub fn next_token(&mut self) -> Option<Token> {
        self.skip_whitespace();
        
        if self.position >= self.input.len() {
            return None;
        }

        let token = match self.input[self.position] {
            '[' => {
                self.position += 1;
                Some(Token::LeftBracket)
            }
            ']' => {
                self.position += 1;
                Some(Token::RightBracket)
            }
            '{' => {
                self.position += 1;
                Some(Token::LeftBrace)
            }
            '}' => {
                self.position += 1;
                Some(Token::RightBrace)
            }
            ':' => {
                self.position += 1;
                Some(Token::Colon)
            }
            ',' => {
                self.position += 1;
                Some(Token::Comma)
            }
            '+' => {
                self.position += 1;
                Some(Token::Plus)
            }
            '-' => {
                self.position += 1;
                Some(Token::Minus)
            }
            '*' => {
                self.position += 1;
                Some(Token::Multiply)
            }
            '/' => {
                self.position += 1;
                Some(Token::Divide)
            }
            '^' => {
                self.position += 1;
                Some(Token::Power)
            }
            c if c.is_alphabetic() => {
                Some(Token::Identifier(self.read_identifier()))
            }
            c if c.is_digit(10) => {
                Some(Token::Number(self.read_number()))
            }
            _ => None,
        };

        token
    }

    fn skip_whitespace(&mut self) {
        while self.position < self.input.len() && 
              self.input[self.position].is_whitespace() {
            self.position += 1;
        }
    }

    fn read_identifier(&mut self) -> String {
        let mut identifier = String::new();
        while self.position < self.input.len() && 
              self.input[self.position].is_alphabetic() {
            identifier.push(self.input[self.position]);
            self.position += 1;
        }
        identifier
    }

    fn read_number(&mut self) -> i64 {
        let mut number = String::new();
        while self.position < self.input.len() && 
              self.input[self.position].is_digit(10) {
            number.push(self.input[self.position]);
            self.position += 1;
        }
        number.parse().unwrap_or(0)
    }
}
