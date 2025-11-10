//! Rust Code Generation Module
//!
//! Translates the W language AST into idiomatic Rust source code

use crate::ast::{Expression, Operator, LogLevel, Type, TypeAnnotation};
use std::fmt::Write;

pub struct RustCodeGenerator {
    output: String,
    indent_level: usize,
    /// Track if we're inside a function definition (to avoid wrapping in main)
    in_function: bool,
}

impl RustCodeGenerator {
    pub fn new() -> Self {
        RustCodeGenerator {
            output: String::new(),
            indent_level: 0,
            in_function: false,
        }
    }

    fn indent(&self) -> String {
        "    ".repeat(self.indent_level)
    }

    pub fn generate(&mut self, expr: &Expression) -> Result<String, std::fmt::Error> {
        // Reset output for each generation
        self.output.clear();
        self.indent_level = 0;

        // Check if this is a program with multiple expressions
        match expr {
            Expression::Program(expressions) => {
                // Separate function definitions from other expressions
                let mut functions = Vec::new();
                let mut statements = Vec::new();

                for e in expressions {
                    match e {
                        Expression::FunctionDefinition { .. } => functions.push(e),
                        _ => statements.push(e),
                    }
                }

                // Generate all function definitions first
                for func in &functions {
                    self.generate_top_level_item(func)?;
                    writeln!(self.output)?;
                }

                // Generate main function with statements
                if statements.is_empty() {
                    // Just function definitions, add stub main
                    writeln!(self.output, "fn main() {{")?;
                    writeln!(self.output, "    // Stub main function for compilation")?;
                    writeln!(self.output, "}}")?;
                } else {
                    // Generate main with statements
                    writeln!(self.output, "fn main() {{")?;
                    self.indent_level += 1;
                    for stmt in &statements {
                        self.generate_statement(stmt)?;
                    }
                    self.indent_level -= 1;
                    writeln!(self.output, "}}")?;
                }
            }
            Expression::FunctionDefinition { .. } => {
                // Single function definition
                self.generate_top_level_item(expr)?;
                // Add a stub main function to make it compilable
                writeln!(self.output)?;
                writeln!(self.output, "fn main() {{")?;
                writeln!(self.output, "    // Stub main function for compilation")?;
                writeln!(self.output, "}}")?;
            }
            _ => {
                // Single expression, wrap in main function
                writeln!(self.output, "fn main() {{")?;
                self.indent_level += 1;
                self.generate_statement(expr)?;
                self.indent_level -= 1;
                writeln!(self.output, "}}")?;
            }
        }

        Ok(self.output.clone())
    }

    /// Generate top-level items (functions, imports, etc.)
    fn generate_top_level_item(&mut self, expr: &Expression) -> Result<(), std::fmt::Error> {
        match expr {
            Expression::FunctionDefinition { name, parameters, body } => {
                self.generate_function_definition(name, parameters, body)?;
            }
            _ => {
                // For non-function top-level items, generate as statement
                self.generate_statement(expr)?;
            }
        }
        Ok(())
    }

    /// Generate a function definition
    fn generate_function_definition(
        &mut self,
        name: &str,
        parameters: &[TypeAnnotation],
        body: &Expression,
    ) -> Result<(), std::fmt::Error> {
        // Convert function name to snake_case (Rust convention)
        let rust_name = to_snake_case(name);

        write!(self.output, "{}fn {}(", self.indent(), rust_name)?;

        // Generate parameters
        for (i, param) in parameters.iter().enumerate() {
            if i > 0 {
                write!(self.output, ", ")?;
            }
            let param_name = to_snake_case(&param.name);
            let param_type = self.type_to_rust(&param.type_);
            write!(self.output, "{}: {}", param_name, param_type)?;
        }

        write!(self.output, ")")?;

        // Infer return type from body
        let return_type = self.infer_return_type(body, parameters);
        if return_type != "()" {
            write!(self.output, " -> {}", return_type)?;
        }

        writeln!(self.output, " {{")?;
        self.indent_level += 1;
        self.in_function = true;

        // Generate function body as an expression (no trailing semicolon for return)
        let body_code = self.generate_expression_value(body)?;
        // Write without newline from writeln to keep it as an expression
        write!(self.output, "{}{}\n", self.indent(), body_code)?;

        self.in_function = false;
        self.indent_level -= 1;
        writeln!(self.output, "{}}}", self.indent())?;

        Ok(())
    }

