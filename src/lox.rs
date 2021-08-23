use std::{fs, io};
use crate::token::Token;
use crate::scanner::Scanner;
use std::io::Write;

pub struct Lox {
    pub had_error: bool,
}

impl Lox {
    pub fn run_file(&mut self, path: &String) {
        // let bytes = fs::read(path)?;
        self.run(fs::read_to_string(path).unwrap());
        if self.had_error {
            std::process::exit(65);
        }
    }

    pub fn run_prompt(&mut self) {
        let stdin = io::stdin();
        let mut buffer = String::new();
        loop {
            print!("> ");
            io::stdout().flush().unwrap();
            let line = stdin.read_line(&mut buffer);
            match line {
                Ok(0) => break,
                Ok(_) => {
                    self.run(buffer.clone());
                    self.had_error = false
                }
                _ => break
            }
        }
    }

    fn run(&mut self, source: String) {
        let mut scanner = Scanner::new(source, self);
        let tokens: Vec<Token> = scanner.scan_tokens();
        for token in tokens {
            println!("{:?}", token);
        }
    }

    pub fn error(&mut self, line: u64, message: String) {
        self.report(line, String::from(""), message);
    }

    fn report(&mut self, line: u64, where_error: String, message: String) {
        eprintln!("[line {}] Error{}: {}", line, where_error, message);
        self.had_error = true;
    }
}