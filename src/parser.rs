use crate::ast::{BinaryOp, Expression, Function, Program, Statement, UnaryOperator};
use crate::lexer::{PositionedToken, Token};

pub type ParseError = String;

pub struct ParseState {
    tokens: Vec<PositionedToken>,
}

impl ParseState {
    pub fn new(tokens: Vec<PositionedToken>) -> Self {
        ParseState {
            tokens: tokens.into_iter().rev().collect(),
        }
    }

    fn done(&self) -> bool {
        self.tokens.len() == 0
    }

    fn peek(&self) -> Result<&Token, ParseError> {
        match self.tokens.last() {
            Some(token) => Ok(&token.token),
            None => Err(String::from("unexpected end of file")),
        }
    }

    fn advance(&mut self) -> Result<PositionedToken, ParseError> {
        match self.tokens.pop() {
            Some(token) => Ok(token),
            None => Err(String::from("unexpected end of file")),
        }
    }

    fn expect(&mut self, token: Token) -> Result<Token, ParseError> {
        let got = self.advance()?;
        if token == got.token {
            return Ok(got.token);
        }
        Err(error_message(&token.show(), got))
    }
}

fn error_message(expected: &str, got: PositionedToken) -> String {
    format!("expected {} at {}:{}, got {}", expected, got.line, got.column, got.token.show())
}

pub fn parse_program(state: &mut ParseState) -> Result<Program, ParseError> {
    let mut program = Program::new();
    while !state.done() {
        let function = parse_function(state)?;
        program.push(function);
    }
    Ok(program)
}

fn parse_function(state: &mut ParseState) -> Result<Function, ParseError> {
    state.expect(Token::Function)?;
    let name = parse_identifier(state)?;
    let args = parse_args(state)?;
    let body = parse_body(state)?;
    Ok(Function { name, args, body })
}

fn parse_body(state: &mut ParseState) -> Result<Vec<Statement>, ParseError> {
    state.expect(Token::BraceOpen)?;
    let mut statements = Vec::new();
    loop {
        if let Token::BraceClose = state.peek()? {
            state.advance()?;
            break;
        }
        statements.push(parse_statement(state)?);
    }
    Ok(statements)
}

fn parse_statement(state: &mut ParseState) -> Result<Statement, ParseError> {
    let token = state.peek()?;
    match token {
        Token::Var => {
            state.advance()?;
            let name = parse_identifier(state)?;
            state.expect(Token::Is)?;
            let expr = parse_expr(state)?;
            state.expect(Token::End)?;
            Ok(Statement::Declaration {variable: name, expression: expr})
        }
        Token::Return => {
            state.advance()?;
            if let Token::End = state.peek()? {
                state.advance()?;
                return Ok(Statement::Return(None));
            }
            let expression = parse_expr(state)?;
            state.expect(Token::End)?;
            Ok(Statement::Return(Some(expression)))
        }
        Token::If => parse_if(state),
        Token::While => {
            state.advance()?;
            let condition = parse_expr(state)?;
            let then = parse_body(state)?;
            Ok(Statement::While {condition, then})
        }
        Token::Identifier(name) => {
            let name = name.clone();
            state.advance()?;

            // Call
            if let Token::ParOpen = state.peek()? {
                state.advance()?;
                let args = parse_list(state, parse_expr, Token::ParClose)?;
                state.expect(Token::ParClose)?;
                state.expect(Token::End)?;
                return Ok(Statement::Call {function: name, args});
            }

            // Assign
            let mut indices = Vec::new();
            while let Token::BracketOpen = state.peek()? {
                state.advance()?;
                indices.push(parse_expr(state)?);
                state.expect(Token::BracketClose)?;
            }
            state.expect(Token::Is)?;
            let expression = parse_expr(state)?;
            state.expect(Token::End)?;
            Ok(Statement::Assignment {variable: name, indices, expression})
        },
        Token::BraceOpen => Ok(Statement::Body(parse_body(state)?)),
        _ => Err(error_message("statement", state.advance()?)),
    }
}

fn parse_if(state: &mut ParseState) -> Result<Statement, ParseError> {
    state.expect(Token::If)?;
    let condition = parse_expr(state)?;
    let then = parse_body(state)?;
    let otherwise = parse_else(state)?;
    Ok(Statement::If {condition, then, otherwise})
}

fn parse_else(state: &mut ParseState) -> Result<Vec<Statement>, ParseError> {
    if state.peek()? != &Token::Else {
        return Ok(Vec::new());
    }
    state.advance()?;
    if let Token::If = state.peek()? {
        return Ok(vec![parse_if(state)?]);
    }
    Ok(parse_body(state)?)
}

fn parse_args(state: &mut ParseState) -> Result<Vec<String>, ParseError> {
    state.expect(Token::ParOpen)?;
    let args = parse_list(state, parse_identifier, Token::ParClose)?;
    state.expect(Token::ParClose)?;
    Ok(args)
}

fn parse_identifier(state: &mut ParseState) -> Result<String, ParseError> {
    let got = state.advance()?;
    if let Token::Identifier(name) = got.token {
        return Ok(name);
    }
    Err(error_message("identifier", got))
}

