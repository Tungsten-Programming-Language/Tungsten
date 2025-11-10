//! # W Language Parser
//! 
//! This parser is responsible for converting a stream of tokens into an Abstract Syntax Tree (AST).
//! It implements a recursive descent parsing strategy, supporting various language constructs 
//! such as function calls, binary operations, log statements, lists, maps, and more.
//! 
//! The parser works closely with the lexer to transform source code into a structured representation
//! that can be further processed by other compiler stages like type checking or code generation.

use crate::ast::{Expression, Operator, Type, TypeAnnotation, LogLevel};
use crate::lexer::{Lexer, Token};

/// Helper enum to distinguish between function arguments and parameters during parsing
enum ArgumentOrParameter {
    Expression(Expression),
    Parameter(TypeAnnotation),
}

/// Represents the parser state, holding a lexer and the current token being processed.
/// 
/// The parser maintains the context needed to parse a sequence of tokens into an Abstract Syntax Tree.
pub struct Parser {
    /// The lexer that provides a stream of tokens
    lexer: Lexer,
    /// The current token being examined during parsing
    current_token: Option<Token>,
}

impl Parser {
    /// Creates a new Parser instance from an input string.
    /// 
    /// # Arguments
    /// * `input` - The source code to be parsed
    /// 
    /// # Returns
    /// A new Parser with the first token loaded
    pub fn new(input: String) -> Self {
        let mut lexer = Lexer::new(input);
        let current_token = lexer.next_token();
        Parser {
            lexer,
            current_token,
        }
    }

    /// Parses the entire input and returns the resulting expression.
    /// 
    /// This method attempts to parse the full input, ensuring all tokens are consumed.
    /// 
    /// # Returns
    /// An optional Expression representing the parsed input, or None if parsing fails
    pub fn parse(&mut self) -> Option<Expression> {
        // Try parsing the entire input, handling multiple expressions if needed
        let expr = self.parse_expression();
        
        // Ensure no tokens are left unparsed
        if self.current_token.is_none() {
            expr
        } else {
            None // Parsing failed if tokens remain
        }
    }

    /// Attempts to parse a general expression, trying different expression types.
    /// 
    /// This method tries parsing expressions in a specific order:
    /// 1. Function definitions
    /// 2. Function calls
    /// 3. Binary operations
    /// 
    /// # Returns
    /// An optional Expression representing the parsed input, or None if parsing fails
    pub fn parse_expression(&mut self) -> Option<Expression> {
        // Check if this might be a function (call or definition)
        // by looking for Identifier followed by [
        if let Some(Token::Identifier(_)) = &self.current_token {
            // Peek ahead to check if next token is LeftBracket
            // We need to check this to avoid consuming tokens unnecessarily
            let is_function_syntax = self.lexer.peek_token()
                .map(|t| matches!(t, Token::LeftBracket))
                .unwrap_or(false);

            if is_function_syntax {
                // Try to parse as function call or definition
                if let Some(func_or_call) = self.parse_function_or_call() {
                    return Some(func_or_call);
                }
            }
        }

        // Try binary operations
        self.parse_binary_operation()
    }

    /// Parse either a function definition or function call
    fn parse_function_or_call(&mut self) -> Option<Expression> {
        // Get the identifier
        let name = match &self.current_token {
            Some(Token::Identifier(id)) => id.clone(),
            _ => return None,
        };
        self.advance();

        // Expect left bracket
        match self.current_token {
            Some(Token::LeftBracket) => self.advance(),
            _ => return None,
        }

        // Parse the contents of the brackets (could be parameters or arguments)
        // We'll determine whether this is a function definition or call
        // by checking for := after the closing bracket
        let mut items = Vec::new();
        loop {
            match &self.current_token {
                Some(Token::RightBracket) => {
                    self.advance();
                    break;
                }
                Some(Token::Comma) => {
                    self.advance();
                }
                _ => {
                    if let Some(item) = self.parse_argument_or_parameter() {
                        items.push(item);
                    } else {
                        return None;
                    }
                }
            }
        }

        // Now check if next token is :=
        match &self.current_token {
            Some(Token::Define) => {
                // It's a function definition
                self.advance();

                // Convert items to parameters
                let parameters: Vec<TypeAnnotation> = items.into_iter()
                    .filter_map(|item| {
                        if let ArgumentOrParameter::Parameter(p) = item {
                            Some(p)
                        } else {
                            None
                        }
                    })
                    .collect();

                // Parse body
                let body = Box::new(self.parse_expression()?);

                Some(Expression::FunctionDefinition {
                    name,
                    parameters,
                    body,
                })
            }
            _ => {
                // It's a function call
                let arguments: Vec<Expression> = items.into_iter()
                    .filter_map(|item| {
                        match item {
                            ArgumentOrParameter::Expression(e) => Some(e),
                            ArgumentOrParameter::Parameter(_) => None,
                        }
                    })
                    .collect();

                Some(Expression::FunctionCall {
                    function: Box::new(Expression::Identifier(name)),
                    arguments,
                })
            }
        }
    }

