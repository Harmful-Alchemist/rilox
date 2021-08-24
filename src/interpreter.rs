use crate::expr::Expr;
use crate::lox::Lox;
use crate::loxvalue::LoxValue;
use crate::token::Token;

pub struct Interpreter {}

impl Interpreter {
    pub fn new() -> Self {
        Interpreter {}
    }

    pub fn interpret(&mut self, expression: &dyn Expr) -> Result<(), (String, Token)> {
        match expression.evaluate() {
            Ok(value) => {
                println!("{}", value);
                Ok(())
            }
            Err((msg, token)) => Err((String::from(msg), token.clone())),
        }
    }
}
