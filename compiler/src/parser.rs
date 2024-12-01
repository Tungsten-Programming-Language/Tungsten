use crate::ast::{Expression, Operator, Type, TypeAnnotation};
use crate::lexer::{Lexer, Token};

pub struct Parser {
    lexer: Lexer,
    current_token: Option<Token>,
}

impl Parser {
    pub fn new(input: String) -> Self {
        let mut lexer = Lexer::new(input);
        let current_token = lexer.next_token();
        Parser {
            lexer,
            current_token,
        }
    }

    /// Parses the entire input and returns the resulting expression
    /// This method is used by tests and may appear unused in some contexts
    pub fn parse(&mut self) -> Option<Expression> {
        self.parse_expression()
    }

    pub fn parse_expression(&mut self) -> Option<Expression> {
        // Try parsing function definition first
        if let Some(func_def) = self.parse_function_definition() {
            return Some(func_def);
        }

        // Then try function call
        if let Some(func_call) = self.parse_function_call() {
            return Some(func_call);
        }

        // Then try binary operations
        self.parse_binary_operation()
    }

    fn parse_function_definition(&mut self) -> Option<Expression> {
        // Check for function definition syntax: f[x, y] := body
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

        // Parse parameters with optional type annotations
        let mut parameters = Vec::new();
        while self.current_token.is_some() {
            let param_name = match &self.current_token {
                Some(Token::Identifier(name)) => name.clone(),
                Some(Token::RightBracket) => break,
                _ => return None,
            };
            self.advance();

            // Check for type annotation
            let param_type = match self.current_token {
                Some(Token::Colon) => {
                    self.advance();
                    match self.parse_type() {
                        Some(t) => t,
                        None => Type::Int, // Default type if not specified
                    }
                },
                _ => Type::Int, // Default type if not specified
            };

            parameters.push(TypeAnnotation {
                name: param_name,
                type_: param_type,
            });

            // Handle comma or end of parameter list
            match self.current_token {
                Some(Token::Comma) => self.advance(),
                Some(Token::RightBracket) => break,
                _ => return None,
            }
        }
        
        // Consume right bracket
        match self.current_token {
            Some(Token::RightBracket) => self.advance(),
            _ => return None,
        }

        // Expect define token
        match self.current_token {
            Some(Token::Define) => self.advance(),
            _ => return None,
        }

        // Parse function body
        let body = match self.parse_expression() {
            Some(expr) => Box::new(expr),
            None => return None,
        };

        Some(Expression::FunctionDefinition {
            name,
            parameters,
            body,
        })
    }

    fn parse_function_call(&mut self) -> Option<Expression> {
        // Function call syntax: Function[arg1, arg2, ...]
        let function = match &self.current_token {
            Some(Token::Identifier(_)) | Some(Token::LeftBracket) => 
                Box::new(self.parse_expression()?),
            _ => return None,
        };

        // Expect left bracket for arguments
        match self.current_token {
            Some(Token::LeftBracket) => self.advance(),
            _ => return None,
        }

        // Parse arguments
        let mut arguments = Vec::new();
        while let Some(token) = &self.current_token {
            match token {
                Token::RightBracket => break,
                _ => {
                    let arg = self.parse_expression()?;
                    arguments.push(arg);

                    // Handle comma between arguments
                    match self.current_token {
                        Some(Token::Comma) => self.advance(),
                        Some(Token::RightBracket) => break,
                        _ => return None,
                    }
                }
            }
        }
        self.advance(); // Consume right bracket

        Some(Expression::FunctionCall {
            function,
            arguments,
        })
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
            Some(Token::Identifier(id)) => {
                let expr = Expression::Identifier(id.clone());
                self.advance();
                Some(expr)
            }
            Some(Token::LeftBracket) => self.parse_list(),
            Some(Token::LeftBrace) => self.parse_map(),
            _ => None,
        }
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

    fn parse_type(&mut self) -> Option<Type> {
        match &self.current_token {
            Some(Token::Identifier(id)) => {
                let type_ = match id.as_str() {
                    "int" => Type::Int,
                    "float" => Type::Float,
                    "string" => Type::String,
                    "bool" => Type::Bool,
                    _ => return None,
                };
                self.advance();
                Some(type_)
            }
            _ => None,
        }
    }

    fn advance(&mut self) {
        self.current_token = self.lexer.next_token();
    }
}
