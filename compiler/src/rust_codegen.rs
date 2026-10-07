//! Rust Code Generation Module
//!
//! Translates the W language AST into idiomatic Rust source code

use crate::ast::{Expression, LogLevel, Operator, Pattern, Type, TypeAnnotation, UnaryOperator};
use std::collections::HashMap;
use std::fmt::Write;

/// Arm classification for Cond/If type normalization.
/// - Unit: definitely evaluates to () (Print, Break, Continue)
/// - Value: definitely evaluates to a non-unit value
/// - Unknown: cannot tell (bare identifier not in parameters, e.g. a
///   With-bound variable, or an unknown call) - never drives normalization
#[derive(Clone, Copy, PartialEq)]
enum ArmKind {
    Unit,
    Value,
    Unknown,
}

pub struct RustCodeGenerator {
    output: String,
    indent_level: usize,
    /// Track if we're inside a function definition (to avoid wrapping in main)
    in_function: bool,
    /// Track defined struct names and their fields
    struct_definitions: HashMap<String, Vec<String>>,
    /// Parameters of the function being generated, for type-aware inference
    parameters: Vec<TypeAnnotation>,
    /// Inferred return types of user-defined functions (name -> Rust type)
    fn_returns: HashMap<String, String>,
}

impl RustCodeGenerator {
    pub fn new() -> Self {
        RustCodeGenerator {
            output: String::new(),
            indent_level: 0,
            in_function: false,
            struct_definitions: HashMap::new(),
            parameters: Vec::new(),
            fn_returns: HashMap::new(),
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
                        Expression::FunctionDefinition { .. }
                        | Expression::StructDefinition { .. } => top_level_items.push(e),
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
            Expression::FunctionDefinition {
                name,
                parameters,
                body,
            } => {
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
        // Record for call-site inference (used by Cond/If arm classification)
        self.fn_returns.insert(name.to_string(), return_type);

        writeln!(self.output, " {{")?;
        self.indent_level += 1;
        self.in_function = true;
        // Track parameters so nested Cond/If codegen can infer arm types
        // (e.g. a bare identifier `lines` resolves to its List[String] type)
        self.parameters = parameters.to_vec();

        // Generate function body as an expression (no trailing semicolon for return)
        let body_code = self.generate_expression_value(body)?;
        // Write without newline from writeln to keep it as an expression
        write!(self.output, "{}{}\n", self.indent(), body_code)?;

        self.parameters = Vec::new();
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
        let field_names: Vec<String> = fields.iter().map(|f| to_snake_case(&f.name)).collect();
        self.struct_definitions
            .insert(name.to_string(), field_names);

        // Generate: #[derive(Debug, Clone, PartialEq)]
        //           pub struct Name {
        //               field1: Type1,
        //               field2: Type2,
        //           }
        writeln!(
            self.output,
            "{}#[derive(Debug, Clone, PartialEq)]",
            self.indent()
        )?;
        writeln!(self.output, "{}pub struct {} {{", self.indent(), name)?;

        self.indent_level += 1;
        for field in fields {
            let field_name = to_snake_case(&field.name);
            let field_type = self.type_to_rust(&field.type_);
            writeln!(
                self.output,
                "{}pub {}: {},",
                self.indent(),
                field_name,
                field_type
            )?;
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
                    let type_strs: Vec<String> =
                        types.iter().map(|t| self.type_to_rust(t)).collect();
                    format!("({})", type_strs.join(", "))
                }
            }

            // Complex types
            Type::List(inner) => format!("Vec<{}>", self.type_to_rust(inner)),
            Type::Array(inner, size) => format!("[{}; {}]", self.type_to_rust(inner), size),
            Type::Slice(inner) => format!("&[{}]", self.type_to_rust(inner)),
            Type::Map(key, value) => {
                format!(
                    "std::collections::HashMap<{}, {}>",
                    self.type_to_rust(key),
                    self.type_to_rust(value)
                )
            }
            Type::HashSet(inner) => {
                format!("std::collections::HashSet<{}>", self.type_to_rust(inner))
            }
            Type::BTreeMap(key, value) => {
                format!(
                    "std::collections::BTreeMap<{}, {}>",
                    self.type_to_rust(key),
                    self.type_to_rust(value)
                )
            }
            Type::BTreeSet(inner) => {
                format!("std::collections::BTreeSet<{}>", self.type_to_rust(inner))
            }
            Type::Iterator(inner) => {
                format!("std::iter::Iterator<Item = {}>", self.type_to_rust(inner))
            }
            Type::Function(params, ret) => {
                let param_types: Vec<String> =
                    params.iter().map(|p| self.type_to_rust(p)).collect();
                format!(
                    "fn({}) -> {}",
                    param_types.join(", "),
                    self.type_to_rust(ret)
                )
            }

            // Error handling types (Rust's safety model)
            Type::Option(inner) => format!("Option<{}>", self.type_to_rust(inner)),
            Type::Result(ok_type, err_type) => {
                format!(
                    "Result<{}, {}>",
                    self.type_to_rust(ok_type),
                    self.type_to_rust(err_type)
                )
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
            Expression::Number(_) => "i64".to_string(), // Default to i64 for benchmarks
            Expression::Float(_) => "f64".to_string(),
            Expression::String(_) => "String".to_string(),
            Expression::Boolean(_) => "bool".to_string(),
            Expression::Tuple(elements) => {
                if elements.is_empty() {
                    "()".to_string()
                } else {
                    let element_types: Vec<String> = elements
                        .iter()
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
            Expression::BinaryOp {
                left,
                right: _,
                operator,
            } => {
                // Infer from left operand (simplified)
                let left_type = self.infer_return_type(left, parameters);
                // For arithmetic operations, return the inferred type
                match operator {
                    Operator::Add | Operator::Subtract | Operator::Multiply | Operator::Divide => {
                        // If left is a known numeric type, return it
                        if matches!(
                            left_type.as_str(),
                            "i8" | "i16"
                                | "i32"
                                | "i64"
                                | "i128"
                                | "isize"
                                | "u8"
                                | "u16"
                                | "u32"
                                | "u64"
                                | "u128"
                                | "usize"
                                | "f32"
                                | "f64"
                        ) {
                            left_type
                        } else {
                            "i32".to_string() // Default
                        }
                    }
                    Operator::Equals
                    | Operator::NotEquals
                    | Operator::LessThan
                    | Operator::GreaterThan
                    | Operator::LessEqual
                    | Operator::GreaterEqual => "bool".to_string(),
                    Operator::And | Operator::Or => "bool".to_string(),
                    _ => "i32".to_string(),
                }
            }
            Expression::UnaryOp {
                operator,
                operand: _,
            } => match operator {
                UnaryOperator::Not => "bool".to_string(),
            },
            // Error handling types
            Expression::None => "Option<()>".to_string(), // Type needs context
            Expression::Some { value } => {
                let inner_type = self.infer_return_type(value, parameters);
                format!("Option<{}>", inner_type)
            }
            Expression::Ok { value } => {
                let ok_type = self.infer_return_type(value, parameters);
                format!("Result<{}, ()>", ok_type) // Error type needs context
            }
            Expression::Err { error } => {
                let err_type = self.infer_return_type(error, parameters);
                format!("Result<(), {}>", err_type) // Ok type needs context
            }
            Expression::Propagate { expr } => {
                // ? unwraps the inner type
                self.infer_return_type(expr, parameters)
            }
            Expression::With { body, .. } => {
                // Return type is the type of the body expression
                self.infer_return_type(body, parameters)
            }
            Expression::Module { body, .. } => {
                // Return type is the type of the body expression
                self.infer_return_type(body, parameters)
            }
            Expression::Cond {
                conditions,
                default_statements,
            } => {
                // Infer return type from first condition's body (all branches should have same type)
                if !conditions.is_empty() {
                    self.infer_return_type(&conditions[0].1, parameters)
                } else if let Some(default) = default_statements {
                    self.infer_return_type(default, parameters)
                } else {
                    "()".to_string()
                }
            }
            Expression::Match { arms, .. } => {
                // Infer return type from the first arm with a concrete type.
                // Pattern-bound identifiers (Some[content] -> content) are not in
                // parameters, so they infer as "()"; skip those and pick a concrete arm.
                for (_, arm_expr) in arms {
                    let t = self.infer_return_type(arm_expr, parameters);
                    if t != "()" {
                        return t;
                    }
                }
                "()".to_string()
            }
            Expression::FunctionCall {
                function,
                arguments,
            } => match function.as_ref() {
                Expression::Identifier(name) => match name.as_str() {
                    "First" => "i64".to_string(),
                    "Length" => "usize".to_string(),
                    "Nth" => "i64".to_string(),
                    "Sqrt" | "Sin" | "Cos" => "f64".to_string(),
                    "ParseInt" => "i64".to_string(),
                    "Args" => "Vec<String>".to_string(),
                    "ReadLine" => "Option<String>".to_string(),
                    "ReadFile" => "Option<String>".to_string(),
                    "WriteFile" => "Result<(), String>".to_string(),
                    "ToString" => "String".to_string(),
                    "StringJoin" => "String".to_string(),
                    "StringSplit" => "Vec<String>".to_string(),
                    "If" => {
                        if arguments.len() >= 2 {
                            self.infer_return_type(&arguments[1], parameters)
                        } else {
                            "()".to_string()
                        }
                    }
                    "Rest" | "Reverse" | "Concat" | "Set" | "Take" | "Range" => {
                        // Infer list element type from the list argument when possible
                        // (argument[0] for Rest/Reverse/Set/Range, argument[1] for Concat/Take)
                        let list_arg_idx = if name == "Concat" || name == "Take" { 1 } else { 0 };
                        let elem_type = if arguments.len() > list_arg_idx {
                            match &arguments[list_arg_idx] {
                                Expression::Identifier(id) => {
                                    // Look up parameter type: Vec<T> -> T
                                    let full = parameters
                                        .iter()
                                        .find(|p| p.name == *id)
                                        .map(|p| self.type_to_rust(&p.type_))
                                        .unwrap_or_else(|| "Vec<i64>".to_string());
                                    full.trim_start_matches("Vec<").trim_end_matches('>').to_string()
                                }
                                _ => "i64".to_string(),
                            }
                        } else {
                            "i64".to_string()
                        };
                        format!("Vec<{}>", elem_type)
                    }
                    "Append" => {
                        if arguments.len() >= 1 {
                            self.infer_return_type(&arguments[0], parameters)
                        } else {
                            "Vec<i64>".to_string()
                        }
                    }
                    "Map" | "Filter" | "Take" => {
                        if arguments.len() >= 2 {
                            self.infer_return_type(&arguments[1], parameters)
                        } else {
                            "Vec<i64>".to_string()
                        }
                    }
                    "LazyMap" | "LazyFilter" => {
                        // Returns an iterator (simplified as impl Iterator)
                        if arguments.len() >= 2 {
                            self.infer_return_type(&arguments[1], parameters)
                        } else {
                            "impl Iterator<Item = i64>".to_string()
                        }
                    }
                    "Collect" => {
                        // Returns a Vec
                        if arguments.len() >= 1 {
                            self.infer_return_type(&arguments[0], parameters)
                        } else {
                            "Vec<i64>".to_string()
                        }
                    }
                    "FlatMap" => {
                        if arguments.len() >= 2 {
                            self.infer_return_type(&arguments[1], parameters)
                        } else {
                            "Vec<i64>".to_string()
                        }
                    }
                    "Zip" => "Vec<(i64, i64)>".to_string(),
                    "GroupBy" => "std::collections::HashMap<i64, Vec<i64>>".to_string(),
                    "Fold" => {
                        if arguments.len() >= 2 {
                            self.infer_return_type(&arguments[1], parameters)
                        } else {
                            "()".to_string()
                        }
                    }
                    _ => self
                        .fn_returns
                        .get(name)
                        .cloned()
                        .unwrap_or_else(|| "()".to_string()),
                },
                _ => "()".to_string(),
            },
            _ => "()".to_string(),
        }
    }

    /// Classify a Cond/If arm's value kind for unit-normalization decisions
    fn classify_arm(&self, expr: &Expression) -> ArmKind {
        match expr {
            // Definitely unit side-effects
            Expression::Break | Expression::Continue => ArmKind::Unit,
            Expression::FunctionCall { function, .. } => {
                if let Expression::Identifier(name) = function.as_ref() {
                    if name == "Print" {
                        return ArmKind::Unit;
                    }
                }
                // Classify by inferred type (builtins and user fns via fn_returns)
                let t = self.infer_return_type(expr, &self.parameters);
                if t == "()" {
                    ArmKind::Unit
                } else {
                    ArmKind::Value
                }
            }
            // Bare identifier: unit only if it's a parameter (which has a known
            // type); a With-bound or unresolved identifier is Unknown so it does
            // not force normalization (e.g. binary_trees' `n`)
            Expression::Identifier(name) => {
                let is_param = self.parameters.iter().any(|p| p.name == *name);
                if is_param {
                    let t = self.infer_return_type(expr, &self.parameters);
                    if t == "()" {
                        ArmKind::Unit
                    } else {
                        ArmKind::Value
                    }
                } else {
                    ArmKind::Unknown
                }
            }
            _ => {
                let t = self.infer_return_type(expr, &self.parameters);
                if t == "()" {
                    ArmKind::Unit
                } else {
                    ArmKind::Value
                }
            }
        }
    }

    /// Generate a statement (expression with side effects, like println or assignments)
    fn generate_statement(&mut self, expr: &Expression) -> Result<(), std::fmt::Error> {
        match expr {
            Expression::FunctionCall {
                function,
                arguments,
            } => {
                match function.as_ref() {
                    Expression::Identifier(name) if name == "Print" => {
                        // Generate print call
                        write!(self.output, "{}println!(", self.indent())?;

                        // Generate format string with appropriate formatters
                        if !arguments.is_empty() {
                            let format_parts: Vec<String> = arguments
                                .iter()
                                .map(|arg| {
                                    // Use {:?} for complex types that don't implement Display
                                    match arg {
                                        Expression::List(_)
                                        | Expression::Map(_)
                                        | Expression::Tuple(_) => "{:?}".to_string(),
                                        // Also check for Map/Filter function calls that return Vec
                                        Expression::FunctionCall { function, .. } => {
                                            match function.as_ref() {
                                                Expression::Identifier(name) => {
                                                    // Check if it returns a Vec type (needs debug format)
                                                    if name == "Map"
                                                        || name == "Filter"
                                                        || name == "Args"
                                                        || name == "Range"
                                                        || name == "Reverse"
                                                        || name == "Concat"
                                                        || name == "Rest"
                                                        || name == "Set"
                                                        || name == "Take"
                                                        || name == "Zip"
                                                        || name == "FlatMap"
                                                        || name == "GroupBy"
                                                        || name == "Collect"
                                                        || name == "Append"
                                                        || name == "ReadFile"
                                                        || name == "WriteFile"
                                                        || name == "StringSplit"
                                                        || self
                                                            .struct_definitions
                                                            .contains_key(name)
                                                    {
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

            Expression::Float(f) => Ok(format!("{}f64", f)),

            Expression::String(s) => Ok(format!("\"{}\".to_string()", s)),

            Expression::Boolean(b) => Ok(b.to_string()),

            Expression::Identifier(name) => match name.as_str() {
                "Pi" => Ok("std::f64::consts::PI".to_string()),
                _ => Ok(to_snake_case(name)),
            },

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
                result.push_str(&format!(
                    "{}let mut map = std::collections::HashMap::new();\n",
                    self.indent()
                ));

                for (key, value) in entries {
                    let key_val = self.generate_expression_value(key)?;
                    let value_val = self.generate_expression_value(value)?;
                    result.push_str(&format!(
                        "{}map.insert({}, {});\n",
                        self.indent(),
                        key_val,
                        value_val
                    ));
                }

                result.push_str(&format!("{}map\n", self.indent()));
                self.indent_level -= 1;
                result.push_str(&format!("{}}}", self.indent()));
                Ok(result)
            }

            Expression::BinaryOp {
                left,
                operator,
                right,
            } => {
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
                    Operator::LessEqual => Ok(format!("({} <= {})", left_val, right_val)),
                    Operator::GreaterEqual => Ok(format!("({} >= {})", left_val, right_val)),
                    Operator::And => Ok(format!("({} && {})", left_val, right_val)),
                    Operator::Or => Ok(format!("({} || {})", left_val, right_val)),
                }
            }

            Expression::UnaryOp { operator, operand } => {
                let operand_val = self.generate_expression_value(operand)?;
                match operator {
                    UnaryOperator::Not => Ok(format!("(!{})", operand_val)),
                }
            }

            Expression::FunctionCall {
                function,
                arguments,
            } => {
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
                            "If" => {
                                // If[cond, then] or If[cond, then, else] -> Rust if/else expression
                                // Without an else branch, the else arm is the unit value ()
                                if arguments.len() < 2 || arguments.len() > 3 {
                                    return Err(std::fmt::Error);
                                }
                                let cond = self.generate_expression_value(&arguments[0])?;
                                let then_val = self.generate_expression_value(&arguments[1])?;
                                // If If arms mix a definite-unit side-effect with a definite value,
                                // append ";" after every arm so all arms normalize
                                // to unit. All-value Ifs stay as values.
                                let mut has_unit = self.classify_arm(&arguments[1]) == ArmKind::Unit;
                                let mut has_value = self.classify_arm(&arguments[1]) == ArmKind::Value;
                                if arguments.len() == 3 {
                                    let k = self.classify_arm(&arguments[2]);
                                    if k == ArmKind::Unit {
                                        has_unit = true;
                                    } else if k == ArmKind::Value {
                                        has_value = true;
                                    }
                                }
                                let is_unit = has_unit && has_value;
                                let then_suffix = if is_unit { ";" } else { "" };
                                let mut result = format!("if {} {{\n", cond);
                                self.indent_level += 1;
                                result.push_str(&format!(
                                    "{}{}{}\n",
                                    self.indent(),
                                    then_val,
                                    then_suffix
                                ));
                                self.indent_level -= 1;
                                result.push_str(&format!("{}}}", self.indent()));
                                if arguments.len() == 3 {
                                    let else_val = self.generate_expression_value(&arguments[2])?;
                                    result.push_str(&format!(" else {{\n"));
                                    self.indent_level += 1;
                                    result.push_str(&format!(
                                        "{}{}{}\n",
                                        self.indent(),
                                        else_val,
                                        then_suffix
                                    ));
                                    self.indent_level -= 1;
                                    result.push_str(&format!("{}}}", self.indent()));
                                } else {
                                    result.push_str(" else { () }");
                                }
                                Ok(result)
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
                                            Ok(format!(
                                                "{}.into_iter().map(|{}| {}).collect::<Vec<_>>()",
                                                list, param, body_str
                                            ))
                                        } else {
                                            Err(std::fmt::Error)
                                        }
                                    }
                                    _ => {
                                        let func = self.generate_expression_value(&arguments[0])?;
                                        Ok(format!(
                                            "{}.into_iter().map({}).collect::<Vec<_>>()",
                                            list, func
                                        ))
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
                                        Ok(format!(
                                            "{}.into_iter().filter({}).collect::<Vec<_>>()",
                                            list, func
                                        ))
                                    }
                                }
                            }
                            "LazyMap" => {
                                // LazyMap[function, list] -> list.into_iter().map(|x| function(x))
                                // Returns an iterator, not a collected Vec
                                if arguments.len() != 2 {
                                    return Err(std::fmt::Error);
                                }
                                let list = self.generate_expression_value(&arguments[1])?;
                                match &arguments[0] {
                                    Expression::Lambda { parameters, body } => {
                                        if parameters.len() == 1 {
                                            let param = &to_snake_case(&parameters[0].name);
                                            let body_str = self.generate_expression_value(body)?;
                                            Ok(format!(
                                                "{}.into_iter().map(|{}| {})",
                                                list, param, body_str
                                            ))
                                        } else {
                                            Err(std::fmt::Error)
                                        }
                                    }
                                    _ => {
                                        let func = self.generate_expression_value(&arguments[0])?;
                                        Ok(format!("{}.into_iter().map({})", list, func))
                                    }
                                }
                            }
                            "LazyFilter" => {
                                // LazyFilter[predicate, list] -> list.into_iter().filter(|&x| predicate(x))
                                // Returns an iterator, not a collected Vec
                                if arguments.len() != 2 {
                                    return Err(std::fmt::Error);
                                }
                                let list = self.generate_expression_value(&arguments[1])?;
                                match &arguments[0] {
                                    Expression::Lambda { parameters, body } => {
                                        if parameters.len() == 1 {
                                            let param = &to_snake_case(&parameters[0].name);
                                            let body_str = self.generate_expression_value(body)?;
                                            Ok(format!(
                                                "{}.into_iter().filter(|&{}| {})",
                                                list, param, body_str
                                            ))
                                        } else {
                                            Err(std::fmt::Error)
                                        }
                                    }
                                    _ => {
                                        let func = self.generate_expression_value(&arguments[0])?;
                                        Ok(format!("{}.into_iter().filter({})", list, func))
                                    }
                                }
                            }
                            "Collect" => {
                                // Collect[iter] -> iter.collect::<Vec<_>>()
                                // Collects an iterator into a Vec
                                if arguments.len() != 1 {
                                    return Err(std::fmt::Error);
                                }
                                let iter = self.generate_expression_value(&arguments[0])?;
                                Ok(format!("{}.collect::<Vec<_>>()", iter))
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
                                            Ok(format!(
                                                "{}.into_iter().fold({}, |{}, {}| {})",
                                                list, init, param1, param2, body_str
                                            ))
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
                                    let format_parts: Vec<String> = arguments
                                        .iter()
                                        .map(|arg| {
                                            match arg {
                                                Expression::List(_)
                                                | Expression::Map(_)
                                                | Expression::Tuple(_) => "{:?}".to_string(),
                                                // Also check for Map/Filter function calls that return Vec
                                                Expression::FunctionCall { function, .. } => {
                                                    match function.as_ref() {
                                                        Expression::Identifier(name) => {
                                                            // Check if it returns a Vec type (needs debug format)
                                                            if name == "Map"
                                                                || name == "Filter"
                                                                || name == "Args"
                                                                || name == "Range"
                                                                || name == "Reverse"
                                                                || name == "Concat"
                                                                || name == "Rest"
                                                                || name == "Set"
                                                                || name == "Take"
                                                                || name == "Zip"
                                                                || name == "FlatMap"
                                                                || name == "GroupBy"
                                                                || name == "Collect"
                                                                || name == "Append"
                                                                || name == "ReadFile"
                                                                || name == "WriteFile"
                                                                || name == "StringSplit"
                                                                || self
                                                                    .struct_definitions
                                                                    .contains_key(name)
                                                            {
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
                            "Args" => {
                                // Args[] -> std::env::args().skip(1).collect::<Vec<String>>()
                                Ok("std::env::args().skip(1).collect::<Vec<String>>()".to_string())
                            }
                            "ReadLine" => {
                                // ReadLine[] -> read line from stdin, return Option<String>
                                Ok("{ let mut s = String::new(); match std::io::stdin().read_line(&mut s) { Ok(0) => None, Ok(_) => Some(s.trim().to_string()), Err(_) => None } }".to_string())
                            }
                            "ReadFile" => {
                                // ReadFile[path] -> read entire file as Option<String>
                                // Returns None if the file doesn't exist or can't be read
                                if arguments.len() != 1 {
                                    return Err(std::fmt::Error);
                                }
                                let path = self.generate_expression_value(&arguments[0])?;
                                Ok(format!(
                                    "std::fs::read_to_string({}).ok()",
                                    path
                                ))
                            }
                            "WriteFile" => {
                                // WriteFile[path, contents] -> Result<(), String>
                                // Returns Ok(()) on success, Err(message) on failure
                                if arguments.len() != 2 {
                                    return Err(std::fmt::Error);
                                }
                                let path = self.generate_expression_value(&arguments[0])?;
                                let contents = self.generate_expression_value(&arguments[1])?;
                                Ok(format!(
                                    "std::fs::write({}, {}).map_err(|e| e.to_string())",
                                    path, contents
                                ))
                            }
                            "ToString" => {
                                // ToString[x] -> converts any Display value to String
                                if arguments.len() != 1 {
                                    return Err(std::fmt::Error);
                                }
                                let x = self.generate_expression_value(&arguments[0])?;
                                Ok(format!("({}).to_string()", x))
                            }
                            "StringJoin" => {
                                // StringJoin[list, sep] -> joins List[String] with separator
                                if arguments.len() != 2 {
                                    return Err(std::fmt::Error);
                                }
                                let list = self.generate_expression_value(&arguments[0])?;
                                let sep = self.generate_expression_value(&arguments[1])?;
                                Ok(format!("({}).join({}.as_str())", list, sep))
                            }
                            "StringSplit" => {
                                // StringSplit[text, sep] -> splits String into List[String]
                                if arguments.len() != 2 {
                                    return Err(std::fmt::Error);
                                }
                                let text = self.generate_expression_value(&arguments[0])?;
                                let sep = self.generate_expression_value(&arguments[1])?;
                                Ok(format!(
                                    "({}).split({}.as_str()).map(|s| s.to_string()).collect::<Vec<String>>()",
                                    text, sep
                                ))
                            }
                            "Length" => {
                                // Length[list] -> list.len()
                                if arguments.len() != 1 {
                                    return Err(std::fmt::Error);
                                }
                                let list = self.generate_expression_value(&arguments[0])?;
                                Ok(format!("{}.len()", list))
                            }
                            "Sqrt" => {
                                // Sqrt[x] -> x.sqrt()
                                if arguments.len() != 1 {
                                    return Err(std::fmt::Error);
                                }
                                let x = self.generate_expression_value(&arguments[0])?;
                                Ok(format!("({}).sqrt()", x))
                            }
                            "Sin" => {
                                // Sin[x] -> x.sin()
                                if arguments.len() != 1 {
                                    return Err(std::fmt::Error);
                                }
                                let x = self.generate_expression_value(&arguments[0])?;
                                Ok(format!("({}).sin()", x))
                            }
                            "Cos" => {
                                // Cos[x] -> x.cos()
                                if arguments.len() != 1 {
                                    return Err(std::fmt::Error);
                                }
                                let x = self.generate_expression_value(&arguments[0])?;
                                Ok(format!("({}).cos()", x))
                            }
                            "ParseInt" => {
                                // ParseInt[s] -> s.parse::<i64>().unwrap()
                                if arguments.len() != 1 {
                                    return Err(std::fmt::Error);
                                }
                                let s = self.generate_expression_value(&arguments[0])?;
                                Ok(format!("{}.parse::<i64>().unwrap()", s))
                            }
                            "Range" => {
                                // Range[n] -> (1..=n).collect::<Vec<i64>>()
                                // or Range[start, end] -> (start..=end).collect::<Vec<i64>>()
                                match arguments.len() {
                                    1 => {
                                        let n = self.generate_expression_value(&arguments[0])?;
                                        Ok(format!("(1..={}).collect::<Vec<i64>>()", n))
                                    }
                                    2 => {
                                        let start =
                                            self.generate_expression_value(&arguments[0])?;
                                        let end = self.generate_expression_value(&arguments[1])?;
                                        Ok(format!("({}..={}).collect::<Vec<i64>>()", start, end))
                                    }
                                    _ => Err(std::fmt::Error),
                                }
                            }
                            "Nth" => {
                                // Nth[list, i] -> list[i as usize].clone()
                                // Clone so the element can be used by value without
                                // moving out of a named Vec binding (E0507)
                                if arguments.len() != 2 {
                                    return Err(std::fmt::Error);
                                }
                                let list = self.generate_expression_value(&arguments[0])?;
                                let idx = self.generate_expression_value(&arguments[1])?;
                                Ok(format!("{}[{} as usize].clone()", list, idx))
                            }
                            "Set" => {
                                // Set[list, i, v] -> { let mut l = list.clone(); l[i as usize] = v; l }
                                // Clone the input so the replacement value can still
                                // reference the original list (E0382), and the original
                                // binding is not consumed (functional update semantics)
                                if arguments.len() != 3 {
                                    return Err(std::fmt::Error);
                                }
                                let list = self.generate_expression_value(&arguments[0])?;
                                let idx = self.generate_expression_value(&arguments[1])?;
                                let val = self.generate_expression_value(&arguments[2])?;
                                Ok(format!(
                                    "{{ let mut l = {}.clone(); l[{} as usize] = {}; l }}",
                                    list, idx, val
                                ))
                            }
                            "First" => {
                                // First[list] -> list[0].clone()
                                // Clone so the element can be used by value without
                                // moving out of a named Vec binding (E0507)
                                if arguments.len() != 1 {
                                    return Err(std::fmt::Error);
                                }
                                let list = self.generate_expression_value(&arguments[0])?;
                                Ok(format!("{}[0].clone()", list))
                            }
                            "Rest" => {
                                // Rest[list] -> list[1..].to_vec()
                                if arguments.len() != 1 {
                                    return Err(std::fmt::Error);
                                }
                                let list = self.generate_expression_value(&arguments[0])?;
                                Ok(format!("{}[1..].to_vec()", list))
                            }
                            "Concat" => {
                                // Concat[a, b] -> [a, b].concat() or a.into_iter().chain(b).collect()
                                if arguments.len() != 2 {
                                    return Err(std::fmt::Error);
                                }
                                let a = self.generate_expression_value(&arguments[0])?;
                                let b = self.generate_expression_value(&arguments[1])?;
                                Ok(format!(
                                    "{}.into_iter().chain({}).collect::<Vec<_>>()",
                                    a, b
                                ))
                            }
                            "Reverse" => {
                                // Reverse[list] -> list.into_iter().rev().collect::<Vec<_>>()
                                if arguments.len() != 1 {
                                    return Err(std::fmt::Error);
                                }
                                let list = self.generate_expression_value(&arguments[0])?;
                                Ok(format!("{}.into_iter().rev().collect::<Vec<_>>()", list))
                            }
                            "Append" => {
                                // Append[list, item] -> { let mut l = list; l.push(item); l }
                                if arguments.len() != 2 {
                                    return Err(std::fmt::Error);
                                }
                                let list = self.generate_expression_value(&arguments[0])?;
                                let item = self.generate_expression_value(&arguments[1])?;
                                Ok(format!("{{ let mut l = {}; l.push({}); l }}", list, item))
                            }
                            "Take" => {
                                // Take[n, list] -> list.into_iter().take(n as usize).collect::<Vec<_>>()
                                if arguments.len() != 2 {
                                    return Err(std::fmt::Error);
                                }
                                let n = self.generate_expression_value(&arguments[0])?;
                                let list = self.generate_expression_value(&arguments[1])?;
                                Ok(format!(
                                    "{}.into_iter().take({} as usize).collect::<Vec<_>>()",
                                    list, n
                                ))
                            }
                            "Zip" => {
                                // Zip[list1, list2] -> list1.into_iter().zip(list2.into_iter()).collect::<Vec<_>>()
                                if arguments.len() != 2 {
                                    return Err(std::fmt::Error);
                                }
                                let list1 = self.generate_expression_value(&arguments[0])?;
                                let list2 = self.generate_expression_value(&arguments[1])?;
                                Ok(format!(
                                    "{}.into_iter().zip({}.into_iter()).collect::<Vec<_>>()",
                                    list1, list2
                                ))
                            }
                            "FlatMap" => {
                                // FlatMap[function, list] -> list.into_iter().flat_map(|x| function(x)).collect::<Vec<_>>()
                                if arguments.len() != 2 {
                                    return Err(std::fmt::Error);
                                }
                                let list = self.generate_expression_value(&arguments[1])?;
                                match &arguments[0] {
                                    Expression::Lambda { parameters, body } => {
                                        if parameters.len() == 1 {
                                            let param = &to_snake_case(&parameters[0].name);
                                            let body_str = self.generate_expression_value(body)?;
                                            Ok(format!(
                                                "{}.into_iter().flat_map(|{}| {}).collect::<Vec<_>>()",
                                                list, param, body_str
                                            ))
                                        } else {
                                            Err(std::fmt::Error)
                                        }
                                    }
                                    _ => {
                                        let func = self.generate_expression_value(&arguments[0])?;
                                        Ok(format!(
                                            "{}.into_iter().flat_map({}).collect::<Vec<_>>()",
                                            list, func
                                        ))
                                    }
                                }
                            }
                            "GroupBy" => {
                                // GroupBy[key_fn, list] -> fold into HashMap<K, Vec<T>>
                                if arguments.len() != 2 {
                                    return Err(std::fmt::Error);
                                }
                                let list = self.generate_expression_value(&arguments[1])?;
                                match &arguments[0] {
                                    Expression::Lambda { parameters, body } => {
                                        if parameters.len() == 1 {
                                            let param = &to_snake_case(&parameters[0].name);
                                            let body_str = self.generate_expression_value(body)?;
                                            Ok(format!(
                                                "{}.into_iter().fold(std::collections::HashMap::new(), |mut acc, {}| {{ let key = {}; acc.entry(key).or_insert_with(Vec::new).push({}); acc }})",
                                                list, param, body_str, param
                                            ))
                                        } else {
                                            Err(std::fmt::Error)
                                        }
                                    }
                                    _ => {
                                        let func = self.generate_expression_value(&arguments[0])?;
                                        Ok(format!(
                                            "{}.into_iter().fold(std::collections::HashMap::new(), |mut acc, x| {{ let key = {}(x); acc.entry(key).or_insert_with(Vec::new).push(x); acc }})",
                                            list, func
                                        ))
                                    }
                                }
                            }
                            _ => {
                                // Check if this is a struct constructor
                                if let Some(field_names) =
                                    self.struct_definitions.get(name).cloned()
                                {
                                    // Generate struct instantiation: StructName { field1: value1, field2: value2 }
                                    if field_names.len() != arguments.len() {
                                        return Err(std::fmt::Error);
                                    }

                                    let mut result = format!("{} {{ ", name);
                                    for (i, (field_name, arg)) in
                                        field_names.iter().zip(arguments.iter()).enumerate()
                                    {
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

            Expression::Cond {
                conditions,
                default_statements,
            } => {
                let mut result = String::new();

                // If Cond arms mix a definite-unit side-effect (Print/Break) with a
                // definite value (WriteFile -> Result), append ";" after every arm
                // so all arms normalize to unit. Unknown arms (With-bound or
                // unresolved identifiers) never drive normalization, and
                // all-value Conds (benchmark CountNodes) keep their values.
                let mut has_unit = false;
                let mut has_value = false;
                for (_, stmts) in conditions {
                    match self.classify_arm(stmts) {
                        ArmKind::Unit => has_unit = true,
                        ArmKind::Value => has_value = true,
                        ArmKind::Unknown => {}
                    }
                }
                if let Some(d) = &default_statements {
                    match self.classify_arm(d) {
                        ArmKind::Unit => has_unit = true,
                        ArmKind::Value => has_value = true,
                        ArmKind::Unknown => {}
                    }
                }
                let is_unit = has_unit && has_value;

                for (i, (condition, statements)) in conditions.iter().enumerate() {
                    if i > 0 {
                        result.push_str(" else ");
                    }

                    let cond_val = self.generate_expression_value(condition)?;

                    if cond_val == "true" {
                        result.push_str("{\n");
                        self.indent_level += 1;
                        let stmt_val = self.generate_expression_value(statements)?;
                        if is_unit {
                            write!(&mut result, "{}{};\n", self.indent(), stmt_val)?;
                        } else {
                            write!(&mut result, "{}{}\n", self.indent(), stmt_val)?;
                        }
                        self.indent_level -= 1;
                        write!(&mut result, "{}}}", self.indent())?;
                    } else {
                        write!(&mut result, "if {} {{\n", cond_val)?;
                        self.indent_level += 1;
                        let stmt_val = self.generate_expression_value(statements)?;
                        if is_unit {
                            write!(&mut result, "{}{};\n", self.indent(), stmt_val)?;
                        } else {
                            write!(&mut result, "{}{}\n", self.indent(), stmt_val)?;
                        }
                        self.indent_level -= 1;
                        write!(&mut result, "{}}}", self.indent())?;
                    }
                }

                if let Some(default_expr) = default_statements {
                    write!(&mut result, " else {{\n")?;
                    self.indent_level += 1;
                    let default_val = self.generate_expression_value(default_expr)?;
                    if is_unit {
                        write!(&mut result, "{}{};\n", self.indent(), default_val)?;
                    } else {
                        write!(&mut result, "{}{}\n", self.indent(), default_val)?;
                    }
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

            Expression::Propagate { expr } => {
                let inner = self.generate_expression_value(expr)?;
                Ok(format!("({})?", inner))
            }

            Expression::StructInstantiation {
                struct_name,
                field_values,
            } => {
                // Generate: StructName { field1: value1, field2: value2 }
                // Look up the field names from the struct definition
                let field_names = self
                    .struct_definitions
                    .get(struct_name)
                    .cloned()
                    .ok_or(std::fmt::Error)?;

                if field_names.len() != field_values.len() {
                    // Mismatch between number of fields and values
                    return Err(std::fmt::Error);
                }

                let mut result = format!("{} {{ ", struct_name);

                // Generate field: value pairs
                for (i, (field_name, value)) in
                    field_names.iter().zip(field_values.iter()).enumerate()
                {
                    if i > 0 {
                        result.push_str(", ");
                    }
                    let value_str = self.generate_expression_value(value)?;
                    result.push_str(&format!("{}: {}", field_name, value_str));
                }

                result.push_str(" }");
                Ok(result)
            }

            Expression::With { bindings, body } => {
                // Generate Rust block expression:
                // {
                //   let x = expr1;
                //   let y = expr2;
                //   body_expr
                // }
                let mut result = String::from("{\n");
                self.indent_level += 1;

                // Generate let statements for each binding
                for (name, expr) in bindings {
                    let name_snake = to_snake_case(name);
                    let expr_str = self.generate_expression_value(expr)?;
                    result.push_str(&format!(
                        "{}let {} = {};\n",
                        self.indent(),
                        name_snake,
                        expr_str
                    ));
                }

                // Generate body expression (without semicolon, as it's a value)
                let body_str = self.generate_expression_value(body)?;
                result.push_str(&format!("{}{}\n", self.indent(), body_str));

                self.indent_level -= 1;
                result.push_str(&format!("{}}}", self.indent()));
                Ok(result)
            }

            Expression::Module { bindings, body } => {
                let mut result = String::from("{\n");
                self.indent_level += 1;

                for (name, init_expr) in bindings {
                    let name_snake = to_snake_case(name);
                    if let Some(expr) = init_expr {
                        let expr_str = self.generate_expression_value(expr)?;
                        result.push_str(&format!(
                            "{}let mut {} = {};\n",
                            self.indent(),
                            name_snake,
                            expr_str
                        ));
                    } else {
                        result.push_str(&format!("{}let mut {}: _;\n", self.indent(), name_snake));
                    }
                }

                let body_str = self.generate_expression_value(body)?;
                result.push_str(&format!("{}{}\n", self.indent(), body_str));

                self.indent_level -= 1;
                result.push_str(&format!("{}}}", self.indent()));
                Ok(result)
            }

            Expression::Do {
                body,
                var,
                start,
                end,
                step,
            } => {
                // Generate Rust for loop
                // Do[body, {n}] -> for _ in 1..=n { body }
                // Do[body, {i, imax}] -> for i in 1..=imax { body }
                // Do[body, {i, imin, imax}] -> for i in imin..=imax { body }
                // Do[body, {i, imin, imax, di}] -> for i in (imin..=imax).step_by(di) { body }

                let mut result = String::from("{\n");
                self.indent_level += 1;

                let end_str = self.generate_expression_value(end)?;

                match (var, start, step) {
                    (None, None, None) => {
                        // Do[body, {n}] - no loop variable
                        result.push_str(&format!("{}for _ in 1..={} {{\n", self.indent(), end_str));
                    }
                    (Some(var_name), None, None) => {
                        // Do[body, {i, imax}] - start from 1
                        let var_snake = to_snake_case(var_name);
                        result.push_str(&format!(
                            "{}for {} in 1..={} {{\n",
                            self.indent(),
                            var_snake,
                            end_str
                        ));
                    }
                    (Some(var_name), Some(start_expr), None) => {
                        // Do[body, {i, imin, imax}] - explicit start
                        let var_snake = to_snake_case(var_name);
                        let start_str = self.generate_expression_value(start_expr)?;
                        result.push_str(&format!(
                            "{}for {} in {}..={} {{\n",
                            self.indent(),
                            var_snake,
                            start_str,
                            end_str
                        ));
                    }
                    (Some(var_name), Some(start_expr), Some(step_expr)) => {
                        // Do[body, {i, imin, imax, di}] - with step
                        let var_snake = to_snake_case(var_name);
                        let start_str = self.generate_expression_value(start_expr)?;
                        let step_str = self.generate_expression_value(step_expr)?;
                        result.push_str(&format!(
                            "{}for {} in ({}..={}).step_by({} as usize) {{\n",
                            self.indent(),
                            var_snake,
                            start_str,
                            end_str,
                            step_str
                        ));
                    }
                    _ => return Err(std::fmt::Error),
                }

                self.indent_level += 1;
                let body_str = self.generate_expression_value(body)?;
                result.push_str(&format!("{}{};\n", self.indent(), body_str));
                self.indent_level -= 1;

                result.push_str(&format!("{}}}\n", self.indent()));

                self.indent_level -= 1;
                result.push_str(&format!("{}}}", self.indent()));
                Ok(result)
            }

            Expression::While { condition, body } => {
                // Generate Rust while loop
                // While[condition, body] -> while condition { body }

                let mut result = String::from("{\n");
                self.indent_level += 1;

                let cond_str = self.generate_expression_value(condition)?;
                result.push_str(&format!("{}while {} {{\n", self.indent(), cond_str));

                self.indent_level += 1;
                let body_str = self.generate_expression_value(body)?;
                result.push_str(&format!("{}{};\n", self.indent(), body_str));
                self.indent_level -= 1;

                result.push_str(&format!("{}}}\n", self.indent()));

                self.indent_level -= 1;
                result.push_str(&format!("{}}}", self.indent()));
                Ok(result)
            }

            Expression::Break => Ok("break".to_string()),

            Expression::Continue => Ok("continue".to_string()),
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

    // Avoid Rust reserved keywords by appending underscore
    let reserved_keywords = [
        "as", "break", "const", "continue", "crate", "else", "enum", "extern", "false", "fn",
        "for", "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut", "pub", "ref",
        "return", "self", "Self", "static", "struct", "super", "trait", "true", "type", "unsafe",
        "use", "where", "while", "async", "await", "dyn",
    ];

    if reserved_keywords.contains(&result.as_str()) {
        result.push('_');
    }

    result
}
