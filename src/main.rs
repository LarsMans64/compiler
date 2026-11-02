use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::time::Instant;

mod lexer;
mod parser;
mod ast;
mod assembler;

fn main() {
    let now = Instant::now();

    let code = read_file("code.txt");
    let result = lexer::process(code.as_str());

    let tokens = match result {
        Ok(tokens) => tokens,
        Err(error) => {
            eprintln!("Unexpected character: \"{}\" at {}:{}",
                      error.state.remaining.chars().next().unwrap(),
                      error.state.line,
                      error.state.column);
            return;
        }
    };

    for token in &tokens {
        println!("{:?}", token);
    }

    let mut state = parser::ParseState::new(tokens);
    let program = match parser::parse_program(&mut state) {
        Ok(program) => program,
        Err(error) => {
            eprintln!("{error}");
            return;
        }
    };

    println!("AST:\n{:#?}", program);

    println!("{:?}", now.elapsed());
}

fn read_file(name: &str) -> String {
    let path = Path::new(name);
    let display = path.display();

    let mut file = match File::open(&path) {
        Err(why) => panic!("couldn't open {}: {}", display, why),
        Ok(file) => file,
    };

    let mut text = String::new();

    if let Err(why) = file.read_to_string(&mut text) {
        panic!("couldn't read {}: {}", display, why);
    }

    text
}