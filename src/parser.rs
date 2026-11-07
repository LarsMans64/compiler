use crate::ast::{AssignmentTarget, BinaryOp, Expression, Function, Program, Statement, Type, UnaryOperator};
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

    fn peek(&self) -> &Token {
        if let Some(token) = self.tokens.last() {
            return &token.token;
        }
        panic!("should not continue parsing after end of file has been encountered")
    }

    fn advance(&mut self) -> PositionedToken {
        if let Some(token) = self.tokens.pop() {
            return token;
        }
        panic!("should not continue parsing after end of file has been encountered")
    }

    fn expect(&mut self, token: Token) -> Result<Token, ParseError> {
        let got = self.advance();
        if token == got.token {
            return Ok(got.token);
        }
        Err(error_message(&token.show(), got))
    }
}

fn error_message(expected: &str, got: PositionedToken) -> String {
    format!("Expected {} at {}:{}, got {}", expected, got.line, got.column, got.token.show())
}

pub fn parse_program(state: &mut ParseState) -> Result<Program, ParseError> {
    let mut program = Program::new();
    while state.peek() != &Token::EndOfFile {
        let function = parse_function(state)?;
        program.push(function);
    }
    Ok(program)
}

fn parse_function(state: &mut ParseState) -> Result<Function, ParseError> {
    state.expect(Token::Function)?;
    let name = parse_identifier(state)?;
    let args = parse_args(state)?;
    let return_type = if state.peek() == &Token::Arrow {
        state.advance();
        Some(parse_type(state)?)
    } else {
        None
    };
    let body = parse_body(state)?;
    Ok(Function { name, args, return_type, body })
}

fn parse_type(state: &mut ParseState) -> Result<Type, ParseError> {
    match state.peek() {
        Token::And => {
            state.advance();
            Ok(Type::Pointer(Box::new(parse_type(state)?)))
        }
        Token::BracketOpen => {
            state.advance();
            let typ = parse_type(state)?;
            state.expect(Token::BracketClose)?;
            Ok(Type::Array(Box::new(typ)))
        }
        Token::BooleanType => {
            state.advance();
            Ok(Type::Boolean)
        },
        Token::IntegerType => {
            state.advance();
            Ok(Type::Integer)
        },
        Token::CharacterType => {
            state.advance();
            Ok(Type::Character)
        },
        _ => Err(error_message("a type", state.advance()))
    }
}

fn parse_body(state: &mut ParseState) -> Result<Vec<Statement>, ParseError> {
    state.expect(Token::BraceOpen)?;
    let mut statements = Vec::new();
    loop {
        if let Token::BraceClose = state.peek() {
            state.advance();
            break;
        }
        statements.push(parse_statement(state)?);
    }
    Ok(statements)
}

fn parse_statement(state: &mut ParseState) -> Result<Statement, ParseError> {
    match state.peek() {
        Token::Var => {
            state.advance();
            let name = parse_identifier(state)?;
            let type_hint = if state.peek() == &Token::Colon {
                state.advance();
                Some(parse_type(state)?)
            } else {
                None
            };
            state.expect(Token::Is)?;
            let expr = parse_expr(state)?;
            state.expect(Token::End)?;
            Ok(Statement::Declaration {variable: name, type_hint, expression: expr})
        }
        Token::Return => {
            state.advance();
            if let Token::End = state.peek() {
                state.advance();
                return Ok(Statement::Return(None));
            }
            let expression = parse_expr(state)?;
            state.expect(Token::End)?;
            Ok(Statement::Return(Some(expression)))
        }
        Token::If => parse_if(state),
        Token::While => {
            state.advance();
            let condition = parse_expr(state)?;
            let then = parse_body(state)?;
            Ok(Statement::While {condition, then})
        }
        Token::Identifier(_) | Token::Star => {
            let target = parse_assignment_target(state)?;

            // Call
            if let AssignmentTarget::Variable(name) = target {
                state.advance();
                let args = parse_list_or_empty(state, parse_expr, Token::ParClose)?;
                state.expect(Token::End)?;
                return Ok(Statement::Call((name, args)));
            }

            // Assign
            state.expect(Token::Is)?;
            let expression = parse_expr(state)?;
            state.expect(Token::End)?;
            Ok(Statement::Assignment {target, expression})
        },
        Token::BraceOpen => Ok(Statement::Body(parse_body(state)?)),
        _ => Err(error_message("a statement", state.advance())),
    }
}