    fn parse_argument_or_parameter(&mut self) -> Option<ArgumentOrParameter> {
        // Try to parse as parameter (identifier with optional type)
        if let Some(Token::Identifier(name)) = &self.current_token {
            // Peek ahead to see if this is a type annotation
            let next_is_colon = self.lexer.peek_token()
                .map(|t| matches!(t, Token::Colon))
                .unwrap_or(false);

            if next_is_colon {
                // Parse as parameter with type annotation
                let param_name = name.clone();
                self.advance(); // consume identifier
                self.advance(); // consume colon

                if let Some(ty) = self.parse_type() {
                    return Some(ArgumentOrParameter::Parameter(TypeAnnotation {
                        name: param_name,
                        type_: ty,
                    }));
                }
            }
        }

        // Parse as general expression (handles identifiers, function calls, etc.)
        self.parse_expression().map(ArgumentOrParameter::Expression)
    }


    fn parse_binary_operation(&mut self) -> Option<Expression> {
        let mut left = self.parse_primary()?;

        while let Some(token) = &self.current_token {
            let operator = match token {
                Token::Plus => Operator::Add,
                Token::Minus => Operator::Subtract,
                Token::Multiply => Operator::Multiply,
                Token::Divide => Operator::Divide,
                Token::Power => Operator::Power,
                _ => break,
            };

            self.advance();
            let right = self.parse_primary()?;

            left = Expression::BinaryOp {
                left: Box::new(left),
                operator,
                right: Box::new(right),
            };
        }

        Some(left)
    }

    /// Parses a primary expression, which includes basic types, lists, maps, and log calls.
    /// 
    /// This method handles parsing of:
    /// - Numbers (integer and float)
    /// - Strings
    /// - Identifiers
    /// - Lists
    /// - Maps
    /// - Log calls (Debug, Info, Warn, Error)
    /// 
    /// # Returns
    /// - `Some(Expression)` if a valid primary expression is found
    /// - `None` if no valid primary expression can be parsed
    fn parse_primary(&mut self) -> Option<Expression> {
        match &self.current_token {
            Some(Token::Number(n)) => {
                let expr = Expression::Number(*n);
                self.advance();
                Some(expr)
            }
            Some(Token::Float(f)) => {
                let expr = Expression::Float(*f);
                self.advance();
                Some(expr)
            }
            Some(Token::String(s)) => {
                let expr = Expression::String(s.clone());
                self.advance();
                Some(expr)
            }
            Some(Token::Identifier(id)) if id == "Cond" => {
                self.advance();
                self.parse_cond_expression()
            }
            Some(Token::Identifier(id)) => {
                let expr = Expression::Identifier(id.clone());
                self.advance();
                Some(expr)
            }
            Some(Token::LeftBracket) => self.parse_list(),
            Some(Token::LeftBrace) => self.parse_map(),
            Some(Token::LogDebug) => {
                self.advance();
                self.parse_log_call(LogLevel::Debug)
            }
            Some(Token::LogInfo) => {
                self.advance();
                self.parse_log_call(LogLevel::Info)
            }
            Some(Token::LogWarn) => {
                self.advance();
                self.parse_log_call(LogLevel::Warn)
            }
            Some(Token::LogError) => {
                self.advance();
                self.parse_log_call(LogLevel::Error)
            }
            _ => None,
        }
    }

    /// Parses a Cond expression with the structure:
    /// Cond[[condition1 statements1] [condition2 statements2] ... [default_statements]]
    /// 
    /// # Returns
    /// - `Some(Expression::Cond)` if parsing succeeds
    /// - `None` if parsing fails
    fn parse_cond_expression(&mut self) -> Option<Expression> {
        // Expect left bracket for Cond
        match self.current_token {
            Some(Token::LeftBracket) => self.advance(),
            _ => return None,
        }

        let mut conditions = Vec::new();
        let mut default_statements = None;

        while let Some(token) = &self.current_token {
            match token {
                Token::RightBracket => break,
                Token::LeftBracket => {
                    self.advance(); // Consume left bracket of condition pair

                    // Parse condition
                    let condition = self.parse_expression()?;

                    // Parse statements for this condition
                    let statements = self.parse_expression()?;

                    // Consume right bracket of condition pair
                    match self.current_token {
                        Some(Token::RightBracket) => self.advance(),
                        _ => return None,
                    }

                    // If this is the last condition and no statements parsed yet, 
                    // treat it as default statements
                    if conditions.is_empty() && default_statements.is_none() {
                        default_statements = Some(Box::new(statements));
                    } else {
                        conditions.push((condition, statements));
                    }
                }
                _ => return None,
            }
        }

        // Consume right bracket of Cond
        match self.current_token {
            Some(Token::RightBracket) => self.advance(),
            _ => return None,
        }

        Some(Expression::Cond {
            conditions,
            default_statements,
        })
    }