    /// Convert W type to Rust type
    fn type_to_rust(&self, ty: &Type) -> String {
        match ty {
            // Signed integers
            Type::Int8 => "i8".to_string(),
            Type::Int16 => "i16".to_string(),
            Type::Int32 => "i32".to_string(),
            Type::Int64 => "i64".to_string(),
            Type::Int128 => "i128".to_string(),
            Type::Int => "isize".to_string(),

            // Unsigned integers
            Type::UInt8 => "u8".to_string(),
            Type::UInt16 => "u16".to_string(),
            Type::UInt32 => "u32".to_string(),
            Type::UInt64 => "u64".to_string(),
            Type::UInt128 => "u128".to_string(),
            Type::UInt => "usize".to_string(),

            // Floating point
            Type::Float32 => "f32".to_string(),
            Type::Float64 => "f64".to_string(),

            // Other primitives
            Type::Bool => "bool".to_string(),
            Type::Char => "char".to_string(),
            Type::String => "String".to_string(),

            // Complex types
            Type::List(inner) => format!("Vec<{}>", self.type_to_rust(inner)),
            Type::Array(inner, size) => format!("[{}; {}]", self.type_to_rust(inner), size),
            Type::Slice(inner) => format!("&[{}]", self.type_to_rust(inner)),
            Type::Map(key, value) => {
                format!("std::collections::HashMap<{}, {}>",
                    self.type_to_rust(key),
                    self.type_to_rust(value))
            }
            Type::HashSet(inner) => format!("std::collections::HashSet<{}>", self.type_to_rust(inner)),
            Type::BTreeMap(key, value) => {
                format!("std::collections::BTreeMap<{}, {}>",
                    self.type_to_rust(key),
                    self.type_to_rust(value))
            }
            Type::BTreeSet(inner) => format!("std::collections::BTreeSet<{}>", self.type_to_rust(inner)),
            Type::Function(params, ret) => {
                let param_types: Vec<String> = params.iter()
                    .map(|p| self.type_to_rust(p))
                    .collect();
                format!("fn({}) -> {}", param_types.join(", "), self.type_to_rust(ret))
            }

            // Error handling types (Rust's safety model)
            Type::Option(inner) => format!("Option<{}>", self.type_to_rust(inner)),
            Type::Result(ok_type, err_type) => {
                format!("Result<{}, {}>",
                    self.type_to_rust(ok_type),
                    self.type_to_rust(err_type))
            }

            // Special types
            Type::LogLevel => "LogLevel".to_string(),
        }
    }

    /// Infer return type from expression
    fn infer_return_type(&self, expr: &Expression, parameters: &[TypeAnnotation]) -> String {
        match expr {
            Expression::Number(_) => "i32".to_string(),  // Default to i32 like Rust
            Expression::Float(_) => "f64".to_string(),
            Expression::String(_) => "String".to_string(),
            Expression::Boolean(_) => "bool".to_string(),
            Expression::List(_) => "Vec<i32>".to_string(), // Simplified
            Expression::Map(_) => "HashMap<String, String>".to_string(), // Simplified
            Expression::Identifier(name) => {
                // Look up the parameter type
                for param in parameters {
                    if param.name == *name {
                        return self.type_to_rust(&param.type_);
                    }
                }
                "()".to_string()
            }
            Expression::BinaryOp { left, right: _, operator } => {
                // Infer from left operand (simplified)
                let left_type = self.infer_return_type(left, parameters);
                // For arithmetic operations, return the inferred type
                match operator {
                    Operator::Add | Operator::Subtract | Operator::Multiply | Operator::Divide => {
                        // If left is a known numeric type, return it
                        if matches!(left_type.as_str(), "i8" | "i16" | "i32" | "i64" | "i128" | "isize" |
                                    "u8" | "u16" | "u32" | "u64" | "u128" | "usize" |
                                    "f32" | "f64") {
                            left_type
                        } else {
                            "i32".to_string() // Default
                        }
                    }
                    _ => "i32".to_string(),
                }
            }
            // Error handling types
            Expression::None => "Option<()>".to_string(),  // Type needs context
            Expression::Some { value } => {
                let inner_type = self.infer_return_type(value, parameters);
                format!("Option<{}>", inner_type)
            }
            Expression::Ok { value } => {
                let ok_type = self.infer_return_type(value, parameters);
                format!("Result<{}, ()>", ok_type)  // Error type needs context
            }
            Expression::Err { error } => {
                let err_type = self.infer_return_type(error, parameters);
                format!("Result<(), {}>", err_type)  // Ok type needs context
            }
            _ => "()".to_string(),
        }
    }

