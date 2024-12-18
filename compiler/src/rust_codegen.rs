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
                let result_val = self.generate_expression_as_value(expr)?;
                write!(self.output, "{}println!(\"{}\");", self.indent(), result_val)?;
            }
            Expression::FunctionCall { function, arguments } => {
                match function.as_ref() {
                    Expression::Identifier(name) if name == "Print" => {
                        // Generate print call that converts all arguments to strings
                        let print_args: Result<Vec<String>, std::fmt::Error> = arguments
                            .iter()
                            .map(|arg| self.generate_print_argument(arg))
                            .collect();
                        
                        let print_args = print_args?;
                        write!(
                            self.output, 
                            "{}println!(\"{}\");", 
                            self.indent(), 
                            print_args.join(" ")
                        )?;
                    }
                    _ => {
                        write!(self.output, "{}// Unsupported function call", self.indent())?;
                    }
                }
            }
            Expression::Cond { conditions, default_statements } => {
                // Generate Rust equivalent of Cond expression
                write!(self.output, "{}{{", self.indent())?;
                self.indent_level += 1;

                // Generate condition checks
                for (condition, statements) in conditions {
                    let condition_val = self.generate_expression_as_value(condition)?;
                    write!(
                        self.output, 
                        "\n{}if {} {{", 
                        self.indent(), 
                        condition_val
                    )?;
                    
                    self.indent_level += 1;
                    self.generate_expression(statements)?;
                    self.indent_level -= 1;
                    
                    write!(self.output, "\n{}break;", self.indent())?;
                    write!(self.output, "\n{}}", self.indent())?;
                }

                // Generate default statements if present
                if let Some(default_expr) = default_statements {
                    write!(self.output, "\n{}else {{", self.indent())?;
                    self.indent_level += 1;
                    self.generate_expression(default_expr)?;
                    self.indent_level -= 1;
                    write!(self.output, "\n{}}", self.indent())?;
                }

                self.indent_level -= 1;
                write!(self.output, "\n{}}}", self.indent())?;
            }
            _ => {
                write!(self.output, "{}// Unsupported expression", self.indent())?;
            }
        }
        Ok(())
    }

    // New helper method to convert arguments to printable strings
    fn generate_print_argument(&mut self, expr: &Expression) -> Result<String, std::fmt::Error> {
        match expr {
            Expression::Number(n) => Ok(n.to_string()),
            Expression::String(s) => Ok(format!("{}", s)),
            Expression::Boolean(b) => Ok(b.to_string()),
            _ => Ok("/* unsupported print arg */".to_string()),
        }
    }

    fn generate_expression_as_value(&mut self, expr: &Expression) -> Result<String, std::fmt::Error> {
        match expr {
            Expression::Number(n) => Ok(n.to_string()),
            Expression::String(s) => Ok(format!("\"{}\"", s)),
            Expression::BinaryOp { left, operator, right } => {
                let left_val = self.generate_expression_as_value(left)?;
                let right_val = self.generate_expression_as_value(right)?;
                
                let op_str = match operator {
                    Operator::Add => "+",
                    Operator::Subtract => "-",
                    Operator::Multiply => "*",
                    Operator::Divide => "/",
                    Operator::Power => "pow",
                    _ => "/* unsupported */",
                };
                
                Ok(format!("({} {} {})", left_val, op_str, right_val))
            }
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