    fn parse_log_call(&mut self, level: LogLevel) -> Option<Expression> {
        // Expect left bracket
        match self.current_token {
            Some(Token::LeftBracket) => self.advance(),
            _ => return None,
        }

        // Parse log message
        let message = match self.parse_expression() {
            Some(expr) => Box::new(expr),
            None => return None,
        };

        // Expect right bracket
        match self.current_token {
            Some(Token::RightBracket) => self.advance(),
            _ => return None,
        }

        Some(Expression::LogCall {
            level,
            message,
        })
    }

    fn parse_map(&mut self) -> Option<Expression> {
        // Consume left brace
        match self.current_token {
            Some(Token::LeftBrace) => self.advance(),
            _ => return None,
        }

        let mut map_entries = Vec::new();
        while let Some(token) = &self.current_token {
            match token {
                Token::RightBrace => break,
                _ => {
                    // Parse key
                    let key = self.parse_expression()?;

                    // Expect colon
                    match self.current_token {
                        Some(Token::Colon) => self.advance(),
                        _ => return None,
                    }

                    // Parse value
                    let value = self.parse_expression()?;
                    map_entries.push((key, value));

                    // Handle comma between entries
                    match self.current_token {
                        Some(Token::Comma) => self.advance(),
                        Some(Token::RightBrace) => break,
                        _ => return None,
                    }
                }
            }
        }
        self.advance(); // Consume right brace

        Some(Expression::Map(map_entries))
    }

    fn parse_list(&mut self) -> Option<Expression> {
        // Consume left bracket
        match self.current_token {
            Some(Token::LeftBracket) => self.advance(),
            _ => return None,
        }

        let mut elements = Vec::new();
        while let Some(token) = &self.current_token {
            match token {
                Token::RightBracket => break,
                _ => {
                    let elem = self.parse_expression()?;
                    elements.push(elem);

                    // Handle comma between elements
                    match self.current_token {
                        Some(Token::Comma) => self.advance(),
                        Some(Token::RightBracket) => break,
                        _ => return None,
                    }
                }
            }
        }
        self.advance(); // Consume right bracket

        Some(Expression::List(elements))
    }

    /// Parses a type annotation from the current token.
    ///
    /// Recognizes all Rust primitive types:
    /// - Signed integers: Int8, Int16, Int32, Int64, Int128, Int (isize)
    /// - Unsigned integers: UInt8, UInt16, UInt32, UInt64, UInt128, UInt (usize)
    /// - Floats: Float32, Float64
    /// - Other primitives: Bool, Char, String
    /// - Backward compatible: int (→ Int32), float (→ Float64)
    ///
    /// # Returns
    /// - `Some(Type)` if a valid type is found
    /// - `None` if the current token is not a recognized type identifier
    fn parse_type(&mut self) -> Option<Type> {
        match &self.current_token {
            Some(Token::Identifier(id)) => {
                let type_ = match id.as_str() {
                    // Signed integers
                    "Int8" => Type::Int8,
                    "Int16" => Type::Int16,
                    "Int32" => Type::Int32,
                    "Int64" => Type::Int64,
                    "Int128" => Type::Int128,
                    "Int" => Type::Int,

                    // Unsigned integers
                    "UInt8" => Type::UInt8,
                    "UInt16" => Type::UInt16,
                    "UInt32" => Type::UInt32,
                    "UInt64" => Type::UInt64,
                    "UInt128" => Type::UInt128,
                    "UInt" => Type::UInt,

                    // Floating point
                    "Float32" => Type::Float32,
                    "Float64" => Type::Float64,

                    // Other primitives
                    "Bool" => Type::Bool,
                    "Char" => Type::Char,
                    "String" => Type::String,

                    // Backward compatible (lowercase)
                    "int" => Type::Int32,      // Default to i32 like Rust
                    "float" => Type::Float64,  // Default to f64 like Rust
                    "string" => Type::String,
                    "bool" => Type::Bool,
                    "char" => Type::Char,

                    _ => return None,
                };
                self.advance();
                Some(type_)
            }
            _ => None,
        }
    }

    /// Advances the parser to the next token in the input stream.
    /// 
    /// This method updates the current_token by requesting the next token from the lexer.
    /// It is typically called after processing the current token to move parsing forward.
    fn advance(&mut self) {
        self.current_token = self.lexer.next_token();
    }
}