    /// Generate a statement (expression with side effects, like println or assignments)
    fn generate_statement(&mut self, expr: &Expression) -> Result<(), std::fmt::Error> {
        match expr {
            Expression::FunctionCall { function, arguments } => {
                match function.as_ref() {
                    Expression::Identifier(name) if name == "Print" => {
                        // Generate print call
                        write!(self.output, "{}println!(", self.indent())?;

                        // Generate format string with appropriate formatters
                        if !arguments.is_empty() {
                            let format_parts: Vec<String> = arguments.iter()
                                .map(|arg| {
                                    // Use {:?} for complex types that don't implement Display
                                    match arg {
                                        Expression::List(_) | Expression::Map(_) => "{:?}".to_string(),
                                        _ => "{}".to_string(),
                                    }
                                })
                                .collect();
                            write!(self.output, "\"{}\"", format_parts.join(" "))?;

                            // Add arguments
                            for arg in arguments {
                                write!(self.output, ", ")?;
                                let arg_val = self.generate_expression_value(arg)?;
                                write!(self.output, "{}", arg_val)?;
                            }
                        }

                        writeln!(self.output, ");")?;
                    }
                    _ => {
                        // Generic function call
                        let call_expr = self.generate_expression_value(expr)?;
                        writeln!(self.output, "{}{};", self.indent(), call_expr)?;
                    }
                }
            }
            _ => {
                // For other expressions, generate as value and discard
                let value = self.generate_expression_value(expr)?;
                writeln!(self.output, "{}{};", self.indent(), value)?;
            }
        }
        Ok(())
    }

