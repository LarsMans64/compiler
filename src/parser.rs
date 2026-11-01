use std::cmp::PartialEq;
use std::collections::HashMap;
use crate::ast::Program;
use crate::lexer::Token;

pub struct ParseError {
    line: usize,
    column: usize,
    found: Token,
}

pub fn parse_program(tokens: Vec<Token>) -> Result<Program, ParseError> {
    let rules = grammar_rules();
    
    todo!()
}

fn first_table() -> HashMap<NonTerminal, Vec<Option<Terminal>>> {
    todo!()
}

fn follow_table() -> HashMap<NonTerminal, Vec<Option<Terminal>>> {
    todo!()
}

type RightSide = Vec<GrammarSymbol>;

type GrammarRules = HashMap<NonTerminal, Vec<RightSide>>;

#[derive(Debug, PartialEq)]
enum GrammarSymbol {
    Terminal(Terminal),
    NonTerminal(NonTerminal),
    Epsilon,
}

#[derive(Debug, PartialEq, Eq, Hash)]
enum NonTerminal {
    Program,
    Function,
    Args,
    ArgsTail,
    Arg,
    Body,
    Statement,
    StatementRest,
    If,
    Else,
    While,
    Decl,
    Assign,
    Indices,
    Index,
    Call,
    Return,
    ExprList,
    ExprListTail,
    Expr,
    E1_,
    E2,
    E2_,
    E3,
    E3_,
    E4,
    E4_,
    E5,
    E5_,
    E6,
    E6_,
    E7,
    E7Rest,
}

#[derive(Debug, PartialEq)]
pub enum Terminal {
    Function,
    Var,
    Return,
    If,
    Else,
    While,
    Is,
    Comma,
    End,
    Eq,
    Neq,
    Gt,
    Lt,
    Gte,
    Lte,
    And,
    Or,
    Not,
    Add,
    Sub,
    Div,
    Pow,
    Asterisk,
    Address,
    ParOpen,
    ParClose,
    BracketOpen,
    BracketClose,
    BraceOpen,
    BraceClose,
    Boolean,
    Integer,
    Character,
    String,
    Identifier,
}

impl Terminal {
    fn matches_token(&self, token: &Token) -> bool {
         *self == match token {
            Token::Function => Terminal::Function,
            Token::Var => Terminal::Var,
            Token::Return => Terminal::Return,
            Token::If => Terminal::If,
            Token::Else => Terminal::Else,
            Token::While => Terminal::While,
            Token::Is => Terminal::Is,
            Token::Comma => Terminal::Comma,
            Token::End => Terminal::End,
            Token::Eq => Terminal::Eq,
            Token::Neq => Terminal::Neq,
            Token::Gt => Terminal::Gt,
            Token::Lt => Terminal::Lt,
            Token::Gte => Terminal::Gte,
            Token::Lte => Terminal::Lte,
            Token::And => Terminal::And,
            Token::Or => Terminal::Or,
            Token::Not => Terminal::Not,
            Token::Add => Terminal::Add,
            Token::Sub => Terminal::Sub,
            Token::Div => Terminal::Div,
            Token::Pow => Terminal::Pow,
            Token::Asterisk => Terminal::Asterisk,
            Token::Address => Terminal::Address,
            Token::ParOpen => Terminal::ParOpen,
            Token::ParClose => Terminal::ParClose,
            Token::BracketOpen => Terminal::BracketOpen,
            Token::BracketClose => Terminal::BracketClose,
            Token::BraceOpen => Terminal::BraceOpen,
            Token::BraceClose => Terminal::BraceClose,
            Token::Boolean(_) => Terminal::Boolean,
            Token::Integer(_) => Terminal::Integer,
            Token::Character(_) => Terminal::Character,
            Token::String(_) => Terminal::String,
            Token::Identifier(_) => Terminal::Identifier,
        }
    }
}



// ===== RULE DEFINITIONS =====

fn t(terminal: Terminal) -> GrammarSymbol {
    GrammarSymbol::Terminal(terminal)
}

fn n(non_terminal: NonTerminal) -> GrammarSymbol {
    GrammarSymbol::NonTerminal(non_terminal)
}

fn eps() -> RightSide {
    vec![GrammarSymbol::Epsilon]
}

