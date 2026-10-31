use std::fs::File;
use std::io::Read;
use std::path::Path;

mod lexer;

fn main() {
    let code = read_file("code.txt");

    let result = lexer::process(code.as_str());

    match result {
        Ok(tokens) => {
            for token in tokens {
                println!("{:?}", token);
            }
        }
        Err(error) => println!("syntax error at {}:{}", error.state.line, error.state.column),
    }
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