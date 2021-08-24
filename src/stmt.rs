use crate::expr::Expr;
use crate::token::Token;

pub trait Stmt {
    fn evaluate(&self) -> Result<(), (&'static str, &Token)>;
}

pub struct Expression {
    pub(crate) expression: Box<dyn Expr>,
}

impl Stmt for Expression {
    fn evaluate(&self) -> Result<(), (&'static str, &Token)> {
        match self.expression.evaluate() {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
    }
}

pub struct Print {
    pub(crate) expression: Box<dyn Expr>,
}

impl Stmt for Print {
    fn evaluate(&self) -> Result<(), (&'static str, &Token)> {
        match self.expression.evaluate() {
            Ok(value) => {
                println!("{}", value);
                Ok(())
            }
            Err(e) => Err(e),
        }
    }
}
