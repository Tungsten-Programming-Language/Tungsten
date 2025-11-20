/// Represents the different types of tokens recognized by the lexer.
///
/// Each variant corresponds to a specific syntactic element in the language,
/// such as literals, operators, punctuation, and special keywords.
///
/// # Token Types
/// - Identifiers: Variable and function names
/// - Brackets and Braces: For function calls, lists, and maps
/// - Literals: Numbers, Strings, Booleans
/// - Operators: Arithmetic and comparison operations
/// - Log Levels: Debugging and logging keywords
///
/// # Variants
/// Some variants carry additional data (e.g., `Identifier`, `Number`) to preserve
/// the original lexeme's value during parsing.
#[allow(dead_code)]
#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    /// Represents a user-defined or language identifier
    Identifier(String),

    /// Left square bracket `[`
    LeftBracket,
    /// Right square bracket `]`
    RightBracket,
    /// Left curly brace `{`
    LeftBrace,
    /// Right curly brace `}`
    RightBrace,
    /// Left parenthesis `(`
    LeftParen,
    /// Right parenthesis `)`
    RightParen,

    /// Comma `,` used for separating elements
    Comma,
    /// Colon `:` used for type annotations
    Colon,
    /// Define token `:=` for function definitions
    Define,

    /// 32-bit integer literal (Rust's default)
    Number(i32),
    /// 64-bit floating-point literal (Rust's default)
    Float(f64),
    /// String literal
    String(String),
    /// Boolean literal (true/false)
    Boolean(bool),

    /// Addition operator `+`
    Plus,
    /// Subtraction operator `-`
    Minus,
    /// Multiplication operator `*`
    Multiply,
    /// Division operator `/`
    Divide,
    /// Exponentiation operator `^`
    Power,

    /// Equality comparison `==`
    Equals,
    /// Inequality comparison `!=`
    NotEquals,
    /// Less than comparison `<`
    LessThan,
    /// Greater than comparison `>`
    GreaterThan,

    /// Logging level tokens for different verbosity levels
    LogDebug,   // Debug log level
    LogInfo,    // Info log level
    LogWarn,    // Warning log level
    LogError,   // Error log level

    // Error handling keywords (Rust's safety model)
    /// None - represents empty Option value
    None,
    /// Some - represents present Option value
    Some,
    /// Ok - represents success Result value
    Ok,
    /// Err - represents error Result value
    Err,
}

/// Represents the lexical analyzer (tokenizer) for the language.
///
/// # Purpose
/// The Lexer breaks down the input source code into a sequence of tokens
/// that can be further processed by the parser.
///
/// # Components
/// - `input`: A vector of characters representing the entire input source
/// - `position`: Current position in the input stream during tokenization
///
/// # Tokenization Process
/// 1. Convert input string to a character vector
/// 2. Iterate through characters
/// 3. Recognize and generate appropriate tokens
/// 4. Skip whitespace and handle different token types
pub struct Lexer {
    /// The entire input source code as a vector of characters
    input: Vec<char>,
    /// Current reading position in the input stream
    position: usize,
}

impl Lexer {
    /// Creates a new Lexer instance from an input string.
    ///
    /// # Arguments
    /// * `input` - The source code to be tokenized
    ///
    /// # Returns
    /// A new Lexer with the input converted to a character vector
    ///
    /// # Details
    /// - Converts the input string to a vector of characters
    /// - Initializes the reading position to the start of the input
    pub fn new(input: String) -> Self {
        Lexer {
            input: input.chars().collect(),
            position: 0,
        }
    }

    /// Peek at the next token without consuming it
    ///
    /// # Returns
    /// - `Some(Token)` if a valid token is found
    /// - `None` if no more tokens are available
    pub fn peek_token(&self) -> Option<Token> {
        // Create a temporary clone to peek ahead
        let mut temp_lexer = Lexer {
            input: self.input.clone(),
            position: self.position,
        };
        temp_lexer.next_token()
    }

