use crate::ast::{Expression, Type, Operator};

#[derive(Debug)]
pub enum TypeError {
    TypeMismatch {
        expected: Type,
        found: Type,
    },
    FunctionCallTypeMismatch {
        function_type: Type,
        arguments: Vec<Type>,
    },
    ListTypeInconsistency {
        expected: Type,
        found: Type,
    },
    UnknownIdentifier(String),
}

pub struct TypeChecker {
    // Future: Add symbol table or context if needed
}

impl TypeChecker {
    pub fn new() -> Self {
        TypeChecker {}
    }

    pub fn check(&mut self, expr: &Expression) -> Result<Type, TypeError> {
        match expr {
            Expression::Number(_) => Ok(Type::Int),
            Expression::Float(_) => Ok(Type::Float),
            Expression::String(_) => Ok(Type::String),
            Expression::Boolean(_) => Ok(Type::Bool),
            Expression::Identifier(_) => todo!("Implement identifier type resolution"),
            Expression::List(elements) => self.check_list_type(elements),
            Expression::Map(entries) => self.check_map_type(entries),
            Expression::FunctionCall { function, arguments } => 
                self.check_function_call(function, arguments),
            Expression::FunctionDefinition { name: _, parameters: _, body } => 
                self.check(body),
            Expression::BinaryOp { left, operator, right } => 
                self.check_binary_operation(left, *operator, right),
            Expression::LogCall { level: _, message } => {
                // Log function always returns void/unit type
                self.check(message)?;
                Ok(Type::Int) // Placeholder, could be a specific "Void" type
            }
        }
    }

    fn check_list_type(&mut self, elements: &[Expression]) -> Result<Type, TypeError> {
        if elements.is_empty() {
            // Default list type is list of integers
            return Ok(Type::List(Box::new(Type::Int)));
        }

        let first_type = self.check(&elements[0])?;

        for elem in &elements[1..] {
            let elem_type = self.check(elem)?;
            if elem_type != first_type {
                return Err(TypeError::ListTypeInconsistency { 
                    expected: first_type, 
                    found: elem_type 
                });
            }
        }

        Ok(Type::List(Box::new(first_type)))
    }

    fn check_map_type(&mut self, entries: &[(Expression, Expression)]) -> Result<Type, TypeError> {
        if entries.is_empty() {
            // Default map type with int key and int value
            return Ok(Type::Map(Box::new(Type::Int), Box::new(Type::Int)));
        }

        let (first_key, first_value) = &entries[0];
        let key_type = self.check(first_key)?;
        let value_type = self.check(first_value)?;

        for (key, value) in &entries[1..] {
            let current_key_type = self.check(key)?;
            let current_value_type = self.check(value)?;

            if current_key_type != key_type || current_value_type != value_type {
                return Err(TypeError::ListTypeInconsistency { 
                    expected: key_type.clone(), 
                    found: current_key_type 
                });
            }
        }

        Ok(Type::Map(Box::new(key_type), Box::new(value_type)))
    }

    fn check_function_call(
        &mut self, 
        function: &Expression, 
        arguments: &[Expression]
    ) -> Result<Type, TypeError> {
        let function_type = self.check(function)?;

        match function_type {
            Type::Function(param_types, return_type) => {
                // Check argument types match parameter types
                if arguments.len() != param_types.len() {
                    return Err(TypeError::FunctionCallTypeMismatch { 
                        function_type, 
                        arguments: arguments.iter().map(|a| self.check(a).unwrap()).collect() 
                    });
                }

                for (arg, expected_type) in arguments.iter().zip(param_types.iter()) {
                    let arg_type = self.check(arg)?;
                    if *arg_type != *expected_type {
                        return Err(TypeError::FunctionCallTypeMismatch { 
                            function_type, 
                            arguments: arguments.iter().map(|a| self.check(a).unwrap()).collect() 
                        });
                    }
                }

                Ok(*return_type)
            }
            _ => Err(TypeError::FunctionCallTypeMismatch { 
                function_type, 
                arguments: arguments.iter().map(|a| self.check(a).unwrap()).collect() 
            }),
        }
    }

    fn check_binary_operation(
        &mut self, 
        left: &Expression, 
        operator: Operator, 
        right: &Expression
    ) -> Result<Type, TypeError> {
        let left_type = self.check(left)?;
        let right_type = self.check(right)?;

        // Ensure types match for binary operations
        match operator {
            Operator::Add | Operator::Subtract | Operator::Multiply | Operator::Divide | Operator::Power => {
                if left_type == Type::Int && right_type == Type::Int {
                    Ok(Type::Int)
                } else if left_type == Type::Float && right_type == Type::Float {
                    Ok(Type::Float)
                } else {
                    Err(TypeError::TypeMismatch { 
                        expected: left_type.clone(), 
                        found: right_type 
                    })
                }
            }
            Operator::Equals | Operator::NotEquals => {
                if left_type == right_type {
                    Ok(Type::Bool)
                } else {
                    Err(TypeError::TypeMismatch { 
                        expected: left_type.clone(), 
                        found: right_type 
                    })
                }
            }
            Operator::LessThan | Operator::GreaterThan => {
                if (left_type == Type::Int || left_type == Type::Float) && 
                   (right_type == Type::Int || right_type == Type::Float) {
                    Ok(Type::Bool)
                } else {
                    Err(TypeError::TypeMismatch { 
                        expected: left_type.clone(), 
                        found: right_type 
                    })
                }
            }
        }
    }
}