fn grammar_rules() -> GrammarRules {
    use NonTerminal as N;
    use Terminal as T;

    HashMap::from([
        (N::Program, vec![vec![n(N::Function), n(N::Program)], eps()]),
        (
            N::Function,
            vec![vec![
                t(T::Function),
                t(T::Identifier),
                t(T::ParOpen),
                n(N::Args),
                t(T::ParClose),
                t(T::BraceOpen),
                n(N::Body),
                t(T::BraceClose),
            ]],
        ),
        (N::Args, vec![vec![n(N::Arg), n(N::ArgsTail)], eps()]),
        (N::ArgsTail, vec![vec![t(T::Comma), n(N::Arg), n(N::ArgsTail)], eps()]),
        (N::Arg, vec![vec![t(T::Identifier)]]),
        (N::Body, vec![vec![n(N::Statement), n(N::Body)], eps()]),
        (
            N::Statement,
            vec![
                vec![n(N::Decl), t(T::End)],
                vec![t(T::Identifier), n(N::StatementRest)],
                vec![n(N::Return), t(T::End)],
                vec![n(N::If)],
                vec![n(N::While)],
            ],
        ),
        (N::StatementRest, vec![vec![n(N::Assign)], vec![n(N::Call)]]),
        (N::Decl, vec![vec![t(T::Var), t(T::Identifier), t(T::Is), n(N::Expr)]]),
        (N::Assign, vec![vec![n(N::Indices), t(T::Is), n(N::Expr)]]),
        (N::Indices, vec![vec![n(N::Index), n(N::Indices)], eps()]),
        (N::Index, vec![vec![t(T::BracketOpen), n(N::Expr), t(T::BracketClose)]]),
        (N::Call, vec![vec![t(T::ParOpen), n(N::ExprList), t(T::ParClose)]]),
        (N::Return, vec![vec![t(T::Return), n(N::Expr)]]),
        (N::ExprList, vec![vec![n(N::Expr), n(N::ExprListTail)], eps()]),
        (N::ExprListTail, vec![vec![t(T::Comma), n(N::Expr), n(N::ExprListTail)], eps()]),
        (
            N::If,
            vec![vec![
                t(T::If),
                n(N::Expr),
                t(T::BraceOpen),
                n(N::Body),
                t(T::BraceClose),
                n(N::Else),
            ]],
        ),
        (
            N::Else,
            vec![
                vec![t(T::Else), n(N::If)],
                vec![t(T::Else), t(T::BraceOpen), n(N::Body), t(T::BraceClose)],
                eps(),
            ],
        ),
        (
            N::While,
            vec![vec![
                t(T::While),
                n(N::Expr),
                t(T::BraceOpen),
                n(N::Body),
                t(T::BraceClose),
            ]],
        ),
        (N::Expr, vec![vec![n(N::E2), n(N::E1_)]]),
        (
            N::E1_,
            vec![
                vec![t(T::Eq), n(N::E2), n(N::E1_)],
                vec![t(T::Neq), n(N::E2), n(N::E1_)],
                eps(),
            ],
        ),
        (N::E2, vec![vec![n(N::E3), n(N::E2_)]]),
        (
            N::E2_,
            vec![
                vec![t(T::Gt), n(N::E3), n(N::E2_)],
                vec![t(T::Lt), n(N::E3), n(N::E2_)],
                vec![t(T::Gte), n(N::E3), n(N::E2_)],
                vec![t(T::Lte), n(N::E3), n(N::E2_)],
                eps(),
            ],
        ),
        (N::E3, vec![vec![n(N::E4), n(N::E3_)]]),
        (N::E3_, vec![vec![t(T::Or), n(N::E4), n(N::E3_)], eps()]),
        (N::E4, vec![vec![n(N::E5), n(N::E4_)]]),
        (N::E4_, vec![vec![t(T::And), n(N::E5), n(N::E4_)], eps()]),
        (N::E5, vec![vec![n(N::E6), n(N::E5_)]]),
        (
            N::E5_,
            vec![
                vec![t(T::Add), n(N::E6), n(N::E5_)],
                vec![t(T::Sub), n(N::E6), n(N::E5_)],
                eps(),
            ],
        ),
        (N::E6, vec![vec![n(N::E7), n(N::E6_)]]),
        (
            N::E6_,
            vec![
                vec![t(T::Asterisk), n(N::E7), n(N::E6_)],
                vec![t(T::Div), n(N::E7), n(N::E6_)],
                eps(),
            ],
        ),
        (
            N::E7,
            vec![
                vec![t(T::ParOpen), n(N::Expr), t(T::ParClose)],
                vec![t(T::Not), n(N::Expr)],
                vec![t(T::Asterisk), n(N::Expr)],
                vec![t(T::Address), n(N::Expr)],
                vec![t(T::Integer)],
                vec![t(T::Sub), t(T::Integer)],
                vec![t(T::Boolean)],
                vec![t(T::Character)],
                vec![t(T::String)],
                vec![t(T::Identifier)],
                vec![n(N::Call)],
            ],
        ),
        (N::E7Rest, vec![vec![n(N::Call)], eps()]),
    ])
}