    /// Generates the next token from the input stream.
    ///
    /// # Returns
    /// - `Some(Token)` if a valid token is found
    /// - `None` if no more tokens are available
    ///
    /// # Token Recognition
    /// Recognizes various token types:
    /// - Punctuation (brackets, braces, commas)
    /// - Operators (arithmetic, comparison)
    /// - Literals (numbers, strings, booleans)
    /// - Keywords (log levels, boolean values)
    /// - Identifiers
    pub fn next_token(&mut self) -> Option<Token> {
        // Skip any leading whitespace
        self.skip_whitespace();
        
        // Check if we've reached the end of input
        if self.position >= self.input.len() {
            return None;
        }

        // Match and generate tokens based on current character
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
            '(' => {
                self.position += 1;
                Some(Token::LeftParen)
            }
            ')' => {
                self.position += 1;
                Some(Token::RightParen)
            }
            ':' => {
                self.position += 1;
                // Check for :=
                if self.position < self.input.len() && self.input[self.position] == '=' {
                    self.position += 1;
                    Some(Token::Define)
                } else {
                    Some(Token::Colon)
                }
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
            '=' => {
                self.position += 1;
                // Check for ==
                if self.position < self.input.len() && self.input[self.position] == '=' {
                    self.position += 1;
                    Some(Token::Equals)
                } else {
                    // Single = is not a token in this language
                    None
                }
            }
            '!' => {
                self.position += 1;
                // Check for !=
                if self.position < self.input.len() && self.input[self.position] == '=' {
                    self.position += 1;
                    Some(Token::NotEquals)
                } else {
                    // Single ! is not a token in this language
                    None
                }
            }
            '<' => {
                self.position += 1;
                Some(Token::LessThan)
            }
            '>' => {
                self.position += 1;
                Some(Token::GreaterThan)
            }
            '"' => {
                // Handle string literals
                Some(Token::String(self.read_string()))
            }
            c if c.is_alphabetic() => {
                // Handle keywords, identifiers, and boolean literals
                let identifier = self.read_identifier();
                match identifier.as_str() {
                    "LogDebug" => Some(Token::LogDebug),
                    "LogInfo" => Some(Token::LogInfo),
                    "LogWarn" => Some(Token::LogWarn),
                    "LogError" => Some(Token::LogError),
                    "true" => Some(Token::Boolean(true)),
                    "false" => Some(Token::Boolean(false)),
                    // Error handling keywords
                    "None" => Some(Token::None),
                    "Some" => Some(Token::Some),
                    "Ok" => Some(Token::Ok),
                    "Err" => Some(Token::Err),
                    _ => Some(Token::Identifier(identifier))
                }
            }
            c if c.is_digit(10) => {
                // Handle numeric literals
                Some(Token::Number(self.read_number()))
            }
            // Unrecognized character
            _ => None,
        };

        token
    }

    fn skip_whitespace(&mut self) {
        loop {
            // Skip whitespace
            while self.position < self.input.len() && self.input[self.position].is_whitespace() {
                self.position += 1;
            }

            // Check for ML-style comments (* ... *)
            if self.position + 1 < self.input.len()
                && self.input[self.position] == '('
                && self.input[self.position + 1] == '*' {
                // Skip the opening (*
                self.position += 2;

                // Find the closing *)
                while self.position + 1 < self.input.len() {
                    if self.input[self.position] == '*' && self.input[self.position + 1] == ')' {
                        // Skip the closing *)
                        self.position += 2;
                        break;
                    }
                    self.position += 1;
                }
            } else {
                // No more whitespace or comments to skip
                break;
            }
        }
    }

    fn read_identifier(&mut self) -> String {
        let mut identifier = String::new();
        while self.position < self.input.len() &&
              (self.input[self.position].is_alphabetic() ||
               self.input[self.position].is_digit(10) ||
               self.input[self.position] == '_') {
            identifier.push(self.input[self.position]);
            self.position += 1;
        }
        identifier
    }

    fn read_number(&mut self) -> i32 {
        let mut number = String::new();
        while self.position < self.input.len() &&
              self.input[self.position].is_digit(10) {
            number.push(self.input[self.position]);
            self.position += 1;
        }
        number.parse().unwrap_or(0)
    }

    fn read_string(&mut self) -> String {
        // Consume opening quote
        self.position += 1;
        let mut string = String::new();
        while self.position < self.input.len() && 
              self.input[self.position] != '"' {
            string.push(self.input[self.position]);
            self.position += 1;
        }
        // Consume closing quote
        if self.position < self.input.len() {
            self.position += 1;
        }
        string
    }
}