fn parse_expr(state: &mut ParseState) -> Result<Expression, ParseError> {
    parse_binary_expr(state, parse_and_expr, &[Token::Or])
}

fn parse_and_expr(state: &mut ParseState) -> Result<Expression, ParseError> {
    parse_binary_expr(state, parse_equality_expr, &[Token::And])
}

fn parse_equality_expr(state: &mut ParseState) -> Result<Expression, ParseError> {
    parse_binary_expr(state, parse_relation_expr, &[Token::Eq, Token::Neq])
}

fn parse_relation_expr(state: &mut ParseState) -> Result<Expression, ParseError> {
    parse_binary_expr(state, parse_additive_expr, &[Token::Lt, Token::Gt, Token::Lte, Token::Gte])
}

fn parse_additive_expr(state: &mut ParseState) -> Result<Expression, ParseError> {
    parse_binary_expr(state, parse_multiplicative_expr, &[Token::Plus, Token::Minus])
}

fn parse_multiplicative_expr(state: &mut ParseState) -> Result<Expression, ParseError> {
    parse_binary_expr(state, parse_exponential_expr, &[Token::Star, Token::Slash])
}

fn parse_exponential_expr(state: &mut ParseState) -> Result<Expression, ParseError> {
    parse_binary_expr(state, parse_unary_expr, &[Token::Caret])
}

fn parse_unary_expr(state: &mut ParseState) -> Result<Expression, ParseError> {
    match state.peek()? {
        Token::Minus => unary(state, UnaryOperator::Negate),
        Token::Not => unary(state, UnaryOperator::Invert),
        Token::Star => unary(state, UnaryOperator::Dereference),
        Token::And => unary(state, UnaryOperator::AddressOf),
        _ => parse_unit_expr(state),
    }
}

fn parse_unit_expr(state: &mut ParseState) -> Result<Expression, ParseError> {
    match state.peek()?.clone() {
        Token::ParOpen => {
            state.advance()?;
            let inner = parse_expr(state)?;
            state.expect(Token::ParClose)?;
            Ok(inner)
        }
        Token::Boolean(value) => {
            state.advance()?;
            Ok(Expression::Boolean {value})
        }
        Token::Integer(value) => {
            state.advance()?;
            Ok(Expression::Integer {value})
        }
        Token::Character(value) => {
            state.advance()?;
            Ok(Expression::Character {value})
        }
        Token::String(value) => {
            state.advance()?;
            Ok(Expression::String {value})
        }
        Token::Identifier(name) => {
            state.advance()?;
            let name = name.clone();
            if let Token::ParOpen = state.peek()? {
                state.advance()?;
                let args = parse_list(state, parse_expr, Token::ParClose)?;
                state.expect(Token::ParClose)?;
                return Ok(Expression::Call {function: name, args});
            }
            Ok(Expression::Variable {name})
        }
        _ => Err(error_message("expression", state.advance()?)),
    }
}

fn parse_list<T, F>(state: &mut ParseState, inner: F, terminator: Token) -> Result<Vec<T>, ParseError>
    where F: Fn(&mut ParseState) -> Result<T, ParseError>
{
    let mut items = Vec::new();
    if state.peek()? != &terminator {
        items.push(inner(state)?);
        while let Token::Comma = state.peek()? {
            state.advance()?;
            items.push(inner(state)?);
        }
    }
    Ok(items)
}

fn parse_binary_expr<F>(state: &mut ParseState, inner: F, ops: &[Token]) -> Result<Expression, ParseError>
where F: Fn(&mut ParseState) -> Result<Expression, ParseError>
{
    let mut expr = inner(state)?;
    loop {
        let next_token = state.peek()?.clone();
        if !ops.contains(&next_token) {
            break;
        }
        state.advance()?;
        expr = Expression::BinaryOperation {
            operator: token_to_binary_op(&next_token).clone(),
            left: Box::new(expr),
            right: Box::new(inner(state)?),
        };
    }
    Ok(expr)
}

fn unary(state: &mut ParseState, operator: UnaryOperator) -> Result<Expression, ParseError> {
    state.advance()?;
    Ok(Expression::UnaryOperation {
        operator,
        operand: Box::new(parse_unit_expr(state)?),
    })
}

fn token_to_binary_op(token: &Token) -> BinaryOp {
    match token {
        Token::Eq => BinaryOp::Eq,
        Token::Neq => BinaryOp::Neq,
        Token::Gt => BinaryOp::Gt,
        Token::Lt => BinaryOp::Lt,
        Token::Gte => BinaryOp::Gte,
        Token::Lte => BinaryOp::Lte,
        Token::And => BinaryOp::And,
        Token::Or => BinaryOp::Or,
        Token::Plus => BinaryOp::Add,
        Token::Star => BinaryOp::Multiply,
        Token::Minus => BinaryOp::Subtract,
        Token::Slash => BinaryOp::Divide,
        Token::Caret => BinaryOp::Power,
        _ => panic!("unexpected token {:?}", token)
    }
}
