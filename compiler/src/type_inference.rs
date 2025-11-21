//! Type Inference Module
//!
//! Performs type inference and type checking on the W language AST.
//! This runs after parsing and before code generation.

use crate::ast::{Expression, Type, TypeAnnotation, Operator, Pattern};
use std::collections::HashMap;
use std::fmt;

/// Type inference errors
#[derive(Debug, Clone, PartialEq)]
pub enum TypeError {
    /// Type mismatch between expected and actual types
    TypeMismatch {
        expected: Type,
        actual: Type,
        context: String,
    },
    /// Undefined variable or function
    UndefinedIdentifier(String),
    /// Wrong number of arguments in function call
    ArityMismatch {
        function: String,
        expected: usize,
        actual: usize,
    },
    /// Cannot infer type (insufficient information)
    CannotInfer(String),
    /// Struct not defined
    UndefinedStruct(String),
    /// Field count mismatch in struct instantiation
    FieldCountMismatch {
        struct_name: String,
        expected: usize,
        actual: usize,
    },
}

impl fmt::Display for TypeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TypeError::TypeMismatch { expected, actual, context } => {
                write!(f, "Type mismatch in {}: expected {:?}, got {:?}", context, expected, actual)
            }
            TypeError::UndefinedIdentifier(name) => {
                write!(f, "Undefined identifier: {}", name)
            }
            TypeError::ArityMismatch { function, expected, actual } => {
                write!(f, "Function {} expects {} arguments, got {}", function, expected, actual)
            }
            TypeError::CannotInfer(context) => {
                write!(f, "Cannot infer type for: {}", context)
            }
            TypeError::UndefinedStruct(name) => {
                write!(f, "Undefined struct: {}", name)
            }
            TypeError::FieldCountMismatch { struct_name, expected, actual } => {
                write!(f, "Struct {} expects {} fields, got {}", struct_name, expected, actual)
            }
        }
    }
}

/// Type environment tracks variable and function types
#[derive(Debug, Clone)]
pub struct TypeEnvironment {
    /// Maps variable/function names to their types
    bindings: HashMap<String, Type>,
    /// Maps struct names to their field types
    structs: HashMap<String, Vec<TypeAnnotation>>,
}

impl TypeEnvironment {
    pub fn new() -> Self {
        TypeEnvironment {
            bindings: HashMap::new(),
            structs: HashMap::new(),
        }
    }

    /// Add a variable or function binding
    pub fn bind(&mut self, name: String, ty: Type) {
        self.bindings.insert(name, ty);
    }

    /// Look up a variable or function type
    pub fn lookup(&self, name: &str) -> Option<&Type> {
        self.bindings.get(name)
    }

    /// Add a struct definition
    pub fn define_struct(&mut self, name: String, fields: Vec<TypeAnnotation>) {
        self.structs.insert(name, fields);
    }

    /// Look up a struct definition
    pub fn lookup_struct(&self, name: &str) -> Option<&Vec<TypeAnnotation>> {
        self.structs.get(name)
    }

    /// Create a child environment (for nested scopes)
    pub fn child(&self) -> Self {
        TypeEnvironment {
            bindings: self.bindings.clone(),
            structs: self.structs.clone(),
        }
    }
}

/// Type inference engine
pub struct TypeInference {
    env: TypeEnvironment,
}

impl TypeInference {
    pub fn new() -> Self {
        TypeInference {
            env: TypeEnvironment::new(),
        }
    }

