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

#[derive(Debug)]
pub enum Expression {
    BinaryOperation { operator: BinaryOp, left: Box<Expression>, right: Box<Expression> },
    UnaryOperation { operator: UnaryOperator, operand: Box<Expression> },
    Call { function: String, args: Vec<Expression> },
    Integer { value: u32 },
    Boolean { value: bool },
    Character { value: char },
    String { value: String },
    Variable { name: String },
}

#[derive(Debug, Clone)]
pub enum BinaryOp {
    Add,
    Subtract,
    Multiply,
    Divide,
    Power,
    And,
    Or,
    Eq,
    Neq,
    Lt,
    Gt,
    Lte,
    Gte,
}

#[derive(Debug, Clone)]
pub enum UnaryOperator {
    Negate,
    Invert,
    AddressOf,
    Dereference,
}