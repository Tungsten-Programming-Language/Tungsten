//! # W Language Parser
//!
//! This parser is responsible for converting a stream of tokens into an Abstract Syntax Tree (AST).
//! It implements a recursive descent parsing strategy, supporting various language constructs
//! such as function calls, binary operations, log statements, lists, maps, and more.
//!
//! The parser works closely with the lexer to transform source code into a structured representation
//! that can be further processed by other compiler stages like type checking or code generation.

use crate::ast::{Expression, LogLevel, Operator, Pattern, Type, TypeAnnotation, UnaryOperator};
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
        let mut expressions = Vec::new();

        // Parse all expressions until we run out of tokens
        while self.current_token.is_some() {
            if let Some(expr) = self.parse_expression() {
                expressions.push(expr);
            } else {
                return None; // Parsing failed
            }
        }

        // If we have multiple expressions, wrap them in a Program node
        if expressions.is_empty() {
            None
        } else if expressions.len() == 1 {
            Some(expressions.into_iter().next().unwrap())
        } else {
            Some(Expression::Program(expressions))
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
        let mut expr = self.parse_base_expression()?;

        // Check for pipe operator |> (lowest precedence, left-associative)
        while matches!(&self.current_token, Some(Token::Pipe)) {
            self.advance();

            // Parse the RHS — must be a function call or bare identifier
            let rhs = self.parse_base_expression()?;

            // Unwrap Propagate on RHS for pipe desugaring, then re-wrap
            let (inner_rhs, propagate) = match rhs {
                Expression::Propagate { expr: inner } => (*inner, true),
                other => (other, false),
            };

            // Desugar: insert LHS as last argument of RHS function call
            expr = match inner_rhs {
                Expression::FunctionCall {
                    function,
                    mut arguments,
                } => {
                    arguments.push(expr);
                    Expression::FunctionCall {
                        function,
                        arguments,
                    }
                }
                Expression::Identifier(name) => Expression::FunctionCall {
                    function: Box::new(Expression::Identifier(name)),
                    arguments: vec![expr],
                },
                _ => return None, // Pipe RHS must be a function call or identifier
            };

            // Re-wrap in Propagate if RHS had ? operator
            if propagate {
                expr = Expression::Propagate {
                    expr: Box::new(expr),
                };
            }
        }

        Some(expr)
    }

    /// Parses a single expression without pipe operator handling.
    /// Pipe handling is in `parse_expression` which wraps this method.
    fn parse_base_expression(&mut self) -> Option<Expression> {
        // Check if this might be a function (call or definition)
        // by looking for Identifier followed by [
        if let Some(Token::Identifier(id)) = &self.current_token {
            // Special handling for With - local bindings expression
            if id == "With" {
                self.advance();
                return self.parse_with_expression();
            }

            // Special handling for Module - local mutable bindings expression
            if id == "Module" {
                self.advance();
                return self.parse_module_expression();
            }

            // Special handling for Cond - don't treat it as a regular function call
            if id == "Cond" {
                self.advance();
                return self.parse_cond_expression();
            }

            // Special handling for Match - pattern matching expression
            if id == "Match" {
                self.advance();
                return self.parse_match_expression();
            }

            // Special handling for Function - lambda/closure expression
            if id == "Function" {
                self.advance();
                return self.parse_lambda_expression();
            }

            // Special handling for Struct - struct definition
            if id == "Struct" {
                self.advance();
                return self.parse_struct_definition();
            }

            // Special handling for Do - loop construct
            if id == "Do" {
                self.advance();
                return self.parse_do_expression();
            }

            // Special handling for While - while loop construct
            if id == "While" {
                self.advance();
                return self.parse_while_expression();
            }

            // Special handling for Break - loop exit
            if id == "Break" {
                self.advance();
                return self.parse_break_expression();
            }

            // Special handling for Continue - loop continue
            if id == "Continue" {
                self.advance();
                return self.parse_continue_expression();
            }

            // Peek ahead to check if next token is LeftBracket
            // We need to check this to avoid consuming tokens unnecessarily
            let is_function_syntax = self
                .lexer
                .peek_token()
                .map(|t| matches!(t, Token::LeftBracket))
                .unwrap_or(false);

            if is_function_syntax {
                // Try to parse as function call or definition
                let func_or_call = self.parse_function_or_call()?;

                // Handle postfix ? on function call result first (highest precedence)
                let mut expr = func_or_call;
                while matches!(&self.current_token, Some(Token::Question)) {
                    self.advance();
                    expr = Expression::Propagate {
                        expr: Box::new(expr),
                    };
                }

                // Now parse binary operators with proper precedence
                // Start from lowest precedence (logical) and work up
                let expr = self.parse_power_operators_from(expr)?;
                let expr = self.parse_multiplicative_continuation(expr)?;
                let expr = self.parse_additive_continuation(expr)?;
                let expr = self.parse_comparison_continuation(expr)?;
                let expr = self.parse_logical_continuation(expr)?;

                return Some(expr);
            }
        }

        // Try binary operations
        let expr = self.parse_binary_operation()?;

        // Check for arrow lambda shorthand: x -> body
        // Desugars: x -> body  →  Function[{x}, body]
        if let Expression::Identifier(param_name) = &expr {
            if matches!(&self.current_token, Some(Token::Arrow)) {
                self.advance(); // consume ->
                let body = Box::new(self.parse_base_expression()?);
                return Some(Expression::Lambda {
                    parameters: vec![TypeAnnotation {
                        name: param_name.clone(),
                        type_: Type::Int32, // Placeholder - will be inferred
                    }],
                    body,
                });
            }
        }

        Some(expr)
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
                let parameters: Vec<TypeAnnotation> = items
                    .into_iter()
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
                let arguments: Vec<Expression> = items
                    .into_iter()
                    .filter_map(|item| match item {
                        ArgumentOrParameter::Expression(e) => Some(e),
                        ArgumentOrParameter::Parameter(_) => None,
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
            let next_is_colon = self
                .lexer
                .peek_token()
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
        self.parse_logical()
    }

    fn parse_logical(&mut self) -> Option<Expression> {
        let left = self.parse_comparison()?;
        self.parse_logical_continuation(left)
    }

    fn parse_logical_continuation(&mut self, mut left: Expression) -> Option<Expression> {
        while let Some(token) = &self.current_token {
            let operator = match token {
                Token::And => Operator::And,
                Token::Or => Operator::Or,
                _ => break,
            };

            self.advance();
            let right = self.parse_comparison()?;

            left = Expression::BinaryOp {
                left: Box::new(left),
                operator,
                right: Box::new(right),
            };
        }

        Some(left)
    }

    fn parse_comparison(&mut self) -> Option<Expression> {
        let left = self.parse_additive()?;
        self.parse_comparison_continuation(left)
    }

    fn parse_comparison_continuation(&mut self, mut left: Expression) -> Option<Expression> {
        while let Some(token) = &self.current_token {
            let operator = match token {
                Token::Equals => Operator::Equals,
                Token::NotEquals => Operator::NotEquals,
                Token::LessThan => Operator::LessThan,
                Token::GreaterThan => Operator::GreaterThan,
                Token::LessEqual => Operator::LessEqual,
                Token::GreaterEqual => Operator::GreaterEqual,
                _ => break,
            };

            self.advance();
            let right = self.parse_additive()?;

            left = Expression::BinaryOp {
                left: Box::new(left),
                operator,
                right: Box::new(right),
            };
        }

        Some(left)
    }

    fn parse_additive(&mut self) -> Option<Expression> {
        let left = self.parse_multiplicative()?;
        self.parse_additive_continuation(left)
    }

    fn parse_additive_continuation(&mut self, mut left: Expression) -> Option<Expression> {
        while let Some(token) = &self.current_token {
            let operator = match token {
                Token::Plus => Operator::Add,
                Token::Minus => Operator::Subtract,
                _ => break,
            };

            self.advance();
            let right = self.parse_multiplicative()?;

            left = Expression::BinaryOp {
                left: Box::new(left),
                operator,
                right: Box::new(right),
            };
        }

        Some(left)
    }

    fn parse_multiplicative(&mut self) -> Option<Expression> {
        let left = self.parse_power()?;
        self.parse_multiplicative_continuation(left)
    }

    fn parse_multiplicative_continuation(&mut self, mut left: Expression) -> Option<Expression> {
        while let Some(token) = &self.current_token {
            let operator = match token {
                Token::Multiply => Operator::Multiply,
                Token::Divide => Operator::Divide,
                _ => break,
            };

            self.advance();
            let right = self.parse_power()?;

            left = Expression::BinaryOp {
                left: Box::new(left),
                operator,
                right: Box::new(right),
            };
        }

        Some(left)
    }

    fn parse_power(&mut self) -> Option<Expression> {
        if matches!(&self.current_token, Some(Token::Minus)) {
            self.advance();
            let operand = self.parse_power()?;
            return Some(Expression::BinaryOp {
                left: Box::new(Expression::Number(0)),
                operator: Operator::Subtract,
                right: Box::new(operand),
            });
        }

        if matches!(&self.current_token, Some(Token::Not)) {
            self.advance();
            let operand = self.parse_power()?;
            return Some(Expression::UnaryOp {
                operator: UnaryOperator::Not,
                operand: Box::new(operand),
            });
        }

        let left = self.parse_primary()?;
        self.parse_power_operators_from(left)
    }

    fn parse_power_operators_from(&mut self, mut left: Expression) -> Option<Expression> {
        while matches!(&self.current_token, Some(Token::Question)) {
            self.advance();
            left = Expression::Propagate {
                expr: Box::new(left),
            };
        }

        if matches!(&self.current_token, Some(Token::Power)) {
            self.advance();
            let right = self.parse_power()?;
            left = Expression::BinaryOp {
                left: Box::new(left),
                operator: Operator::Power,
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
            Some(Token::Boolean(b)) => {
                let expr = Expression::Boolean(*b);
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

                // Check if this is a function call (identifier followed by [)
                if matches!(&self.current_token, Some(Token::LeftBracket)) {
                    self.advance();

                    // Parse arguments
                    let mut arguments = Vec::new();
                    while !matches!(&self.current_token, Some(Token::RightBracket)) {
                        if matches!(&self.current_token, Some(Token::Comma)) {
                            self.advance();
                            continue;
                        }
                        if let Some(arg) = self.parse_expression() {
                            arguments.push(arg);
                        } else {
                            return None;
                        }
                    }

                    // Consume closing bracket
                    match &self.current_token {
                        Some(Token::RightBracket) => self.advance(),
                        _ => return None,
                    }

                    Some(Expression::FunctionCall {
                        function: Box::new(expr),
                        arguments,
                    })
                } else {
                    Some(expr)
                }
            }
            Some(Token::LeftParen) => self.parse_tuple(),
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
            // Error handling expressions
            Some(Token::None) => {
                self.advance();
                Some(Expression::None)
            }
            Some(Token::Some) => {
                self.advance();
                self.parse_some_expression()
            }
            Some(Token::Ok) => {
                self.advance();
                self.parse_ok_expression()
            }
            Some(Token::Err) => {
                self.advance();
                self.parse_err_expression()
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

                    // Parse first expression
                    let first_expr = self.parse_expression()?;

                    // Try to parse second expression (if it exists, this is a condition-statement pair)
                    // If there's a RightBracket next, this is a default statement
                    let is_default = matches!(self.current_token, Some(Token::RightBracket));

                    if is_default {
                        // This bracket contains only one expression - it's the default
                        self.advance(); // Consume right bracket
                        default_statements = Some(Box::new(first_expr));
                    } else {
                        // Parse the second expression (statements for this condition)
                        let statements = self.parse_expression()?;

                        // Consume right bracket of condition pair
                        match self.current_token {
                            Some(Token::RightBracket) => self.advance(),
                            _ => return None,
                        }

                        conditions.push((first_expr, statements));
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

    /// Parses a Match expression with the structure:
    /// Match[value, [pattern1, result1], [pattern2, result2], ...]
    ///
    /// # Returns
    /// - `Some(Expression::Match)` if parsing succeeds
    /// - `None` if parsing fails
    fn parse_match_expression(&mut self) -> Option<Expression> {
        // Expect left bracket for Match
        match self.current_token {
            Some(Token::LeftBracket) => self.advance(),
            _ => return None,
        }

        // Parse the value to match against
        let value = Box::new(self.parse_expression()?);

        // Expect comma after value
        match self.current_token {
            Some(Token::Comma) => self.advance(),
            _ => return None,
        }

        let mut arms = Vec::new();

        // Parse match arms
        while let Some(token) = &self.current_token {
            match token {
                Token::RightBracket => break,
                Token::LeftBracket => {
                    self.advance(); // Consume left bracket of arm

                    // Parse the pattern
                    let pattern = self.parse_pattern()?;

                    // Expect comma between pattern and result
                    match self.current_token {
                        Some(Token::Comma) => self.advance(),
                        _ => return None,
                    }

                    // Parse the result expression
                    let result = self.parse_expression()?;

                    // Consume right bracket of arm
                    match self.current_token {
                        Some(Token::RightBracket) => self.advance(),
                        _ => return None,
                    }

                    arms.push((pattern, result));

                    // Handle optional comma between arms
                    if matches!(self.current_token, Some(Token::Comma)) {
                        self.advance();
                    }
                }
                _ => return None,
            }
        }

        // Consume right bracket of Match
        match self.current_token {
            Some(Token::RightBracket) => self.advance(),
            _ => return None,
        }

        Some(Expression::Match { value, arms })
    }

    /// Parses a Lambda/Closure expression with the structure:
    /// Function[{param1, param2, ...}, body]
    /// or Function[{param1: Type1, param2: Type2}, body]
    ///
    /// # Returns
    /// - `Some(Expression::Lambda)` if parsing succeeds
    /// - `None` if parsing fails
    fn parse_lambda_expression(&mut self) -> Option<Expression> {
        // Expect left bracket for Function
        match self.current_token {
            Some(Token::LeftBracket) => self.advance(),
            _ => return None,
        }

        // Expect left brace for parameter list
        match self.current_token {
            Some(Token::LeftBrace) => self.advance(),
            _ => return None,
        }

        let mut parameters = Vec::new();

        // Parse parameters
        while let Some(token) = &self.current_token {
            match token {
                Token::RightBrace => break,
                Token::Identifier(name) => {
                    let param_name = name.clone();
                    self.advance();

                    // Check for type annotation
                    if matches!(self.current_token, Some(Token::Colon)) {
                        self.advance(); // Consume ':'

                        let param_type = self.parse_type()?;
                        parameters.push(TypeAnnotation {
                            name: param_name,
                            type_: param_type,
                        });
                    } else {
                        // No type annotation - will be inferred
                        // For now, use a placeholder type
                        parameters.push(TypeAnnotation {
                            name: param_name,
                            type_: Type::Int32, // Placeholder - should be inferred
                        });
                    }

                    // Handle comma between parameters
                    if matches!(self.current_token, Some(Token::Comma)) {
                        self.advance();
                    }
                }
                _ => return None,
            }
        }

        // Consume right brace
        match self.current_token {
            Some(Token::RightBrace) => self.advance(),
            _ => return None,
        }

        // Expect comma after parameter list
        match self.current_token {
            Some(Token::Comma) => self.advance(),
            _ => return None,
        }

        // Parse body expression
        let body = Box::new(self.parse_expression()?);

        // Consume right bracket of Function
        match self.current_token {
            Some(Token::RightBracket) => self.advance(),
            _ => return None,
        }

        Some(Expression::Lambda { parameters, body })
    }

    /// Parses a Struct definition with the structure:
    /// Struct[Name, [field1: Type1, field2: Type2, ...]]
    ///
    /// # Returns
    /// - `Some(Expression::StructDefinition)` if parsing succeeds
    /// - `None` if parsing fails
    fn parse_struct_definition(&mut self) -> Option<Expression> {
        // Expect left bracket for Struct
        match self.current_token {
            Some(Token::LeftBracket) => self.advance(),
            _ => return None,
        }

        // Parse struct name
        let struct_name = match &self.current_token {
            Some(Token::Identifier(name)) => name.clone(),
            _ => return None,
        };
        self.advance();

        // Expect comma after name
        match self.current_token {
            Some(Token::Comma) => self.advance(),
            _ => return None,
        }

        // Expect left bracket for field list
        match self.current_token {
            Some(Token::LeftBracket) => self.advance(),
            _ => return None,
        }

        let mut fields = Vec::new();

        // Parse fields
        while let Some(token) = &self.current_token {
            match token {
                Token::RightBracket => break,
                Token::Identifier(field_name) => {
                    let name = field_name.clone();
                    self.advance();

                    // Expect colon for type annotation
                    match self.current_token {
                        Some(Token::Colon) => self.advance(),
                        _ => return None,
                    }

                    // Parse field type
                    let field_type = self.parse_type()?;
                    fields.push(TypeAnnotation {
                        name,
                        type_: field_type,
                    });

                    // Handle comma between fields
                    if matches!(self.current_token, Some(Token::Comma)) {
                        self.advance();
                    }
                }
                _ => return None,
            }
        }

        // Consume right bracket of field list
        match self.current_token {
            Some(Token::RightBracket) => self.advance(),
            _ => return None,
        }

        // Consume right bracket of Struct
        match self.current_token {
            Some(Token::RightBracket) => self.advance(),
            _ => return None,
        }

        Some(Expression::StructDefinition {
            name: struct_name,
            fields,
        })
    }

    /// Parses a Do loop expression with the structure:
    /// Do[body, n]                      - iterate n times (no variable)
    /// Do[body, {n}]                    - iterate n times (no variable)
    /// Do[body, {i, imax}]              - iterate i from 1 to imax
    /// Do[body, {i, imin, imax}]        - iterate i from imin to imax
    /// Do[body, {i, imin, imax, di}]    - iterate with step di
    ///
    /// # Returns
    /// - `Some(Expression::Do)` if parsing succeeds
    /// - `None` if parsing fails
    fn parse_do_expression(&mut self) -> Option<Expression> {
        // Expect left bracket for Do
        match self.current_token {
            Some(Token::LeftBracket) => self.advance(),
            _ => return None,
        }

        // Parse body expression
        let body = Box::new(self.parse_expression()?);

        // Expect comma after body
        match self.current_token {
            Some(Token::Comma) => self.advance(),
            _ => return None,
        }

        // Check if next is a number (simple Do[body, n] form) or left brace
        match &self.current_token {
            Some(Token::Number(n)) => {
                // Do[body, n] - simple iteration without variable
                let end = Box::new(Expression::Number(*n));
                self.advance();

                // Consume right bracket of Do
                match self.current_token {
                    Some(Token::RightBracket) => self.advance(),
                    _ => return None,
                }

                return Some(Expression::Do {
                    body,
                    var: None,
                    start: None,
                    end,
                    step: None,
                });
            }
            Some(Token::LeftBrace) => {
                // Do[body, {iterator_spec}]
                self.advance();
            }
            _ => return None,
        }

        // Parse the iterator specification
        // Could be: {n}, {i, imax}, {i, imin, imax}, {i, imin, imax, di}
        let first = self.parse_expression()?;

        let (var, start, end, step) = match self.current_token {
            Some(Token::RightBrace) => {
                // Just {n} - no loop variable
                self.advance();
                (None, None, Box::new(first), None)
            }
            Some(Token::Comma) => {
                self.advance();
                // First is the loop variable name
                let var_name = match first {
                    Expression::Identifier(name) => name,
                    _ => return None,
                };

                // Parse imin or imax
                let second = self.parse_expression()?;

                match self.current_token {
                    Some(Token::RightBrace) => {
                        // {i, imax} - start from 1
                        self.advance();
                        (Some(var_name), None, Box::new(second), None)
                    }
                    Some(Token::Comma) => {
                        self.advance();
                        // Parse imax
                        let third = self.parse_expression()?;

                        match self.current_token {
                            Some(Token::RightBrace) => {
                                // {i, imin, imax}
                                self.advance();
                                (
                                    Some(var_name),
                                    Some(Box::new(second)),
                                    Box::new(third),
                                    None,
                                )
                            }
                            Some(Token::Comma) => {
                                self.advance();
                                // Parse step
                                let fourth = self.parse_expression()?;

                                // Expect right brace
                                match self.current_token {
                                    Some(Token::RightBrace) => self.advance(),
                                    _ => return None,
                                };

                                (
                                    Some(var_name),
                                    Some(Box::new(second)),
                                    Box::new(third),
                                    Some(Box::new(fourth)),
                                )
                            }
                            _ => return None,
                        }
                    }
                    _ => return None,
                }
            }
            _ => return None,
        };

        // Consume right bracket of Do
        match self.current_token {
            Some(Token::RightBracket) => self.advance(),
            _ => return None,
        }

        Some(Expression::Do {
            body,
            var,
            start,
            end,
            step,
        })
    }

    /// Parses a While loop expression with the structure:
    /// While[condition, body]
    ///
    /// # Returns
    /// - `Some(Expression::While)` if parsing succeeds
    /// - `None` if parsing fails
    fn parse_while_expression(&mut self) -> Option<Expression> {
        // Expect left bracket for While
        match self.current_token {
            Some(Token::LeftBracket) => self.advance(),
            _ => return None,
        }

        // Parse condition expression
        let condition = Box::new(self.parse_expression()?);

        // Expect comma after condition
        match self.current_token {
            Some(Token::Comma) => self.advance(),
            _ => return None,
        }

        // Parse body expression
        let body = Box::new(self.parse_expression()?);

        // Consume right bracket of While
        match self.current_token {
            Some(Token::RightBracket) => self.advance(),
            _ => return None,
        }

        Some(Expression::While { condition, body })
    }

    /// Parses a Break expression: Break[]
    ///
    /// # Returns
    /// - `Some(Expression::Break)` if parsing succeeds
    /// - `None` if parsing fails
    fn parse_break_expression(&mut self) -> Option<Expression> {
        // Expect left bracket
        match self.current_token {
            Some(Token::LeftBracket) => self.advance(),
            _ => return None,
        }

        // Expect right bracket
        match self.current_token {
            Some(Token::RightBracket) => self.advance(),
            _ => return None,
        }

        Some(Expression::Break)
    }

    /// Parses a Continue expression: Continue[]
    ///
    /// # Returns
    /// - `Some(Expression::Continue)` if parsing succeeds
    /// - `None` if parsing fails
    fn parse_continue_expression(&mut self) -> Option<Expression> {
        // Expect left bracket
        match self.current_token {
            Some(Token::LeftBracket) => self.advance(),
            _ => return None,
        }

        // Expect right bracket
        match self.current_token {
            Some(Token::RightBracket) => self.advance(),
            _ => return None,
        }

        Some(Expression::Continue)
    }

    /// Parses a With expression with the structure:
    /// With[{binding1 = expr1, binding2 = expr2, ...}, body]
    ///
    /// # Returns
    /// - `Some(Expression::With)` if parsing succeeds
    /// - `None` if parsing fails
    fn parse_with_expression(&mut self) -> Option<Expression> {
        // Expect left bracket for With
        match self.current_token {
            Some(Token::LeftBracket) => self.advance(),
            _ => return None,
        }

        // Expect left brace for bindings
        match self.current_token {
            Some(Token::LeftBrace) => self.advance(),
            _ => return None,
        }

        let mut bindings = Vec::new();

        // Parse bindings: name = expr, name = expr, ...
        while let Some(token) = &self.current_token {
            match token {
                Token::RightBrace => break,
                Token::Identifier(name) => {
                    let binding_name = name.clone();
                    self.advance();

                    // Expect '=' for binding
                    match self.current_token {
                        Some(Token::Assign) => self.advance(),
                        _ => return None,
                    }

                    // Parse binding expression
                    let binding_expr = self.parse_expression()?;
                    bindings.push((binding_name, binding_expr));

                    // Handle comma between bindings
                    if matches!(self.current_token, Some(Token::Comma)) {
                        self.advance();
                    } else if !matches!(self.current_token, Some(Token::RightBrace)) {
                        return None;
                    }
                }
                _ => return None,
            }
        }

        // Consume right brace
        match self.current_token {
            Some(Token::RightBrace) => self.advance(),
            _ => return None,
        }

        // Expect comma after bindings
        match self.current_token {
            Some(Token::Comma) => self.advance(),
            _ => return None,
        }

        // Parse body expression
        let body = Box::new(self.parse_expression()?);

        // Consume right bracket of With
        match self.current_token {
            Some(Token::RightBracket) => self.advance(),
            _ => return None,
        }

        Some(Expression::With { bindings, body })
    }

    /// Parses a Module expression with the structure:
    /// Module[{x, y, ...}, body] or Module[{x = expr1, y = expr2, ...}, body]
    ///
    /// Variables can be uninitialized (just name) or initialized (name = expr)
    ///
    /// # Returns
    /// - `Some(Expression::Module)` if parsing succeeds
    /// - `None` if parsing fails
    fn parse_module_expression(&mut self) -> Option<Expression> {
        // Expect left bracket for Module
        match self.current_token {
            Some(Token::LeftBracket) => self.advance(),
            _ => return None,
        }

        // Expect left brace for bindings
        match self.current_token {
            Some(Token::LeftBrace) => self.advance(),
            _ => return None,
        }

        let mut bindings = Vec::new();

        // Parse bindings: name, name = expr, ...
        while let Some(token) = &self.current_token {
            match token {
                Token::RightBrace => break,
                Token::Identifier(name) => {
                    let binding_name = name.clone();
                    self.advance();

                    // Check if there's an '=' for initialization
                    match &self.current_token {
                        Some(Token::Assign) => {
                            self.advance();
                            let init_expr = self.parse_expression()?;
                            bindings.push((binding_name, Some(init_expr)));
                        }
                        _ => {
                            // Uninitialized variable
                            bindings.push((binding_name, None));
                        }
                    }

                    // Handle comma between bindings
                    if matches!(self.current_token, Some(Token::Comma)) {
                        self.advance();
                    } else if !matches!(self.current_token, Some(Token::RightBrace)) {
                        return None;
                    }
                }
                _ => return None,
            }
        }

        // Consume right brace
        match self.current_token {
            Some(Token::RightBrace) => self.advance(),
            _ => return None,
        }

        // Expect comma after bindings
        match self.current_token {
            Some(Token::Comma) => self.advance(),
            _ => return None,
        }

        // Parse body expression
        let body = Box::new(self.parse_expression()?);

        // Consume right bracket of Module
        match self.current_token {
            Some(Token::RightBracket) => self.advance(),
            _ => return None,
        }

        Some(Expression::Module { bindings, body })
    }

    /// Parses a pattern for use in Match expressions
    ///
    /// # Pattern Types
    /// - `_` - Wildcard pattern
    /// - Literals: `42`, `"hello"`, `true`, `false`
    /// - Variables: `x`, `value`
    /// - Constructors: `Some[x]`, `Ok[val]`, `None`, `Err[e]`
    /// - Tuples: `(x, y, z)`
    /// - Lists: `[x, y, z]`
    fn parse_pattern(&mut self) -> Option<Pattern> {
        match &self.current_token {
            // Wildcard pattern
            Some(Token::Underscore) => {
                self.advance();
                Some(Pattern::Wildcard)
            }
            // Number literal pattern
            Some(Token::Number(n)) => {
                let pattern = Pattern::Literal(Box::new(Expression::Number(*n)));
                self.advance();
                Some(pattern)
            }
            // String literal pattern
            Some(Token::String(s)) => {
                let pattern = Pattern::Literal(Box::new(Expression::String(s.clone())));
                self.advance();
                Some(pattern)
            }
            // Boolean literal pattern
            Some(Token::Boolean(b)) => {
                let pattern = Pattern::Literal(Box::new(Expression::Boolean(*b)));
                self.advance();
                Some(pattern)
            }
            // None constructor pattern
            Some(Token::None) => {
                self.advance();
                Some(Pattern::Constructor {
                    name: "None".to_string(),
                    patterns: vec![],
                })
            }
            // Some constructor pattern
            Some(Token::Some) => {
                self.advance();
                // Expect '[' for Some[x]
                match self.current_token {
                    Some(Token::LeftBracket) => {
                        self.advance();
                        let pattern = self.parse_pattern()?;
                        // Expect ']'
                        match self.current_token {
                            Some(Token::RightBracket) => self.advance(),
                            _ => return None,
                        }
                        Some(Pattern::Constructor {
                            name: "Some".to_string(),
                            patterns: vec![pattern],
                        })
                    }
                    _ => None,
                }
            }
            // Ok constructor pattern
            Some(Token::Ok) => {
                self.advance();
                // Expect '[' for Ok[x]
                match self.current_token {
                    Some(Token::LeftBracket) => {
                        self.advance();
                        let pattern = self.parse_pattern()?;
                        // Expect ']'
                        match self.current_token {
                            Some(Token::RightBracket) => self.advance(),
                            _ => return None,
                        }
                        Some(Pattern::Constructor {
                            name: "Ok".to_string(),
                            patterns: vec![pattern],
                        })
                    }
                    _ => None,
                }
            }
            // Err constructor pattern
            Some(Token::Err) => {
                self.advance();
                // Expect '[' for Err[x]
                match self.current_token {
                    Some(Token::LeftBracket) => {
                        self.advance();
                        let pattern = self.parse_pattern()?;
                        // Expect ']'
                        match self.current_token {
                            Some(Token::RightBracket) => self.advance(),
                            _ => return None,
                        }
                        Some(Pattern::Constructor {
                            name: "Err".to_string(),
                            patterns: vec![pattern],
                        })
                    }
                    _ => None,
                }
            }
            // Identifier - could be variable binding or constructor
            Some(Token::Identifier(id)) => {
                let name = id.clone();
                self.advance();

                // Check if it's a constructor (followed by '[')
                if matches!(self.current_token, Some(Token::LeftBracket)) {
                    self.advance(); // Consume '['

                    let mut patterns = Vec::new();

                    // Parse constructor arguments
                    while !matches!(self.current_token, Some(Token::RightBracket)) {
                        patterns.push(self.parse_pattern()?);

                        // Handle comma between patterns
                        if matches!(self.current_token, Some(Token::Comma)) {
                            self.advance();
                        } else {
                            break;
                        }
                    }

                    // Consume ']'
                    match self.current_token {
                        Some(Token::RightBracket) => self.advance(),
                        _ => return None,
                    }

                    Some(Pattern::Constructor { name, patterns })
                } else {
                    // It's a variable binding
                    Some(Pattern::Variable(name))
                }
            }
            // Tuple pattern
            Some(Token::LeftParen) => {
                self.advance(); // Consume '('

                let mut patterns = Vec::new();

                while !matches!(self.current_token, Some(Token::RightParen)) {
                    patterns.push(self.parse_pattern()?);

                    // Handle comma between patterns
                    if matches!(self.current_token, Some(Token::Comma)) {
                        self.advance();
                    } else {
                        break;
                    }
                }

                // Consume ')'
                match self.current_token {
                    Some(Token::RightParen) => self.advance(),
                    _ => return None,
                }

                Some(Pattern::Tuple(patterns))
            }
            // List pattern
            Some(Token::LeftBracket) => {
                self.advance(); // Consume '['

                let mut patterns = Vec::new();

                while !matches!(self.current_token, Some(Token::RightBracket)) {
                    patterns.push(self.parse_pattern()?);

                    // Handle comma between patterns
                    if matches!(self.current_token, Some(Token::Comma)) {
                        self.advance();
                    } else {
                        break;
                    }
                }

                // Consume ']'
                match self.current_token {
                    Some(Token::RightBracket) => self.advance(),
                    _ => return None,
                }

                Some(Pattern::List(patterns))
            }
            _ => None,
        }
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

        Some(Expression::LogCall { level, message })
    }

    /// Parses a Some expression with the structure: Some[value]
    ///
    /// # Returns
    /// - `Some(Expression::Some)` if parsing succeeds
    /// - `None` if parsing fails
    fn parse_some_expression(&mut self) -> Option<Expression> {
        // Expect left bracket
        match self.current_token {
            Some(Token::LeftBracket) => self.advance(),
            _ => return None,
        }

        // Parse value
        let value = match self.parse_expression() {
            Some(expr) => Box::new(expr),
            None => return None,
        };

        // Expect right bracket
        match self.current_token {
            Some(Token::RightBracket) => self.advance(),
            _ => return None,
        }

        Some(Expression::Some { value })
    }

    /// Parses an Ok expression with the structure: Ok[value]
    ///
    /// # Returns
    /// - `Some(Expression::Ok)` if parsing succeeds
    /// - `None` if parsing fails
    fn parse_ok_expression(&mut self) -> Option<Expression> {
        // Expect left bracket
        match self.current_token {
            Some(Token::LeftBracket) => self.advance(),
            _ => return None,
        }

        // Parse value
        let value = match self.parse_expression() {
            Some(expr) => Box::new(expr),
            None => return None,
        };

        // Expect right bracket
        match self.current_token {
            Some(Token::RightBracket) => self.advance(),
            _ => return None,
        }

        Some(Expression::Ok { value })
    }

    /// Parses an Err expression with the structure: Err[error]
    ///
    /// # Returns
    /// - `Some(Expression::Err)` if parsing succeeds
    /// - `None` if parsing fails
    fn parse_err_expression(&mut self) -> Option<Expression> {
        // Expect left bracket
        match self.current_token {
            Some(Token::LeftBracket) => self.advance(),
            _ => return None,
        }

        // Parse error
        let error = match self.parse_expression() {
            Some(expr) => Box::new(expr),
            None => return None,
        };

        // Expect right bracket
        match self.current_token {
            Some(Token::RightBracket) => self.advance(),
            _ => return None,
        }

        Some(Expression::Err { error })
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

    fn parse_tuple(&mut self) -> Option<Expression> {
        // Consume left paren
        match self.current_token {
            Some(Token::LeftParen) => self.advance(),
            _ => return None,
        }

        let mut elements = Vec::new();
        while let Some(token) = &self.current_token {
            match token {
                Token::RightParen => break,
                _ => {
                    let elem = self.parse_expression()?;
                    elements.push(elem);

                    // Handle comma between elements
                    match self.current_token {
                        Some(Token::Comma) => self.advance(),
                        Some(Token::RightParen) => break,
                        _ => return None,
                    }
                }
            }
        }
        self.advance(); // Consume right paren

        Some(Expression::Tuple(elements))
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
    /// Recognizes all Rust primitive types and generic container types:
    /// - Primitives: Int8-128, UInt8-128, Float32/64, Bool, Char, String
    /// - Containers: List[T], Array[T, N], Slice[T], Map[K,V], HashSet[T], BTreeMap[K,V], BTreeSet[T]
    ///
    /// # Returns
    /// - `Some(Type)` if a valid type is found
    /// - `None` if the current token is not a recognized type identifier
    fn parse_type(&mut self) -> Option<Type> {
        match &self.current_token {
            Some(Token::Identifier(id)) => {
                let type_name = id.clone();
                self.advance();

                // Check if this is a generic type (followed by [)
                if matches!(self.current_token, Some(Token::LeftBracket)) {
                    return self.parse_generic_type(&type_name);
                }

                // Otherwise it's a primitive type or custom type
                let type_ = match type_name.as_str() {
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
                    "int" => Type::Int32,
                    "float" => Type::Float64,
                    "string" => Type::String,
                    "bool" => Type::Bool,
                    "char" => Type::Char,

                    // Unknown types are treated as custom types
                    _ => Type::Custom(type_name),
                };
                Some(type_)
            }
            _ => None,
        }
    }

    /// Parse generic type syntax like List[Int32], Array[Int32, 10], Map[String, Int32], Tuple[Int32, String, Bool]
    fn parse_generic_type(&mut self, type_name: &str) -> Option<Type> {
        // Consume the left bracket
        self.advance();

        match type_name {
            "Tuple" => {
                // Tuple[T1, T2, T3, ...]
                let mut types = Vec::new();
                loop {
                    match &self.current_token {
                        Some(Token::RightBracket) => break,
                        Some(Token::Comma) => {
                            self.advance();
                        }
                        _ => {
                            types.push(self.parse_type()?);
                        }
                    }
                }
                self.expect_token(Token::RightBracket)?;
                Some(Type::Tuple(types))
            }
            "List" => {
                let inner = Box::new(self.parse_type()?);
                self.expect_token(Token::RightBracket)?;
                Some(Type::List(inner))
            }
            "Array" => {
                // Array[T, N] where T is a type and N is a number
                let inner = Box::new(self.parse_type()?);
                self.expect_token(Token::Comma)?;

                // Parse the size as a number
                let size = match &self.current_token {
                    Some(Token::Number(n)) => {
                        let size = *n as usize;
                        self.advance();
                        size
                    }
                    _ => return None,
                };

                self.expect_token(Token::RightBracket)?;
                Some(Type::Array(inner, size))
            }
            "Slice" => {
                let inner = Box::new(self.parse_type()?);
                self.expect_token(Token::RightBracket)?;
                Some(Type::Slice(inner))
            }
            "HashSet" => {
                let inner = Box::new(self.parse_type()?);
                self.expect_token(Token::RightBracket)?;
                Some(Type::HashSet(inner))
            }
            "BTreeSet" => {
                let inner = Box::new(self.parse_type()?);
                self.expect_token(Token::RightBracket)?;
                Some(Type::BTreeSet(inner))
            }
            "Map" => {
                // Map[K, V]
                let key = Box::new(self.parse_type()?);
                self.expect_token(Token::Comma)?;
                let value = Box::new(self.parse_type()?);
                self.expect_token(Token::RightBracket)?;
                Some(Type::Map(key, value))
            }
            "BTreeMap" => {
                // BTreeMap[K, V]
                let key = Box::new(self.parse_type()?);
                self.expect_token(Token::Comma)?;
                let value = Box::new(self.parse_type()?);
                self.expect_token(Token::RightBracket)?;
                Some(Type::BTreeMap(key, value))
            }
            _ => None,
        }
    }

    /// Helper to expect and consume a specific token
    fn expect_token(&mut self, expected: Token) -> Option<()> {
        if self.current_token == Some(expected) {
            self.advance();
            Some(())
        } else {
            None
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