    /// Infer the type of an expression
    pub fn infer_expression(&mut self, expr: &Expression) -> Result<Type, TypeError> {
        match expr {
            // Literals have known types
            Expression::Number(_) => Ok(Type::Int32),
            Expression::Float(_) => Ok(Type::Float64),
            Expression::String(_) => Ok(Type::String),
            Expression::Boolean(_) => Ok(Type::Bool),

            // Tuples
            Expression::Tuple(elements) => {
                let mut types = Vec::new();
                for elem in elements {
                    types.push(self.infer_expression(elem)?);
                }
                Ok(Type::Tuple(types))
            }

            // Lists
            Expression::List(elements) => {
                if elements.is_empty() {
                    // Empty list - cannot infer type without context
                    return Err(TypeError::CannotInfer("empty list".to_string()));
                }
                // Infer from first element (all elements should have same type)
                let first_type = self.infer_expression(&elements[0])?;
                // Check that all elements have the same type
                for elem in &elements[1..] {
                    let elem_type = self.infer_expression(elem)?;
                    if elem_type != first_type {
                        return Err(TypeError::TypeMismatch {
                            expected: first_type.clone(),
                            actual: elem_type,
                            context: "list elements".to_string(),
                        });
                    }
                }
                Ok(Type::List(Box::new(first_type)))
            }

            // Identifiers look up in environment
            Expression::Identifier(name) => {
                self.env.lookup(name)
                    .cloned()
                    .ok_or_else(|| TypeError::UndefinedIdentifier(name.clone()))
            }

            // Binary operations
            Expression::BinaryOp { left, operator, right } => {
                let left_type = self.infer_expression(left)?;
                let right_type = self.infer_expression(right)?;

                match operator {
                    // Arithmetic operations
                    Operator::Add | Operator::Subtract | Operator::Multiply | Operator::Divide | Operator::Power => {
                        // Both operands should be numeric and same type
                        if !is_numeric(&left_type) {
                            return Err(TypeError::TypeMismatch {
                                expected: Type::Int32,
                                actual: left_type,
                                context: "arithmetic operation".to_string(),
                            });
                        }
                        if left_type != right_type {
                            return Err(TypeError::TypeMismatch {
                                expected: left_type.clone(),
                                actual: right_type,
                                context: "arithmetic operation".to_string(),
                            });
                        }
                        Ok(left_type)
                    }

                    // Comparison operations return bool
                    Operator::Equals | Operator::NotEquals | Operator::LessThan | Operator::GreaterThan => {
                        // Both operands should have the same type
                        if left_type != right_type {
                            return Err(TypeError::TypeMismatch {
                                expected: left_type.clone(),
                                actual: right_type,
                                context: "comparison operation".to_string(),
                            });
                        }
                        Ok(Type::Bool)
                    }
                }
            }

            // Function definitions
            Expression::FunctionDefinition { name, parameters, body } => {
                // Create child environment with parameters
                let mut child_env = self.env.child();
                for param in parameters {
                    child_env.bind(param.name.clone(), param.type_.clone());
                }

                // Infer return type from body
                let mut child_inference = TypeInference { env: child_env };
                let return_type = child_inference.infer_expression(body)?;

                // Create function type
                let param_types: Vec<Type> = parameters.iter().map(|p| p.type_.clone()).collect();
                let func_type = Type::Function(param_types, Box::new(return_type));

                // Bind function in environment
                self.env.bind(name.clone(), func_type.clone());

                Ok(func_type)
            }

            // Function calls
            Expression::FunctionCall { function, arguments } => {
                match function.as_ref() {
                    Expression::Identifier(name) => {
                        // Check for built-in functions
                        match name.as_str() {
                            "Print" => Ok(Type::Tuple(vec![])), // Unit type ()
                            "Tuple" => {
                                let mut types = Vec::new();
                                for arg in arguments {
                                    types.push(self.infer_expression(arg)?);
                                }
                                Ok(Type::Tuple(types))
                            }
                            "Map" | "Filter" => {
                                // Map and Filter return lists
                                // TODO: Infer element type from lambda
                                if arguments.len() != 2 {
                                    return Err(TypeError::ArityMismatch {
                                        function: name.clone(),
                                        expected: 2,
                                        actual: arguments.len(),
                                    });
                                }
                                // For now, return List of unknown type
                                Ok(Type::List(Box::new(Type::Int32)))
                            }
                            "Fold" => {
                                // Fold returns the accumulator type
                                if arguments.len() != 3 {
                                    return Err(TypeError::ArityMismatch {
                                        function: name.clone(),
                                        expected: 3,
                                        actual: arguments.len(),
                                    });
                                }
                                // Return type is the type of the initial value
                                self.infer_expression(&arguments[1])
                            }
                            _ => {
                                // Check if it's a struct constructor
                                if let Some(fields) = self.env.lookup_struct(name).cloned() {
                                    if fields.len() != arguments.len() {
                                        return Err(TypeError::FieldCountMismatch {
                                            struct_name: name.clone(),
                                            expected: fields.len(),
                                            actual: arguments.len(),
                                        });
                                    }
                                    // Check argument types match field types
                                    for (arg, field) in arguments.iter().zip(fields.iter()) {
                                        let arg_type = self.infer_expression(arg)?;
                                        if arg_type != field.type_ {
                                            return Err(TypeError::TypeMismatch {
                                                expected: field.type_.clone(),
                                                actual: arg_type,
                                                context: format!("field {}", field.name),
                                            });
                                        }
                                    }
                                    return Ok(Type::Custom(name.clone()));
                                }

                                // Look up user-defined function
                                if let Some(func_type) = self.env.lookup(name).cloned() {
                                    match func_type {
                                        Type::Function(param_types, return_type) => {
                                            if param_types.len() != arguments.len() {
                                                return Err(TypeError::ArityMismatch {
                                                    function: name.clone(),
                                                    expected: param_types.len(),
                                                    actual: arguments.len(),
                                                });
                                            }
                                            // Check argument types
                                            for (arg, expected_type) in arguments.iter().zip(param_types.iter()) {
                                                let arg_type = self.infer_expression(arg)?;
                                                if &arg_type != expected_type {
                                                    return Err(TypeError::TypeMismatch {
                                                        expected: expected_type.clone(),
                                                        actual: arg_type,
                                                        context: format!("argument to {}", name),
                                                    });
                                                }
                                            }
                                            Ok((*return_type).clone())
                                        }
                                        _ => Err(TypeError::TypeMismatch {
                                            expected: Type::Function(vec![], Box::new(Type::Int32)),
                                            actual: func_type.clone(),
                                            context: format!("{} is not a function", name),
                                        }),
                                    }
                                } else {
                                    Err(TypeError::UndefinedIdentifier(name.clone()))
                                }
                            }
                        }
                    }
                    _ => Err(TypeError::CannotInfer("complex function expression".to_string())),
                }
            }

            // Struct definitions
            Expression::StructDefinition { name, fields } => {
                self.env.define_struct(name.clone(), fields.clone());
                Ok(Type::Tuple(vec![])) // Struct definitions return unit type
            }

            // Other expressions
            Expression::None => Ok(Type::Option(Box::new(Type::Int32))), // TODO: Better inference
            Expression::Some { value } => {
                let inner_type = self.infer_expression(value)?;
                Ok(Type::Option(Box::new(inner_type)))
            }
            Expression::Ok { value } => {
                let ok_type = self.infer_expression(value)?;
                Ok(Type::Result(Box::new(ok_type), Box::new(Type::String)))
            }
            Expression::Err { error } => {
                let err_type = self.infer_expression(error)?;
                Ok(Type::Result(Box::new(Type::Int32), Box::new(err_type)))
            }

            // Match expression with pattern matching
            Expression::Match { value, arms } => {
                // Infer the type of the value being matched
                let value_type = self.infer_expression(value)?;

                if arms.is_empty() {
                    return Err(TypeError::CannotInfer("match with no arms".to_string()));
                }

                // Check each arm and collect result types
                let mut result_type: Option<Type> = None;

                for (pattern, result_expr) in arms {
                    // Create child environment for pattern bindings
                    let mut child_env = self.env.child();

                    // Check pattern against value type and collect bindings
                    self.check_pattern(pattern, &value_type, &mut child_env)?;

                    // Infer result type in the child environment
                    let mut child_inference = TypeInference { env: child_env };
                    let arm_result_type = child_inference.infer_expression(result_expr)?;

                    // Ensure all arms return the same type
                    match &result_type {
                        None => result_type = Some(arm_result_type),
                        Some(expected) => {
                            if expected != &arm_result_type {
                                return Err(TypeError::TypeMismatch {
                                    expected: expected.clone(),
                                    actual: arm_result_type,
                                    context: "match arm result".to_string(),
                                });
                            }
                        }
                    }
                }

                Ok(result_type.unwrap())
            }

            // Conditional expression
            Expression::Cond { conditions, default_statements } => {
                let mut result_type: Option<Type> = None;

                // Check each condition
                for (condition, statements) in conditions {
                    let cond_type = self.infer_expression(condition)?;
                    if cond_type != Type::Bool {
                        return Err(TypeError::TypeMismatch {
                            expected: Type::Bool,
                            actual: cond_type,
                            context: "condition".to_string(),
                        });
                    }

                    let stmt_type = self.infer_expression(statements)?;
                    match &result_type {
                        None => result_type = Some(stmt_type),
                        Some(expected) => {
                            if expected != &stmt_type {
                                return Err(TypeError::TypeMismatch {
                                    expected: expected.clone(),
                                    actual: stmt_type,
                                    context: "cond branch".to_string(),
                                });
                            }
                        }
                    }
                }

                // Check default branch if present
                if let Some(default) = default_statements {
                    let default_type = self.infer_expression(default)?;
                    match &result_type {
                        None => result_type = Some(default_type),
                        Some(expected) => {
                            if expected != &default_type {
                                return Err(TypeError::TypeMismatch {
                                    expected: expected.clone(),
                                    actual: default_type,
                                    context: "cond default branch".to_string(),
                                });
                            }
                        }
                    }
                }

                Ok(result_type.unwrap_or(Type::Tuple(vec![])))
            }

            // Not yet implemented
            Expression::Program(_) => Err(TypeError::CannotInfer("program".to_string())),
            Expression::Lambda { .. } => Err(TypeError::CannotInfer("lambda".to_string())),
            Expression::LogCall { .. } => Ok(Type::Tuple(vec![])),
            Expression::Map(_) => Err(TypeError::CannotInfer("map literal".to_string())),
            Expression::StructInstantiation { .. } => Err(TypeError::CannotInfer("struct instantiation".to_string())),
        }
    }

