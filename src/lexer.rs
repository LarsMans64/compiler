use std::char;
use std::string::String;

#[derive(Debug)]
pub struct LexerError {
    pub state: LexerState,
}

#[derive(Debug)]
pub struct LexerState {
    pub remaining: String,
    pub column: usize,
    pub line: usize,
}

impl LexerState {
    fn new(input: &str) -> Self {
        let mut string = String::from(input);
        string.push('\n'); // line comments depend on \n after all lines
        LexerState {
            remaining: string,
            line: 1,
            column: 0,
        }
    }

    fn advance(&mut self, n: usize) {
        self.remaining = self.remaining[n..].to_string();
        self.column += n;
    }

    fn newline(&mut self) {
        self.remaining = self.remaining[1..].to_string();
        self.column = 0;
        self.line += 1;
    }
}

pub fn process(input: &str) -> Result<Vec<PositionedToken>, LexerError> {
    let mut tokens: Vec<PositionedToken> = Vec::new();
    let mut state = LexerState::new(input);

    while state.remaining.len() > 0 {
        if let Some('\n') = state.remaining.chars().next() {
            state.newline();
            continue;
        }

        if let Some(ignore_length) = ignore(&state.remaining) {
            state.advance(ignore_length);
            continue;
        }

        let matched_tokens = match_tokens(&state.remaining);
        let longest_match = matched_tokens.iter().max_by_key(|x| x.0);

        match longest_match {
            Some((n, token)) => {
                tokens.push(PositionedToken {
                    token: token.clone(),
                    column: state.column,
                    line: state.line,
                });
                state.advance(*n);
                continue;
            }
            None => {
                return Err(LexerError {
                    state,
                });
            }
        }
    }
    
    tokens.push(PositionedToken {
        token: Token::EndOfFile,
        column: state.column,
        line: state.line,
    });

    Ok(tokens)
}

fn ignore(input: &String) -> Option<usize> {
    // Check whitespace
    let chars = input.chars();
    for (i, char) in chars.enumerate() {
        if !(char == ' ' || char == '\t') {
            if i > 0 {
                return Some(i);
            }
            break;
        }
    }

    // Check line comment
    let mut chars = input.chars();
    if let (Some('/'), Some('/')) = (chars.next(), chars.next()) {
        for (i, char) in chars.enumerate() {
            if char == '\n' {
                return Some(i + 2);
            }
        }
    }

    // Check block comment
    let mut chars = input.chars();
    if let (Some('/'), Some('*')) = (chars.next(), chars.next()) {
        let mut end = false;
        for (i, char) in chars.enumerate() {
            if end && char == '/' {
                return Some(i + 3);
            }
            end = char == '*';
        }
    }

    None
}

type MatchingTokens = Vec<(usize, Token)>;

fn match_tokens(input: &str) -> MatchingTokens {
    let mut tokens: MatchingTokens = Vec::new();

    add_identifier(input, &mut tokens);

    add_string(input, &mut tokens);
    add_character(input, &mut tokens);
    add_integer(input, &mut tokens);

    add_if_exact_match(input, &mut tokens, "func", Token::Function);
    add_if_exact_match(input, &mut tokens, "var", Token::Var);
    add_if_exact_match(input, &mut tokens, "return", Token::Return);
    add_if_exact_match(input, &mut tokens, "if", Token::If);
    add_if_exact_match(input, &mut tokens, "else", Token::Else);
    add_if_exact_match(input, &mut tokens, "while", Token::While);

    add_if_exact_match(input, &mut tokens, "=", Token::Is);
    add_if_exact_match(input, &mut tokens, ":", Token::Colon);
    add_if_exact_match(input, &mut tokens, "->", Token::Arrow);
    add_if_exact_match(input, &mut tokens, ",", Token::Comma);
    add_if_exact_match(input, &mut tokens, ";", Token::End);

    add_if_exact_match(input, &mut tokens, "==", Token::Eq);
    add_if_exact_match(input, &mut tokens, "!=", Token::Neq);
    add_if_exact_match(input, &mut tokens, "<", Token::Lt);
    add_if_exact_match(input, &mut tokens, ">", Token::Gt);
    add_if_exact_match(input, &mut tokens, "<=", Token::Lte);
    add_if_exact_match(input, &mut tokens, ">=", Token::Gte);
    add_if_exact_match(input, &mut tokens, "&", Token::Amp);
    add_if_exact_match(input, &mut tokens, "|", Token::Bar);
    add_if_exact_match(input, &mut tokens, "!", Token::Excl);
    add_if_exact_match(input, &mut tokens, "+", Token::Plus);
    add_if_exact_match(input, &mut tokens, "-", Token::Minus);
    add_if_exact_match(input, &mut tokens, "*", Token::Star);
    add_if_exact_match(input, &mut tokens, "/", Token::Slash);
    add_if_exact_match(input, &mut tokens, "^", Token::Caret);


    add_if_exact_match(input, &mut tokens, "(", Token::ParOpen);
    add_if_exact_match(input, &mut tokens, ")", Token::ParClose);
    add_if_exact_match(input, &mut tokens, "[", Token::BracketOpen);
    add_if_exact_match(input, &mut tokens, "]", Token::BracketClose);
    add_if_exact_match(input, &mut tokens, "{", Token::BraceOpen);
    add_if_exact_match(input, &mut tokens, "}", Token::BraceClose);

    add_if_exact_match(input, &mut tokens, "bool", Token::BooleanType);
    add_if_exact_match(input, &mut tokens, "int", Token::IntegerType);
    add_if_exact_match(input, &mut tokens, "char", Token::CharacterType);

    add_if_exact_match(input, &mut tokens, "true", Token::Boolean(true));
    add_if_exact_match(input, &mut tokens, "false", Token::Boolean(false));

    tokens
}

