use crate::context::DataSize;

pub type Program = Vec<Function>;

#[derive(Debug)]
pub struct Function {
    pub name: String,
    pub args: Vec<(String, Type)>,
    pub return_type: Option<Type>,
    pub body: Vec<Statement>,
}

#[derive(Debug)]
pub enum Statement {
    Declaration { variable: String, type_hint: Option<Type>, expression: Expression },
    Assignment { target: AssignmentTarget, expression: Expression },
    Call(Call),
    Return(Option<Expression>),
    If { condition: Expression, then: Vec<Statement>, otherwise: Vec<Statement> },
    While { condition: Expression, then: Vec<Statement> },
}

#[derive(Debug)]
pub enum AssignmentTarget {
    Index(Box<AssignmentTarget>, Expression),
    Dereference(Box<AssignmentTarget>),
    Variable(String),
}

pub type Call = (String, Vec<Expression>);

#[derive(Debug, PartialEq)]
pub enum Expression {
    BinaryOperation(Box<Expression>, BinaryOp, Box<Expression>),
    UnaryOperation(UnaryOperator, Box<Expression>),
    Call(Call),
    Array(Vec<Expression>),
    Access(Box<Expression>, Box<Expression>),
    AddressOf(String),
    Integer(u32),
    Boolean(bool),
    Character(char),
    String(String),
    Variable(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOp {
    Plus,
    Minus,
    Times,
    DividedBy,
    And,
    Or,
    Xor,
    Eq,
    Neq,
    Lt,
    Gt,
    Lte,
    Gte,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOperator {
    Minus,
    Not,
    Dereference,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Integer,
    Boolean,
    Character,
    Pointer(Box<Type>),
    Array(Box<Type>),
}

impl BinaryOp {
    pub fn show(&self) -> String {
        match self {
            BinaryOp::Plus => "+",
            BinaryOp::Minus => "-",
            BinaryOp::Times => "*",
            BinaryOp::DividedBy => "/",
            BinaryOp::And => "&",
            BinaryOp::Or => "|",
            BinaryOp::Xor => "^",
            BinaryOp::Eq => "==",
            BinaryOp::Neq => "!=",
            BinaryOp::Lt => "<",
            BinaryOp::Gt => ">",
            BinaryOp::Lte => "<=",
            BinaryOp::Gte => ">=",
        }.to_string()
    }
}

impl UnaryOperator {
    pub fn show(&self) -> String {
        match self {
            UnaryOperator::Minus => "-",
            UnaryOperator::Not => "!",
            UnaryOperator::Dereference => "*",
        }.to_string()
    }
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
    
    pub fn size(&self) -> DataSize {
        match self {
            Type::Integer => 1,
            Type::Boolean => 1,
            Type::Character => 1,
            Type::Pointer(_) => 1,
            Type::Array(_) => 1,
        }
    }
}

enum Direction {
    North,
    East,
    South,
    West,
    Up,
    Down,
}

impl Direction {
    fn opposite(&self) -> Direction {
        match self {
            Direction::North => Direction::South,
            Direction::East => Direction::West,
            Direction::South => Direction::North,
            Direction::West => Direction::East,
            Direction::Up => Direction::Down,
            Direction::Down => Direction::Up,
        }
    }
}