//! Rust Code Generation Module
//!
//! Translates the W language AST into idiomatic Rust source code

use crate::ast::{Expression, Operator, LogLevel, Type, TypeAnnotation, Pattern};
use std::fmt::Write;
use std::collections::HashMap;

pub struct RustCodeGenerator {
    output: String,
    indent_level: usize,
    /// Track if we're inside a function definition (to avoid wrapping in main)
    in_function: bool,
    /// Track defined struct names and their fields
    struct_definitions: HashMap<String, Vec<String>>,
}

impl RustCodeGenerator {
    pub fn new() -> Self {
        RustCodeGenerator {
            output: String::new(),
            indent_level: 0,
            in_function: false,
            struct_definitions: HashMap::new(),
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
                // Separate top-level items (structs, functions) from statements
                let mut top_level_items = Vec::new();
                let mut statements = Vec::new();

                for e in expressions {
                    match e {
                        Expression::FunctionDefinition { .. } | Expression::StructDefinition { .. } => {
                            top_level_items.push(e)
                        }
                        _ => statements.push(e),
                    }
                }

                // Generate all top-level items first (structs, then functions)
                for item in &top_level_items {
                    self.generate_top_level_item(item)?;
                    writeln!(self.output)?;
                }

                // Generate main function with statements
                if statements.is_empty() {
                    // Just top-level definitions, add stub main
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
            Expression::FunctionDefinition { .. } | Expression::StructDefinition { .. } => {
                // Single top-level definition
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

    /// Generate top-level items (functions, structs, etc.)
    fn generate_top_level_item(&mut self, expr: &Expression) -> Result<(), std::fmt::Error> {
        match expr {
            Expression::FunctionDefinition { name, parameters, body } => {
                self.generate_function_definition(name, parameters, body)?;
            }
            Expression::StructDefinition { name, fields } => {
                self.generate_struct_definition(name, fields)?;
            }
            _ => {
                // For other top-level items, generate as statement
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

    /// Generate a struct definition
    fn generate_struct_definition(
        &mut self,
        name: &str,
        fields: &[TypeAnnotation],
    ) -> Result<(), std::fmt::Error> {
        // Track this struct's field names for constructor detection
        let field_names: Vec<String> = fields.iter()
            .map(|f| to_snake_case(&f.name))
            .collect();
        self.struct_definitions.insert(name.to_string(), field_names);

        // Generate: #[derive(Debug, Clone, PartialEq)]
        //           pub struct Name {
        //               field1: Type1,
        //               field2: Type2,
        //           }
        writeln!(self.output, "{}#[derive(Debug, Clone, PartialEq)]", self.indent())?;
        writeln!(self.output, "{}pub struct {} {{", self.indent(), name)?;

        self.indent_level += 1;
        for field in fields {
            let field_name = to_snake_case(&field.name);
            let field_type = self.type_to_rust(&field.type_);
            writeln!(self.output, "{}pub {}: {},", self.indent(), field_name, field_type)?;
        }
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

            // Composite types
            Type::Tuple(types) => {
                if types.is_empty() {
                    "()".to_string()
                } else {
                    let type_strs: Vec<String> = types.iter()
                        .map(|t| self.type_to_rust(t))
                        .collect();
                    format!("({})", type_strs.join(", "))
                }
            }

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

            // User-defined types
            Type::Custom(name) => name.clone(),
        }
    }

    /// Infer return type from expression
    fn infer_return_type(&self, expr: &Expression, parameters: &[TypeAnnotation]) -> String {
        match expr {
            Expression::Number(_) => "i32".to_string(),  // Default to i32 like Rust
            Expression::Float(_) => "f64".to_string(),
            Expression::String(_) => "String".to_string(),
            Expression::Boolean(_) => "bool".to_string(),
            Expression::Tuple(elements) => {
                if elements.is_empty() {
                    "()".to_string()
                } else {
                    let element_types: Vec<String> = elements.iter()
                        .map(|e| self.infer_return_type(e, parameters))
                        .collect();
                    format!("({})", element_types.join(", "))
                }
            }
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
                                        Expression::List(_) | Expression::Map(_) | Expression::Tuple(_) => "{:?}".to_string(),
                                        // Also check for Map/Filter function calls that return Vec
                                        Expression::FunctionCall { function, .. } => {
                                            match function.as_ref() {
                                                Expression::Identifier(name) => {
                                                    // Check if it's Map/Filter or a struct constructor
                                                    if name == "Map" || name == "Filter" || self.struct_definitions.contains_key(name) {
                                                        "{:?}".to_string()
                                                    } else {
                                                        "{}".to_string()
                                                    }
                                                }
                                                _ => "{}".to_string(),
                                            }
                                        }
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

            Expression::Tuple(elements) => {
                // Generate tuple: (elem1, elem2, ...)
                if elements.is_empty() {
                    // Unit type
                    Ok("()".to_string())
                } else {
                    let mut result = String::from("(");
                    for (i, elem) in elements.iter().enumerate() {
                        if i > 0 {
                            result.push_str(", ");
                        }
                        result.push_str(&self.generate_expression_value(elem)?);
                    }
                    // Add trailing comma for single-element tuples (Rust requirement)
                    if elements.len() == 1 {
                        result.push(',');
                    }
                    result.push(')');
                    Ok(result)
                }
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
                        // Add type suffix to avoid ambiguity
                        Ok(format!("(({} as i32).pow({} as u32))", left_val, right_val))
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
                            "Tuple" => {
                                // Generate tuple from explicit Tuple[...] constructor
                                if arguments.is_empty() {
                                    Ok("()".to_string())
                                } else {
                                    let mut result = String::from("(");
                                    for (i, arg) in arguments.iter().enumerate() {
                                        if i > 0 {
                                            result.push_str(", ");
                                        }
                                        result.push_str(&self.generate_expression_value(arg)?);
                                    }
                                    // Add trailing comma for single-element tuples
                                    if arguments.len() == 1 {
                                        result.push(',');
                                    }
                                    result.push(')');
                                    Ok(result)
                                }
                            }
                            "Map" => {
                                // Map[function, list] -> list.into_iter().map(|x| function(x)).collect::<Vec<_>>()
                                if arguments.len() != 2 {
                                    return Err(std::fmt::Error);
                                }
                                let list = self.generate_expression_value(&arguments[1])?;
                                // Extract lambda body directly for better code generation
                                match &arguments[0] {
                                    Expression::Lambda { parameters, body } => {
                                        if parameters.len() == 1 {
                                            let param = &to_snake_case(&parameters[0].name);
                                            let body_str = self.generate_expression_value(body)?;
                                            Ok(format!("{}.into_iter().map(|{}| {}).collect::<Vec<_>>()",
                                                list, param, body_str))
                                        } else {
                                            Err(std::fmt::Error)
                                        }
                                    }
                                    _ => {
                                        let func = self.generate_expression_value(&arguments[0])?;
                                        Ok(format!("{}.into_iter().map({}).collect::<Vec<_>>()", list, func))
                                    }
                                }
                            }
                            "Filter" => {
                                // Filter[predicate, list] -> list.into_iter().filter(|&x| predicate(x)).collect::<Vec<_>>()
                                // Use pattern matching to get owned values from iterator
                                if arguments.len() != 2 {
                                    return Err(std::fmt::Error);
                                }
                                let func = self.generate_expression_value(&arguments[0])?;
                                let list = self.generate_expression_value(&arguments[1])?;
                                // Extract parameter name from lambda if possible
                                match &arguments[0] {
                                    Expression::Lambda { parameters, body } => {
                                        if parameters.len() == 1 {
                                            let param = &to_snake_case(&parameters[0].name);
                                            let body_str = self.generate_expression_value(body)?;
                                            // Use |&param| to pattern match and get owned value
                                            Ok(format!("{}.into_iter().filter(|&{}| {}).collect::<Vec<_>>()",
                                                list, param, body_str))
                                        } else {
                                            Err(std::fmt::Error)
                                        }
                                    }
                                    _ => {
                                        // For non-lambda functions, use the function directly
                                        Ok(format!("{}.into_iter().filter({}).collect::<Vec<_>>()", list, func))
                                    }
                                }
                            }
                            "Fold" => {
                                // Fold[function, init, list] -> list.into_iter().fold(init, |acc, x| function(acc, x))
                                if arguments.len() != 3 {
                                    return Err(std::fmt::Error);
                                }
                                let init = self.generate_expression_value(&arguments[1])?;
                                let list = self.generate_expression_value(&arguments[2])?;
                                // Extract lambda body directly
                                match &arguments[0] {
                                    Expression::Lambda { parameters, body } => {
                                        if parameters.len() == 2 {
                                            let param1 = &to_snake_case(&parameters[0].name);
                                            let param2 = &to_snake_case(&parameters[1].name);
                                            let body_str = self.generate_expression_value(body)?;
                                            Ok(format!("{}.into_iter().fold({}, |{}, {}| {})",
                                                list, init, param1, param2, body_str))
                                        } else {
                                            Err(std::fmt::Error)
                                        }
                                    }
                                    _ => {
                                        let func = self.generate_expression_value(&arguments[0])?;
                                        Ok(format!("{}.into_iter().fold({}, {})", list, init, func))
                                    }
                                }
                            }
                            "Print" => {
                                // Print returns (), so we generate a block
                                let mut result = String::from("{\n");
                                self.indent_level += 1;

                                write!(&mut result, "{}println!(", self.indent())?;
                                if !arguments.is_empty() {
                                    let format_parts: Vec<String> = arguments.iter()
                                        .map(|arg| {
                                            match arg {
                                                Expression::List(_) | Expression::Map(_) | Expression::Tuple(_) => "{:?}".to_string(),
                                                // Also check for Map/Filter function calls that return Vec
                                                Expression::FunctionCall { function, .. } => {
                                                    match function.as_ref() {
                                                        Expression::Identifier(name) => {
                                                            // Check if it's Map/Filter or a struct constructor
                                                            if name == "Map" || name == "Filter" || self.struct_definitions.contains_key(name) {
                                                                "{:?}".to_string()
                                                            } else {
                                                                "{}".to_string()
                                                            }
                                                        }
                                                        _ => "{}".to_string(),
                                                    }
                                                }
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
                                // Check if this is a struct constructor
                                if let Some(field_names) = self.struct_definitions.get(name).cloned() {
                                    // Generate struct instantiation: StructName { field1: value1, field2: value2 }
                                    if field_names.len() != arguments.len() {
                                        return Err(std::fmt::Error);
                                    }

                                    let mut result = format!("{} {{ ", name);
                                    for (i, (field_name, arg)) in field_names.iter().zip(arguments.iter()).enumerate() {
                                        if i > 0 {
                                            result.push_str(", ");
                                        }
                                        let arg_val = self.generate_expression_value(arg)?;
                                        result.push_str(&format!("{}: {}", field_name, arg_val));
                                    }
                                    result.push_str(" }");
                                    Ok(result)
                                } else {
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

            Expression::Match { value, arms } => {
                let value_str = self.generate_expression_value(value)?;
                let mut result = format!("match {} {{\n", value_str);

                for (pattern, expr) in arms {
                    let pattern_str = self.generate_pattern(pattern)?;
                    let expr_str = self.generate_expression_value(expr)?;
                    result.push_str(&format!("    {} => {},\n", pattern_str, expr_str));
                }

                result.push('}');
                Ok(result)
            }

            Expression::Lambda { parameters, body } => {
                // Generate Rust closure: |param1, param2, ...| body
                let mut result = String::from("|");

                for (i, param) in parameters.iter().enumerate() {
                    if i > 0 {
                        result.push_str(", ");
                    }
                    result.push_str(&to_snake_case(&param.name));

                    // Add type annotation if it's not the placeholder Int32
                    // In the future, we'll have proper type inference
                    // For now, only add type if it's explicitly different
                }

                result.push_str("| ");
                result.push_str(&self.generate_expression_value(body)?);

                Ok(result)
            }

            Expression::StructDefinition { .. } => {
                // Struct definitions should not appear in expression contexts
                Err(std::fmt::Error)
            }

            Expression::StructInstantiation { struct_name, field_values } => {
                // Generate: StructName { field1: value1, field2: value2 }
                // Look up the field names from the struct definition
                let field_names = self.struct_definitions.get(struct_name)
                    .cloned()
                    .ok_or(std::fmt::Error)?;

                if field_names.len() != field_values.len() {
                    // Mismatch between number of fields and values
                    return Err(std::fmt::Error);
                }

                let mut result = format!("{} {{ ", struct_name);

                // Generate field: value pairs
                for (i, (field_name, value)) in field_names.iter().zip(field_values.iter()).enumerate() {
                    if i > 0 {
                        result.push_str(", ");
                    }
                    let value_str = self.generate_expression_value(value)?;
                    result.push_str(&format!("{}: {}", field_name, value_str));
                }

                result.push_str(" }");
                Ok(result)
            }
        }
    }

    /// Generate Rust pattern syntax from Pattern AST
    fn generate_pattern(&self, pattern: &Pattern) -> Result<String, std::fmt::Error> {
        match pattern {
            Pattern::Wildcard => Ok("_".to_string()),

            Pattern::Literal(expr) => {
                match expr.as_ref() {
                    Expression::Number(n) => Ok(n.to_string()),
                    // String patterns match against &str in Rust
                    Expression::String(s) => Ok(format!("s if s == \"{}\"", s)),
                    Expression::Boolean(b) => Ok(b.to_string()),
                    _ => Err(std::fmt::Error),
                }
            }

            Pattern::Variable(name) => Ok(to_snake_case(name)),

            Pattern::Constructor { name, patterns } => {
                match name.as_str() {
                    "Some" => {
                        if patterns.len() == 1 {
                            let inner = self.generate_pattern(&patterns[0])?;
                            Ok(format!("Some({})", inner))
                        } else {
                            Err(std::fmt::Error)
                        }
                    }
                    "None" => Ok("None".to_string()),
                    "Ok" => {
                        if patterns.len() == 1 {
                            let inner = self.generate_pattern(&patterns[0])?;
                            Ok(format!("Ok({})", inner))
                        } else {
                            Err(std::fmt::Error)
                        }
                    }
                    "Err" => {
                        if patterns.len() == 1 {
                            let inner = self.generate_pattern(&patterns[0])?;
                            Ok(format!("Err({})", inner))
                        } else {
                            Err(std::fmt::Error)
                        }
                    }
                    _ => {
                        // Generic constructor - could be custom type
                        let mut result = format!("{}(", name);
                        for (i, p) in patterns.iter().enumerate() {
                            if i > 0 {
                                result.push_str(", ");
                            }
                            result.push_str(&self.generate_pattern(p)?);
                        }
                        result.push(')');
                        Ok(result)
                    }
                }
            }

            Pattern::Tuple(patterns) => {
                if patterns.is_empty() {
                    Ok("()".to_string())
                } else {
                    let mut result = String::from("(");
                    for (i, p) in patterns.iter().enumerate() {
                        if i > 0 {
                            result.push_str(", ");
                        }
                        result.push_str(&self.generate_pattern(p)?);
                    }
                    // Add trailing comma for single-element tuples
                    if patterns.len() == 1 {
                        result.push(',');
                    }
                    result.push(')');
                    Ok(result)
                }
            }

            Pattern::List(patterns) => {
                // In Rust, list patterns are represented as slices
                let mut result = String::from("[");
                for (i, p) in patterns.iter().enumerate() {
                    if i > 0 {
                        result.push_str(", ");
                    }
                    result.push_str(&self.generate_pattern(p)?);
                }
                result.push(']');
                Ok(result)
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
