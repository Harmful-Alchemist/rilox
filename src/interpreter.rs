use crate::environment::Environment;
use crate::loxvalue::{Callable, LoxValue};
use crate::stmt::Stmt;
use crate::token::Token;
use crate::tokentype::TokenType;
use std::rc::Rc;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct Interpreter {
    environment: Environment,
}

impl Interpreter {
    pub fn new() -> Self {
        let mut env = Environment::new();
        let callable = Callable {
            arity: 0,
            function: Rc::new(|_arguments, _env| {
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
                line: 0,
            },
            environment: Box::new(Environment::new()),
        };
        env.define(String::from("clock"), LoxValue::Callable(Rc::new(callable)));
        Interpreter { environment: env }
    }

    pub fn new_with_env(environment: Environment) -> Self {
        Interpreter { environment }
    }

    pub fn interpret(&mut self, statements: Vec<Rc<dyn Stmt>>) -> Result<(), (String, Token)> {
        for statement in statements {
            match statement.evaluate(&mut self.environment) {
                Ok(_) => {}
                Err((msg, token)) => return Err((String::from(msg), token.clone())),
            }
        }
        Ok(())
    }
}