fn parse_assignment_target(state: &mut ParseState) -> Result<AssignmentTarget, ParseError> {
    let mut target = parse_assignment_target_rest(state)?;
    loop {
        let next_token = state.peek().clone();
        if next_token != Token::BracketOpen {
            break;
        }
        state.advance();
        target = AssignmentTarget::Index(Box::new(target), parse_expr(state)?);
        state.expect(Token::BracketClose)?;
    }
    Ok(target)
}

fn parse_assignment_target_rest(state: &mut ParseState) -> Result<AssignmentTarget, ParseError> {
    match state.peek() {
        Token::Star => {
            state.advance();
            Ok(AssignmentTarget::Dereference(Box::new(parse_assignment_target(state)?)))
        }
        Token::Identifier(name) => {
            let new_name = name.clone();
            state.advance();
            Ok(AssignmentTarget::Variable(new_name))
        }
        _ => Err(error_message("an assignment target", state.advance()))
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
    if state.peek() != &Token::Else {
        return Ok(Vec::new());
    }
    state.advance();
    if let Token::If = state.peek() {
        return Ok(vec![parse_if(state)?]);
    }
    Ok(parse_body(state)?)
}

fn parse_args(state: &mut ParseState) -> Result<Vec<(String, Type)>, ParseError> {
    state.expect(Token::ParOpen)?;
    let args = parse_list_or_empty(state, parse_arg, Token::ParClose)?;
    Ok(args)
}

fn parse_arg(state: &mut ParseState) -> Result<(String, Type), ParseError> {
    let name = parse_identifier(state)?;
    state.expect(Token::Colon)?;
    let typ = parse_type(state)?;
    Ok((name, typ))
}

fn parse_identifier(state: &mut ParseState) -> Result<String, ParseError> {
    let got = state.advance();
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
    parse_binary_expr(state, parse_access_expr, &[Token::Caret])
}

fn parse_access_expr(state: &mut ParseState) -> Result<Expression, ParseError> {
    let mut expr = parse_unary_expr(state)?;
    loop {
        let next_token = state.peek().clone();
        if next_token != Token::BracketOpen {
            break;
        }
        state.advance();
        expr = Expression::Access(Box::new(expr), Box::new(parse_unary_expr(state)?));
        state.expect(Token::BracketClose)?;
    }
    Ok(expr)
}

fn parse_unary_expr(state: &mut ParseState) -> Result<Expression, ParseError> {
    match state.peek() {
        Token::Minus => unary(state, UnaryOperator::Minus),
        Token::Not => unary(state, UnaryOperator::Not),
        Token::Star => unary(state, UnaryOperator::Dereference),
        Token::And => unary(state, UnaryOperator::AddressOf),
        _ => parse_unit_expr(state),
    }
}

fn parse_unit_expr(state: &mut ParseState) -> Result<Expression, ParseError> {
    match state.peek().clone() {
        Token::ParOpen => {
            state.advance();
            let inner = parse_expr(state)?;
            state.expect(Token::ParClose)?;
            Ok(inner)
        }
        Token::BracketOpen => {
            state.advance();
            let values = parse_list(state, parse_expr)?;
            state.expect(Token::BracketClose)?;
            Ok(Expression::Array(values))
        }
        Token::Boolean(value) => {
            state.advance();
            Ok(Expression::Boolean(value))
        }
        Token::Integer(value) => {
            state.advance();
            Ok(Expression::Integer(value))
        }
        Token::Character(value) => {
            state.advance();
            Ok(Expression::Character(value))
        }
        Token::String(value) => {
            state.advance();
            Ok(Expression::String(value))
        }
        Token::Identifier(name) => {
            state.advance();
            let name = name.clone();
            if let Token::ParOpen = state.peek() {
                state.advance();
                let args = parse_list_or_empty(state, parse_expr, Token::ParClose)?;
                return Ok(Expression::Call((name, args)));
            }
            Ok(Expression::Variable(name))
        }
        _ => Err(error_message("expression", state.advance())),
    }
}

fn parse_list_or_empty<T, F>(state: &mut ParseState, inner: F, terminator: Token) -> Result<Vec<T>, ParseError>
    where F: Fn(&mut ParseState) -> Result<T, ParseError>
{
    let mut items = Vec::new();
    if state.peek() != &terminator {
        items.push(inner(state)?);
        while let Token::Comma = state.peek() {
            state.advance();
            items.push(inner(state)?);
        }
    }
    state.expect(terminator)?;
    Ok(items)
}

fn parse_list<T, F>(state: &mut ParseState, inner: F) -> Result<Vec<T>, ParseError>
    where F: Fn(&mut ParseState) -> Result<T, ParseError>
{
    let mut items = Vec::new();
    items.push(inner(state)?);
    while let Token::Comma = state.peek() {
        state.advance();
        items.push(inner(state)?);
    }
    Ok(items)
}

fn parse_binary_expr<F>(state: &mut ParseState, inner: F, ops: &[Token]) -> Result<Expression, ParseError>
    where F: Fn(&mut ParseState) -> Result<Expression, ParseError>
{
    let mut expr = inner(state)?;
    loop {
        let next_token = state.peek().clone();
        if !ops.contains(&next_token) {
            break;
        }
        state.advance();
        expr = Expression::BinaryOperation(Box::new(expr), token_to_binary_op(&next_token).clone(), Box::new(inner(state)?));
    }
    Ok(expr)
}

fn unary(state: &mut ParseState, operator: UnaryOperator) -> Result<Expression, ParseError> {
    state.advance();
    Ok(Expression::UnaryOperation(operator, Box::new(parse_unit_expr(state)?)))
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
        Token::Plus => BinaryOp::Plus,
        Token::Star => BinaryOp::Times,
        Token::Minus => BinaryOp::Minus,
        Token::Slash => BinaryOp::DividedBy,
        Token::Caret => BinaryOp::RaisedTo,
        _ => panic!("unexpected token {:?}", token)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer;

    #[test]
    fn test_expr_order() {
        let code = lexer::process("2 == 1 + 1 * 1").unwrap();
        let expected = Expression::BinaryOperation(
            Box::new(Expression::Integer(2)),
            BinaryOp::Eq,
            Box::new(Expression::BinaryOperation(
                Box::new(Expression::Integer(1)),
                BinaryOp::Plus,
                Box::new(Expression::BinaryOperation (
                    Box::new(Expression::Integer(1)),
                    BinaryOp::Times,
                    Box::new(Expression::Integer(1)),
                )),
            )),
        );

        assert_eq!(parse_expr(&mut ParseState::new(code)).unwrap(), expected)
    }

    #[test]
    fn test_expr_left_recursion() {
        let code = lexer::process("1 & 2 & 3").unwrap();
        let expected = Expression::BinaryOperation(
            Box::new(Expression::BinaryOperation (
                Box::new(Expression::Integer(1)),
                BinaryOp::And,
                Box::new(Expression::Integer(2)),
            )),
            BinaryOp::And,
            Box::new(Expression::Integer(3)),
        );

        assert_eq!(parse_expr(&mut ParseState::new(code)).unwrap(), expected)
    }
}
