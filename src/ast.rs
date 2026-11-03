pub type Program = Vec<Function>;

#[derive(Debug)]
pub struct Function {
    pub name: String,
    pub args: Vec<(String, Type)>, // will need types later
    pub return_type: Option<Type>,
    pub body: Vec<Statement>,
}

#[derive(Debug)]
pub enum Statement {
    Declaration { variable: String, type_hint: Option<Type>, expression: Expression }, // this is where an optional type will go
    Assignment { variable: String, indices: Vec<Expression>, expression: Expression },
    Call { function: String, args: Vec<Expression> },
    Return(Option<Expression>),
    If { condition: Expression, then: Vec<Statement>, otherwise: Vec<Statement> },
    While { condition: Expression, then: Vec<Statement> },
    Body(Vec<Statement>),
}

#[derive(Debug, PartialEq)]
pub enum Expression {
    BinaryOperation(Box<Expression>, BinaryOp, Box<Expression>),
    UnaryOperation(UnaryOperator, Box<Expression>),
    Call(String, Vec<Expression>),
    Integer(u32),
    Boolean(bool),
    Character(char),
    String(String),
    Variable(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOp {
    Addition,
    Subtraction,
    Multiplication,
    Division,
    Exponentiation,
    And,
    Or,
    Eq,
    Neq,
    Lt,
    Gt,
    Lte,
    Gte,
}

impl BinaryOp {
    pub fn show(&self) -> String {
        match self {
            BinaryOp::Addition => "+",
            BinaryOp::Subtraction => "-",
            BinaryOp::Multiplication => "*",
            BinaryOp::Division => "/",
            BinaryOp::Exponentiation => "^",
            BinaryOp::And => "&",
            BinaryOp::Or => "|",
            BinaryOp::Eq => "==",
            BinaryOp::Neq => "!=",
            BinaryOp::Lt => "<",
            BinaryOp::Gt => ">",
            BinaryOp::Lte => "<=",
            BinaryOp::Gte => ">=",
        }.to_string()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOperator {
    Negate,
    Invert,
    AddressOf,
    Dereference,
}

impl UnaryOperator {
    pub fn show(&self) -> String {
        match self {
            UnaryOperator::Negate => "-",
            UnaryOperator::Invert => "!",
            UnaryOperator::AddressOf => "&",
            UnaryOperator::Dereference => "*",
        }.to_string()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Integer,
    Boolean,
    Character,
    Pointer(Box<Type>),
    Array(Box<Type>),
}

impl Type {
    pub fn show(&self) -> String {
        match self {
            Type::Integer => String::from("int"),
            Type::Boolean => String::from("bool"),
            Type::Character => String::from("char"),
            Type::Pointer(t) => format!("&{}", t.show()),
            Type::Array(t) => format!("[{}]", t.show()),
        }
    }
}