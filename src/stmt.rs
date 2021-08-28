use crate::environment::Environment;
use crate::expr::{is_truthy, Expr};
use crate::loxvalue::{LoxValue, Callable};
use crate::token::Token;
use crate::tokentype::TokenType;
use crate::interpreter::Interpreter;
use std::rc::Rc;


pub trait Stmt {
    fn evaluate(&self, env: &mut Environment) -> Result<(), (String, &Token)>;
}

pub struct Expression {
    pub(crate) expression: Box<dyn Expr>,
}

impl Stmt for Expression {
    fn evaluate(&self, env: &mut Environment) -> Result<(), (String, &Token)> {
        match self.expression.evaluate(env) {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
    }
}

pub struct Print {
    pub(crate) expression: Box<dyn Expr>,
}

impl Stmt for Print {
    fn evaluate(&self, env: &mut Environment) -> Result<(), (String, &Token)> {
        match self.expression.evaluate(env) {
            Ok(value) => {
                println!("{}", value);
                Ok(())
            }
            Err(e) => Err(e),
        }
    }
}

pub struct Var {
    pub(crate) name: Token,
    pub(crate) initializer: Box<dyn Expr>,
}

impl Stmt for Var {
    fn evaluate(&self, env: &mut Environment) -> Result<(), (String, &Token)> {
        let val = self.initializer.evaluate(env)?;
        env.define(self.name.lexeme.clone(), val);
        Ok(())
    }
}

pub struct Block {
    pub(crate) statements: Vec<Box<dyn Stmt>>,
}

impl Stmt for Block {
    fn evaluate(&self, env: &mut Environment) -> Result<(), (String, &Token)> {
        let mut scoped_env = Environment::new_child(env);

        for statement in &self.statements {
            statement.evaluate(&mut scoped_env)?;
        }
        update_env(env, scoped_env);
        Ok(())
    }
}

pub struct If {
    pub(crate) condition: Box<dyn Expr>,
    pub(crate) then_branch: Box<dyn Stmt>,
    pub(crate) else_branch: Option<Box<dyn Stmt>>,
}

impl Stmt for If {
    fn evaluate(&self, env: &mut Environment) -> Result<(), (String, &Token)> {
        match is_truthy(self.condition.evaluate(env)?, false)? {
            LoxValue::Bool(true) => {
                self.then_branch.evaluate(env)?;
                Ok(())
            }
            _ => match &self.else_branch {
                None => Ok(()),
                Some(a) => {
                    a.evaluate(env)?;
                    Ok(())
                }
            },
        }
    }
}

pub struct While {
    pub(crate) condition: Box<dyn Expr>,
    pub(crate) body: Box<dyn Stmt>,
}

impl Stmt for While {
    fn evaluate(&self, env: &mut Environment) -> Result<(), (String, &Token)> {
        while is_truthy(self.condition.evaluate(env)?, false)? == LoxValue::Bool(true) {
            self.body.evaluate(env)?;
        }
        Ok(())
    }
}

pub struct Function {
   pub(crate) name: Token,
   pub(crate) params: Vec<Token>,
   pub(crate) body:Vec<Rc<dyn Stmt>>
}

impl Stmt for Function{
    fn evaluate(&self, env: &mut Environment) -> Result<(), (String, &Token)> {
        let function = LoxValue::Callable(Box::new(Callable{
            arity: self.params.len(),
            call: Rc::new(|arguments| {
                let mut scoped_env = env.clone();

                for (i, parameter) in self.params.clone().iter().enumerate() {
                    scoped_env.define(parameter.lexeme.clone(), arguments.get(i).expect("Checked").clone());
                }
                let mut interpreter = Interpreter::new_with_env(scoped_env.clone());
                //TODO cannot clone this! Can impl clone manually for all expr and stmt I guess.... Also is move necess? Might help when
                // cloning the loxvalue cause need a solution there. Or Rc statements?
                interpreter.interpret(self.body.clone());
                LoxValue::None

            }),
            string: format!("<fn {}>", self.name.lexeme),
            name: self.name.clone()
        }));
        env.define(self.name.lexeme.clone(), function);
        Ok(())
    }
}

fn update_env(env: &mut Environment, scoped_env: Environment) -> &mut Environment {
    match scoped_env.enclosing.clone() {
        None => env,
        Some(enclosing) => {
            for (key, val) in enclosing.values.clone() {
                let fake_token = Token {
                    token_type: TokenType::Var,
                    lexeme: key,
                    literal: LoxValue::None,
                    line: 0,
                };
                env.assign(&fake_token, val);
            }
            update_env(env, *enclosing)
        }
    }
}
