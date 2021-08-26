use crate::environment::Environment;
use crate::expr::Expr;
use crate::token::Token;

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
