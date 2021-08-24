use crate::stmt::Stmt;
use crate::token::Token;

pub struct Interpreter {}

impl Interpreter {
    pub fn new() -> Self {
        Interpreter {}
    }

    pub fn interpret(&mut self, statements: Vec<Box<dyn Stmt>>) -> Result<(), (String, Token)> {
        for statement in statements {
            match statement.evaluate() {
                Ok(_) => {}
                Err((msg, token)) => return Err((String::from(msg), token.clone())),
            }
        }
        Ok(())
    }
}
