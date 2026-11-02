pub type Program = Vec<Function>;

#[derive(Debug)]
pub struct Function {
    pub name: String,
    pub args: Vec<String>,
    pub body: Vec<Statement>,
}

#[derive(Debug)]
pub enum Statement {
    Declaration { variable: String, expression: Expression },
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

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOperator {
    Negate,
    Invert,
    AddressOf,
    Dereference,
}