fn add_identifier(input: &str, tokens: &mut MatchingTokens) {
    let chars = input.chars();

    let mut identifier: String = String::new();

    for (i, char) in chars.enumerate() {
        if !char.is_ascii_alphanumeric() && char != '_' {
            break;
        }

        if i == 0 && char.is_ascii_digit() {
            return;
        }

        identifier.push(char);
    }

    if identifier.len() > 0 {
        tokens.push((identifier.len(), Token::Identifier(identifier)));
    }
}

fn add_integer(input: &str, tokens: &mut MatchingTokens) {
    let chars = input.chars();

    let mut digits = 0;
    let mut collected_number = 0;
    for char in chars {
        if !char.is_ascii_digit() {
            break;
        }

        collected_number = collected_number * 10 + char.to_digit(10).unwrap();
        digits += 1;
    }
    if digits > 0 {
        tokens.push((digits, Token::Integer(collected_number)))
    }
}

fn add_character(input: &str, tokens: &mut MatchingTokens) {
    let mut chars = input.chars();

    let first = chars.next();
    let second = chars.next();
    let third = chars.next();

    if let (Some('\''), Some(char), Some('\'')) = (first, second, third) {
        if char != '\'' {
            tokens.push((3, Token::Character(char)));
        }
    }
}

fn add_string(input: &str, tokens:  &mut MatchingTokens) {
    let mut chars = input.chars();

    if let Some('"') = chars.next() {
        let mut string = String::new();
        let mut closed = false;
        for char in chars {
            if char == '"' {
                closed = true;
                break;
            }
            if char == '\n' { return; }
            string.push(char);
        }
        if closed {
            tokens.push((string.len() + 2, Token::String(string)));
        }
    }
}

fn add_if_exact_match(input: &str, tokens: &mut MatchingTokens, exact: &str, token: Token) {
    if input.starts_with(exact) {
        tokens.push((exact.len(), token));
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PositionedToken {
    pub token: Token,
    pub column: usize,
    pub line: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Token {
    EndOfFile,
    Function,
    Var,
    Return,
    If,
    Else,
    While,
    Is,
    Colon,
    Arrow,
    Comma,
    End,
    Eq,
    Neq,
    Gt,
    Lt,
    Gte,
    Lte,
    Amp,
    Bar,
    Excl,
    Plus,
    Minus,
    Slash,
    Caret,
    Star,
    ParOpen,
    ParClose,
    BracketOpen,
    BracketClose,
    BraceOpen,
    BraceClose,
    BooleanType,
    IntegerType,
    CharacterType,
    Boolean(bool),
    Integer(u32),
    Character(char),
    String(String),
    Identifier(String),
}

impl Token {
    pub fn show(&self) -> String {
        match self {
            Token::EndOfFile => String::from("end of file"),
            Token::Function => String::from("'func'"),
            Token::Var => String::from("'var'"),
            Token::Return => String::from("'return'"),
            Token::If => String::from("'if'"),
            Token::Else => String::from("'else'"),
            Token::While => String::from("'while'"),
            Token::Is => String::from("'='"),
            Token::Colon => String::from("':'"),
            Token::Arrow => String::from("'->'"),
            Token::Comma => String::from("','"),
            Token::End => String::from("';'"),
            Token::Eq => String::from("'=='"),
            Token::Neq => String::from("'!='"),
            Token::Gt => String::from("'>'"),
            Token::Lt => String::from("'<'"),
            Token::Gte => String::from("'>='"),
            Token::Lte => String::from("'<='"),
            Token::Amp => String::from("'&'"),
            Token::Bar => String::from("'|'"),
            Token::Excl => String::from("'!'"),
            Token::Plus => String::from("'+'"),
            Token::Minus => String::from("'-'"),
            Token::Slash => String::from("'/'"),
            Token::Caret => String::from("'^'"),
            Token::Star => String::from("'*'"),
            Token::ParOpen => String::from("'('"),
            Token::ParClose => String::from("')'"),
            Token::BracketOpen => String::from("'['"),
            Token::BracketClose => String::from("']'"),
            Token::BraceOpen => String::from("'{'"),
            Token::BraceClose => String::from("'}'"),
            Token::BooleanType => String::from("'bool'"),
            Token::IntegerType => String::from("'int'"),
            Token::CharacterType => String::from("'char'"),
            Token::Boolean(b) => if *b {String::from("'true'")} else {String::from("'false'")},
            Token::Integer(i) => format!("integer {}", i),
            Token::Character(c) => format!("character '{}'", c),
            Token::String(s) => format!("string \"{}\"", s),
            Token::Identifier(s) => format!("identifier '{}'", s),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lexer() {
        let input = "func var funcvar=,; a_1 13 \"\" '\"' //comment \n";
        let result = process(input);
        let expected = vec![
            Token::Function,
            Token::Var,
            Token::Identifier(String::from("funcvar")),
            Token::Is,
            Token::Comma,
            Token::End,
            Token::Identifier(String::from("a_1")),
            Token::Integer(13),
            Token::String(String::new()),
            Token::Character('"'),
        ];

        assert!(result.is_ok());
        assert_eq!(expected, result.unwrap().into_iter().map(|pt| pt.token).collect::<Vec<Token>>());
    }

    #[test]
    fn test_syntax_errors() {
        assert!(process("% hello").is_err());
        assert!(process("\"€\"").is_ok());
    }
}