    /// Generate an expression that returns a value (not a statement)
    fn generate_expression_value(&mut self, expr: &Expression) -> Result<String, std::fmt::Error> {
        match expr {
            Expression::Program(_) => {
                // Program nodes should not appear in expression contexts
                Err(std::fmt::Error)
            }
            Expression::Number(n) => Ok(n.to_string()),

            Expression::Float(f) => Ok(f.to_string()),

            Expression::String(s) => Ok(format!("\"{}\".to_string()", s)),

            Expression::Boolean(b) => Ok(b.to_string()),

            Expression::Identifier(name) => {
                // Convert to snake_case
                Ok(to_snake_case(name))
            }

            Expression::List(elements) => {
                // Generate vec![...]
                let mut result = String::from("vec![");
                for (i, elem) in elements.iter().enumerate() {
                    if i > 0 {
                        result.push_str(", ");
                    }
                    result.push_str(&self.generate_expression_value(elem)?);
                }
                result.push(']');
                Ok(result)
            }

            Expression::Map(entries) => {
                // Generate HashMap initialization
                let mut result = String::from("{\n");
                self.indent_level += 1;
                result.push_str(&format!("{}let mut map = std::collections::HashMap::new();\n", self.indent()));

                for (key, value) in entries {
                    let key_val = self.generate_expression_value(key)?;
                    let value_val = self.generate_expression_value(value)?;
                    result.push_str(&format!("{}map.insert({}, {});\n", self.indent(), key_val, value_val));
                }

                result.push_str(&format!("{}map\n", self.indent()));
                self.indent_level -= 1;
                result.push_str(&format!("{}}}", self.indent()));
                Ok(result)
            }

            Expression::BinaryOp { left, operator, right } => {
                let left_val = self.generate_expression_value(left)?;
                let right_val = self.generate_expression_value(right)?;

                match operator {
                    Operator::Add => Ok(format!("({} + {})", left_val, right_val)),
                    Operator::Subtract => Ok(format!("({} - {})", left_val, right_val)),
                    Operator::Multiply => Ok(format!("({} * {})", left_val, right_val)),
                    Operator::Divide => Ok(format!("({} / {})", left_val, right_val)),
                    Operator::Power => {
                        // Use pow for integer exponentiation
                        Ok(format!("({}.pow({} as u32))", left_val, right_val))
                    }
                    Operator::Equals => Ok(format!("({} == {})", left_val, right_val)),
                    Operator::NotEquals => Ok(format!("({} != {})", left_val, right_val)),
                    Operator::LessThan => Ok(format!("({} < {})", left_val, right_val)),
                    Operator::GreaterThan => Ok(format!("({} > {})", left_val, right_val)),
                }
            }

            Expression::FunctionCall { function, arguments } => {
                match function.as_ref() {
                    Expression::Identifier(name) => {
                        // Check for built-in functions
                        match name.as_str() {
                            "Print" => {
                                // Print returns (), so we generate a block
                                let mut result = String::from("{\n");
                                self.indent_level += 1;

                                write!(&mut result, "{}println!(", self.indent())?;
                                if !arguments.is_empty() {
                                    let format_parts: Vec<String> = arguments.iter()
                                        .map(|arg| {
                                            match arg {
                                                Expression::List(_) | Expression::Map(_) => "{:?}".to_string(),
                                                _ => "{}".to_string(),
                                            }
                                        })
                                        .collect();
                                    write!(&mut result, "\"{}\"", format_parts.join(" "))?;

                                    for arg in arguments {
                                        write!(&mut result, ", ")?;
                                        let arg_val = self.generate_expression_value(arg)?;
                                        write!(&mut result, "{}", arg_val)?;
                                    }
                                }
                                write!(&mut result, ");\n")?;

                                self.indent_level -= 1;
                                result.push_str(&format!("{}}}", self.indent()));
                                Ok(result)
                            }
                            _ => {
                                // Generic function call
                                let func_name = to_snake_case(name);
                                let mut result = format!("{}(", func_name);

                                for (i, arg) in arguments.iter().enumerate() {
                                    if i > 0 {
                                        result.push_str(", ");
                                    }
                                    result.push_str(&self.generate_expression_value(arg)?);
                                }

                                result.push(')');
                                Ok(result)
                            }
                        }
                    }
                    _ => Ok("/* unsupported function call */".to_string()),
                }
            }

            Expression::Cond { conditions, default_statements } => {
                // Generate if-else chain
                let mut result = String::new();

                for (i, (condition, statements)) in conditions.iter().enumerate() {
                    if i > 0 {
                        result.push_str(" else ");
                    }

                    let cond_val = self.generate_expression_value(condition)?;
                    write!(&mut result, "if {} {{\n", cond_val)?;

                    self.indent_level += 1;
                    let stmt_val = self.generate_expression_value(statements)?;
                    write!(&mut result, "{}{}\n", self.indent(), stmt_val)?;
                    self.indent_level -= 1;

                    write!(&mut result, "{}}}", self.indent())?;
                }

                // Generate default case if present
                if let Some(default_expr) = default_statements {
                    write!(&mut result, " else {{\n")?;
                    self.indent_level += 1;
                    let default_val = self.generate_expression_value(default_expr)?;
                    write!(&mut result, "{}{}\n", self.indent(), default_val)?;
                    self.indent_level -= 1;
                    write!(&mut result, "{}}}", self.indent())?;
                }

                Ok(result)
            }

            Expression::LogCall { level, message } => {
                let log_macro = match level {
                    LogLevel::Debug => "debug!",
                    LogLevel::Info => "info!",
                    LogLevel::Warn => "warn!",
                    LogLevel::Error => "error!",
                };

                let message_val = self.generate_expression_value(message)?;
                Ok(format!("{}({})", log_macro, message_val))
            }

            Expression::FunctionDefinition { .. } => {
                Ok("/* function definitions not supported as values */".to_string())
            }

            // Error handling expressions (Rust's safety model)
            Expression::None => Ok("None".to_string()),

            Expression::Some { value } => {
                let value_str = self.generate_expression_value(value)?;
                Ok(format!("Some({})", value_str))
            }

            Expression::Ok { value } => {
                let value_str = self.generate_expression_value(value)?;
                Ok(format!("Ok({})", value_str))
            }

            Expression::Err { error } => {
                let error_str = self.generate_expression_value(error)?;
                Ok(format!("Err({})", error_str))
            }
        }
    }
}

/// Convert PascalCase or camelCase to snake_case
fn to_snake_case(s: &str) -> String {
    let mut result = String::new();
    let mut prev_is_upper = false;

    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() {
            if i > 0 && !prev_is_upper {
                result.push('_');
            }
            result.push(c.to_ascii_lowercase());
            prev_is_upper = true;
        } else {
            result.push(c);
            prev_is_upper = false;
        }
    }

    result
}
