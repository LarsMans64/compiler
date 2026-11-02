pub type Program = Vec<Function>;

pub struct Function {
    pub name: String,
    pub args: Vec<String>,
    pub body: Vec<Statement>,
}

pub enum Statement {
    Declaration { variable: String, expression: Expression },
    Assignment { variable: String, indices: Vec<Expression>, expression: Expression },
    Call { function: String, args: Vec<Expression> },
    Return(Option<Expression>),
    If { condition: Expression, then: Vec<Statement>, otherwise: Vec<Statement> },
    While { condition: Expression, then: Vec<Statement> },
    Body(Vec<Statement>),
}

pub enum Expression {
    BinaryOperation { operator: BinaryOperator, left: Box<Expression>, right: Box<Expression> },
    UnaryOperation { operator: UnaryOperator, operand: Box<Expression> },
    Call { function: String, args: Vec<Expression> },
    Integer { value: i32 },
    Boolean { value: bool },
    Character { value: char },
    String { value: String },
    Variable { name: String },
}

pub enum BinaryOperator {
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

pub enum UnaryOperator {
    Negate,
    Invert,
    GetPointer,
    DerefPointer,
}