use crate::stmt::Stmt;
use crate::token::Token;
use crate::environment::Environment;

pub struct Interpreter {
    environment: Environment
}

impl Interpreter {
    pub fn new() -> Self {
        Interpreter {
            environment: Environment::new()
        }
    }

    pub fn interpret(&mut self, statements: Vec<Box<dyn Stmt>>) -> Result<(), (String, Token)> {
        for statement in statements {
            match statement.evaluate(&mut self.environment) {
                Ok(_) => {}
                Err((msg, token)) => return Err((String::from(msg), token.clone())),
            }
        }
        Ok(())
    }
}
