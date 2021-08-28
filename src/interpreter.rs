use crate::environment::Environment;
use crate::loxvalue::{Callable, LoxValue};
use crate::stmt::Stmt;
use crate::token::Token;
use std::time::{SystemTime, UNIX_EPOCH};
use crate::tokentype::TokenType;
use std::rc::Rc;

pub struct Interpreter {
    environment: Environment,
}

impl Interpreter {
    pub fn new() -> Self {
        let mut env = Environment::new();
        let callable = Callable {
            arity: 0,
            call: Rc::new(|_arguments| {
                LoxValue::Number(
                    SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .expect("time went backwards")
                        .as_secs_f64(),
                )
            }),
            string: "<native fn>".to_string(),
            name: Token {
                token_type: TokenType::Identifier,
                lexeme: "clock".to_string(),
                literal: LoxValue::None,
                line: 0
            }
        };
        env.define(
            String::from("clock"),
            LoxValue::Callable(Box::new(callable)),
        );
        Interpreter { environment: env }
    }

    pub fn new_with_env(environment: Environment) -> Self {
        Interpreter{
            environment
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
