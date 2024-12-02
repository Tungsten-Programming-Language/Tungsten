//! Rust Code Generation Module
//! 
//! Translates the W language AST into idiomatic Rust source code

use crate::ast::{Expression, Operator, LogLevel};
use std::fmt::Write;

pub struct RustCodeGenerator {
    output: String,
    indent_level: usize,
}

impl RustCodeGenerator {
    pub fn new() -> Self {
        RustCodeGenerator {
            output: String::new(),
            indent_level: 0,
        }
    }

    fn indent(&self) -> String {
        "    ".repeat(self.indent_level)
    }

    pub fn generate(&mut self, expr: &Expression) -> Result<String, std::fmt::Error> {
        // Reset output for each generation
        self.output.clear();
        self.indent_level = 0;

        // Start with a main function
        writeln!(self.output, "fn main() {{")?;
        self.indent_level += 1;

        // Generate the main expression
        self.generate_expression(expr)?;

        self.indent_level -= 1;
        writeln!(self.output, "}}")?;

        Ok(self.output.clone())
    }

    fn generate_expression(&mut self, expr: &Expression) -> Result<(), std::fmt::Error> {
        match expr {
            Expression::Number(n) => {
                write!(self.output, "{}println!(\"{}\");", self.indent(), n)?;
            }
            Expression::String(s) => {
                write!(self.output, "{}println!(\"{}\");", self.indent(), s)?;
            }
            Expression::LogCall { level, message } => {
                let log_macro = match level {
                    LogLevel::Debug => "debug!",
                    LogLevel::Info => "info!",
                    LogLevel::Warn => "warn!",
                    LogLevel::Error => "error!",
                };
                
                // Generate message
                let message_str = self.generate_log_message(message)?;
                write!(self.output, "{}{}({});", self.indent(), log_macro, message_str)?;
            }
            Expression::BinaryOp { left, operator, right } => {
                // Basic binary operation translation
                let op_str = match operator {
                    Operator::Add => "+",
                    Operator::Subtract => "-",
                    Operator::Multiply => "*",
                    Operator::Divide => "/",
                    Operator::Power => "pow",
                    _ => "/* unsupported */",
                };
                
                let left_val = self.generate_expression_as_value(left)?;
                let right_val = self.generate_expression_as_value(right)?;
                
                write!(self.output, "{}let result = {} {} {};", 
                    self.indent(), 
                    left_val, 
                    op_str, 
                    right_val
                )?;
            }
            _ => {
                write!(self.output, "{}// Unsupported expression", self.indent())?;
            }
        }
        Ok(())
    }

    fn generate_expression_as_value(&mut self, expr: &Expression) -> Result<String, std::fmt::Error> {
        match expr {
            Expression::Number(n) => Ok(n.to_string()),
            Expression::String(s) => Ok(format!("\"{}\"", s)),
            _ => Ok("/* complex expression */".to_string()),
        }
    }

    fn generate_log_message(&mut self, message: &Expression) -> Result<String, std::fmt::Error> {
        match message {
            Expression::String(s) => Ok(format!("\"{}\"", s)),
            Expression::Number(n) => Ok(n.to_string()),
            _ => Ok("\"unknown message\"".to_string()),
        }
    }
}
