// use crate::expr::Expr;
use crate::interpreter::Interpreter;
use crate::parser::Parser;
use crate::scanner::Scanner;
use crate::token::Token;
use crate::tokentype::TokenType;
use std::io::Write;
use std::{fs, io};

pub struct Lox {
    had_error: bool,
    had_runtime_error: bool,
    interpreter: Interpreter,
}

impl Lox {
    pub fn new() -> Self {
        Lox {
            had_error: false,
            had_runtime_error: false,
            interpreter: Interpreter::new(),
        }
    }

    pub fn run_file(&mut self, path: &String) {
        self.run(fs::read_to_string(path).unwrap());
        if self.had_error {
            std::process::exit(65);
        }

        if self.had_runtime_error {
            std::process::exit(70);
        }
    }

    pub fn run_prompt(&mut self) {
        let stdin = io::stdin();

        loop {
            print!("> ");
            io::stdout().flush().unwrap();
            let mut buffer = String::new();
            let line = stdin.read_line(&mut buffer);
            match line {
                Ok(0) => break,
                Ok(_) => {
                    self.run(buffer.clone());
                    self.had_error = false
                }
                _ => break,
            }
        }
    }

    fn run(&mut self, source: String) {
        let mut scanner = Scanner::new(source, self);
        let tokens: Vec<Token> = scanner.scan_tokens();
        let mut parser = Parser::new(tokens, self);
        let statements = parser.parse();
        match self.interpreter.interpret(statements) {
            Ok(_) => {}
            Err((msg, token)) => self.runtime_error((String::from(msg), token.clone())),
        }
    }

    pub fn error(&mut self, line: u64, message: String) {
        self.report(line, String::from(""), message);
    }

    fn report(&mut self, line: u64, where_error: String, message: String) {
        eprintln!("[line {}] Error {}: {}", line, where_error, message);
        self.had_error = true;
    }

    pub fn error_parse(&mut self, token: &Token, msg: &str) {
        match token.token_type {
            TokenType::EOF => self.report(token.line, String::from("at end"), String::from(msg)),
            _ => self.report(
                token.line,
                format!("at '{}'", token.lexeme),
                String::from(msg),
            ),
        }
    }

    pub fn runtime_error(&mut self, error: (String, Token)) {
        let (msg, token) = error;
        eprintln!("{}\n[line {}]", msg, token.line);
        self.had_runtime_error = true;
    }
}
