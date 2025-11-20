#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum Type {
    // Signed integers
    Int8,
    Int16,
    Int32,
    Int64,
    Int128,
    Int,  // isize

    // Unsigned integers
    UInt8,
    UInt16,
    UInt32,
    UInt64,
    UInt128,
    UInt,  // usize

    // Floating point
    Float32,
    Float64,

    // Other primitives
    Bool,
    Char,
    String,

    // Composite types
    Tuple(Vec<Type>),                     // (T1, T2, T3, ...)

    // Complex types
    List(Box<Type>),                      // Vec<T>
    Array(Box<Type>, usize),              // [T; N] - fixed size
    Slice(Box<Type>),                     // &[T]
    Map(Box<Type>, Box<Type>),            // HashMap<K, V>
    HashSet(Box<Type>),                   // HashSet<T>
    BTreeMap(Box<Type>, Box<Type>),       // BTreeMap<K, V>
    BTreeSet(Box<Type>),                  // BTreeSet<T>
    Function(Vec<Type>, Box<Type>),

    // Error handling types (crucial for Rust's safety model)
    Option(Box<Type>),                    // Option<T>
    Result(Box<Type>, Box<Type>),         // Result<T, E>

    // Special types
    LogLevel,

    // User-defined types
    Custom(String),                       // Custom struct types
}

/// Represents patterns for pattern matching
#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum Pattern {
    /// Wildcard pattern `_` - matches anything
    Wildcard,
    /// Literal pattern - matches exact value
    Literal(Box<Expression>),
    /// Variable pattern - binds value to variable name
    Variable(String),
    /// Constructor pattern - e.g., Some[x], Ok[val], Err[e]
    Constructor {
        name: String,
        patterns: Vec<Pattern>,
    },
    /// Tuple pattern - e.g., (x, y, z)
    Tuple(Vec<Pattern>),
    /// List pattern - e.g., [x, y, z]
    List(Vec<Pattern>),
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct TypeAnnotation {
    pub name: String,
    pub type_: Type,
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    Number(i32),  // Default to i32 like Rust
    Float(f64),
    String(String),
    Boolean(bool),
    Tuple(Vec<Expression>),
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
    Program(Vec<Expression>),  // Multiple top-level expressions
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
    },

    // Error handling expressions (Rust's safety model)
    /// Represents None variant of Option
    None,
    /// Represents Some[value] variant of Option
    Some {
        value: Box<Expression>,
    },
    /// Represents Ok[value] variant of Result
    Ok {
        value: Box<Expression>,
    },
    /// Represents Err[error] variant of Result
    Err {
        error: Box<Expression>,
    },

    /// Pattern matching expression
    /// Structure: Match[value, [pattern1, result1], [pattern2, result2], ...]
    Match {
        value: Box<Expression>,
        arms: Vec<(Pattern, Expression)>,
    },

    /// Lambda/Closure expression
    /// Structure: Function[{param1, param2, ...}, body]
    /// or: Function[{param1: Type1, param2: Type2}, body]
    Lambda {
        parameters: Vec<TypeAnnotation>,
        body: Box<Expression>,
    },

    /// Struct definition
    /// Structure: Struct[Name, [field1: Type1, field2: Type2, ...]]
    StructDefinition {
        name: String,
        fields: Vec<TypeAnnotation>,
    },

    /// Struct instantiation
    /// Structure: StructName[value1, value2, ...]
    /// Used when a struct type is called as a constructor
    StructInstantiation {
        struct_name: String,
        field_values: Vec<Expression>,
    },
}

#[derive(Debug, Clone, PartialEq)]
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
