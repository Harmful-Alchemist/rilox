use crate::environment::Environment;
use crate::loxvalue::{Callable, LoxValue};
use crate::stmt::Stmt;
use crate::token::Token;
use crate::tokentype::TokenType;
use std::cell::RefCell;
use std::rc::Rc;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct Interpreter {
    environment: Rc<Environment>,
}

impl Interpreter {
    pub fn new() -> Self {
        let env = Rc::new(Environment::new());
        let callable = Callable {
            arity: 0,
            function: Rc::new(|_arguments, _env| {
                Ok(LoxValue::Number(
                    SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .expect("time went backwards")
                        .as_secs_f64(),
                ))
            }),
            string: "<native fn>".to_string(),
            name: Token {
                token_type: TokenType::Identifier,
                lexeme: "clock".to_string(),
                literal: LoxValue::None,
                line: 0,
            },
            environment: Rc::clone(&env),
            is_initializer: RefCell::new(false),
        };
        env.define(String::from("clock"), LoxValue::Callable(Rc::new(callable)));
        Interpreter { environment: env }
    }

    pub fn new_with_env(environment: Rc<Environment>) -> Self {
        Interpreter {
            environment: Rc::clone(&environment),
        }
    }

    pub fn interpret(
        &mut self,
        statements: Vec<Rc<dyn Stmt>>,
    ) -> Result<LoxValue, (String, Token)> {
        for statement in statements {
            match statement.evaluate(Rc::clone(&self.environment)) {
                Ok(LoxValue::Return(value)) => {
                    return Ok(*value);
                }
                Ok(_) => {}
                Err((msg, token)) => return Err((String::from(msg), token.clone())),
            }
        }
        Ok(LoxValue::None)
    }
}
