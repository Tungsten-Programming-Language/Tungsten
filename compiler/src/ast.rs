#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Int,
    Float,
    String,
    Bool,
    List(Box<Type>),
    Map(Box<Type>, Box<Type>),
    Function(Vec<Type>, Box<Type>),
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct TypeAnnotation {
    pub name: String,
    pub type_: Type,
}

#[allow(dead_code)]
#[derive(Debug)]
pub enum Expression {
    Number(i64),
    Float(f64),
    String(String),
    Boolean(bool),
    List(Vec<Expression>),
    Map(Vec<(Expression, Expression)>),
    Identifier(String),
    FunctionCall {
        function: Box<Expression>,
        arguments: Vec<Expression>,
    },
    FunctionDefinition {
        name: String,
        parameters: Vec<TypeAnnotation>,
        body: Box<Expression>,
    },
    BinaryOp {
        left: Box<Expression>,
        operator: Operator,
        right: Box<Expression>,
    }
}

#[derive(Debug)]
pub enum Operator {
    Add,
    Subtract,
    Multiply,
    Divide,
    Power,
    Equals,
    NotEquals,
    LessThan,
    GreaterThan,
}