    /// Check that a pattern matches the expected type and collect variable bindings
    fn check_pattern(
        &self,
        pattern: &Pattern,
        expected_type: &Type,
        env: &mut TypeEnvironment,
    ) -> Result<(), TypeError> {
        match pattern {
            // Wildcard matches anything
            Pattern::Wildcard => Ok(()),

            // Literal patterns must match exactly
            Pattern::Literal(expr) => {
                // Create a temporary inference context to check the literal
                let mut temp_inference = TypeInference { env: self.env.clone() };
                let literal_type = temp_inference.infer_expression(expr)?;

                if &literal_type != expected_type {
                    return Err(TypeError::TypeMismatch {
                        expected: expected_type.clone(),
                        actual: literal_type,
                        context: "pattern literal".to_string(),
                    });
                }
                Ok(())
            }

            // Variable patterns bind to the expected type
            Pattern::Variable(name) => {
                env.bind(name.clone(), expected_type.clone());
                Ok(())
            }

            // Constructor patterns (Some, Ok, Err, None)
            Pattern::Constructor { name, patterns } => {
                match name.as_str() {
                    "Some" => {
                        match expected_type {
                            Type::Option(inner_type) => {
                                if patterns.len() != 1 {
                                    return Err(TypeError::CannotInfer(
                                        "Some pattern must have exactly one argument".to_string()
                                    ));
                                }
                                self.check_pattern(&patterns[0], inner_type, env)
                            }
                            _ => Err(TypeError::TypeMismatch {
                                expected: Type::Option(Box::new(Type::Int32)),
                                actual: expected_type.clone(),
                                context: "Some pattern".to_string(),
                            }),
                        }
                    }
                    "None" => {
                        match expected_type {
                            Type::Option(_) => {
                                if !patterns.is_empty() {
                                    return Err(TypeError::CannotInfer(
                                        "None pattern should have no arguments".to_string()
                                    ));
                                }
                                Ok(())
                            }
                            _ => Err(TypeError::TypeMismatch {
                                expected: Type::Option(Box::new(Type::Int32)),
                                actual: expected_type.clone(),
                                context: "None pattern".to_string(),
                            }),
                        }
                    }
                    "Ok" => {
                        match expected_type {
                            Type::Result(ok_type, _) => {
                                if patterns.len() != 1 {
                                    return Err(TypeError::CannotInfer(
                                        "Ok pattern must have exactly one argument".to_string()
                                    ));
                                }
                                self.check_pattern(&patterns[0], ok_type, env)
                            }
                            _ => Err(TypeError::TypeMismatch {
                                expected: Type::Result(Box::new(Type::Int32), Box::new(Type::String)),
                                actual: expected_type.clone(),
                                context: "Ok pattern".to_string(),
                            }),
                        }
                    }
                    "Err" => {
                        match expected_type {
                            Type::Result(_, err_type) => {
                                if patterns.len() != 1 {
                                    return Err(TypeError::CannotInfer(
                                        "Err pattern must have exactly one argument".to_string()
                                    ));
                                }
                                self.check_pattern(&patterns[0], err_type, env)
                            }
                            _ => Err(TypeError::TypeMismatch {
                                expected: Type::Result(Box::new(Type::Int32), Box::new(Type::String)),
                                actual: expected_type.clone(),
                                context: "Err pattern".to_string(),
                            }),
                        }
                    }
                    _ => Err(TypeError::CannotInfer(format!("Unknown constructor: {}", name))),
                }
            }

            // Tuple patterns
            Pattern::Tuple(patterns) => {
                match expected_type {
                    Type::Tuple(types) => {
                        if patterns.len() != types.len() {
                            return Err(TypeError::TypeMismatch {
                                expected: expected_type.clone(),
                                actual: Type::Tuple(vec![]), // Placeholder
                                context: format!(
                                    "tuple pattern length mismatch: expected {}, got {}",
                                    types.len(),
                                    patterns.len()
                                ),
                            });
                        }

                        for (pattern, ty) in patterns.iter().zip(types.iter()) {
                            self.check_pattern(pattern, ty, env)?;
                        }
                        Ok(())
                    }
                    _ => Err(TypeError::TypeMismatch {
                        expected: Type::Tuple(vec![]),
                        actual: expected_type.clone(),
                        context: "tuple pattern".to_string(),
                    }),
                }
            }

            // List patterns
            Pattern::List(patterns) => {
                match expected_type {
                    Type::List(element_type) => {
                        // All patterns in the list must match the element type
                        for pattern in patterns {
                            self.check_pattern(pattern, element_type, env)?;
                        }
                        Ok(())
                    }
                    _ => Err(TypeError::TypeMismatch {
                        expected: Type::List(Box::new(Type::Int32)),
                        actual: expected_type.clone(),
                        context: "list pattern".to_string(),
                    }),
                }
            }
        }
    }

    /// Type check a program (multiple expressions)
    pub fn check_program(&mut self, expressions: &[Expression]) -> Result<(), TypeError> {
        for expr in expressions {
            self.infer_expression(expr)?;
        }
        Ok(())
    }
}

/// Check if a type is numeric
fn is_numeric(ty: &Type) -> bool {
    matches!(ty,
        Type::Int8 | Type::Int16 | Type::Int32 | Type::Int64 | Type::Int128 | Type::Int |
        Type::UInt8 | Type::UInt16 | Type::UInt32 | Type::UInt64 | Type::UInt128 | Type::UInt |
        Type::Float32 | Type::Float64
    )
}
