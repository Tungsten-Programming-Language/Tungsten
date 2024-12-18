#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

#[repr(u8)]
#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum Type {
    Int = 0,
    Float = 1,
    String = 2,
    Bool = 3,
    List(Box<Type>) = 4,
    Map(Box<Type>, Box<Type>) = 5,
    Function(Vec<Type>, Box<Type>) = 6,
    LogLevel = 7,
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
    },
    LogCall {
        level: LogLevel,
        message: Box<Expression>,
    },
    /// Conditional expression similar to LISP's `cond`
    /// 
    /// Structure: `Cond[[condition1 statements1] [condition2 statements2] ... [default_statements]]`
    /// 
    /// # Variants
    /// - `conditions`: A list of condition-statement pairs
    /// - `default_statements`: Optional statements to execute if no conditions match
    Cond {
        conditions: Vec<(Expression, Expression)>,
        default_statements: Option<Box<Expression>>,
    }
}

#[derive(Debug)]
#[allow(dead_code)]
pub enum Operator {
    Add = 1,
    Subtract = 2,
    Multiply = 3,
    Divide = 4,
    Power = 5,
    Equals = 6,
    NotEquals = 7,
    LessThan = 8,
    GreaterThan = 9,
}
