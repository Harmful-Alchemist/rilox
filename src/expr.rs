use crate::environment::Environment;
use crate::loxvalue::LoxValue;
use crate::token::Token;
use crate::tokentype::TokenType;

pub trait Expr {
    fn evaluate(&self, env: &mut Environment) -> Result<LoxValue, (String, &Token)>;
}

pub struct Binary {
    pub(crate) left: Box<dyn Expr>,
    pub(crate) operator: Token,
    pub(crate) right: Box<dyn Expr>,
}

impl Expr for Binary {
    fn evaluate(&self, env: &mut Environment) -> Result<LoxValue, (String, &Token)> {
        let left = self.left.evaluate(env)?;
        let right = self.right.evaluate(env)?;
        let token = &self.operator;
        match self.operator.token_type {
            TokenType::BangEqual => Ok(is_equal(left, right, true)),
            TokenType::EqualEqual => Ok(is_equal(left, right, false)),
            TokenType::Greater => match (left, right) {
                (LoxValue::Number(a), LoxValue::Number(b)) => Ok(LoxValue::Bool(a > b)),
                _ => Err((String::from("Can only compare two numbers."), token)),
            },
            TokenType::GreaterEqual => match (left, right) {
                (LoxValue::Number(a), LoxValue::Number(b)) => Ok(LoxValue::Bool(a >= b)),
                _ => Err((String::from("Can only compare two numbers."), token)),
            },
            TokenType::Less => match (left, right) {
                (LoxValue::Number(a), LoxValue::Number(b)) => Ok(LoxValue::Bool(a < b)),
                _ => Err((String::from("Can only compare two numbers."), token)),
            },
            TokenType::LessEqual => match (left, right) {
                (LoxValue::Number(a), LoxValue::Number(b)) => Ok(LoxValue::Bool(a <= b)),
                _ => Err((String::from("Can only compare two numbers."), token)),
            },
            TokenType::Minus => match (left, right) {
                (LoxValue::Number(a), LoxValue::Number(b)) => {
                    Ok(LoxValue::Number(a.clone() - b.clone()))
                }
                _ => Err((String::from("Can only subtract two numbers."), token)),
            },
            TokenType::Plus => match (left, right) {
                (LoxValue::Number(a), LoxValue::Number(b)) => {
                    Ok(LoxValue::Number(a.clone() + b.clone()))
                }
                (LoxValue::String(a), LoxValue::String(b)) => {
                    Ok(LoxValue::String(format!("{}{}", a, b)))
                }
                _ => Err((
                    String::from("Can only add two numbers or concatenate two strings."),
                    token,
                )),
            },
            TokenType::Slash => match (left, right) {
                (LoxValue::Number(a), LoxValue::Number(b)) => {
                    Ok(LoxValue::Number(a.clone() / b.clone()))
                }
                _ => Err((String::from("Can only divide two numbers."), token)),
            },
            TokenType::Star => match (left, right) {
                (LoxValue::Number(a), LoxValue::Number(b)) => {
                    Ok(LoxValue::Number(a.clone() * b.clone()))
                }
                _ => Err((String::from("Can only multiply two numbers."), token)),
            },
            _ => Err((String::from("Unknown binary operation."), token)),
        }
    }
}

pub struct Grouping {
    pub(crate) expression: Box<dyn Expr>,
}

impl Expr for Grouping {
    fn evaluate(&self, env: &mut Environment) -> Result<LoxValue, (String, &Token)> {
        self.expression.evaluate(env)
    }
}

pub struct Literal {
    pub(crate) value: crate::loxvalue::LoxValue,
}

impl Expr for Literal {
    fn evaluate(&self, _env: &mut Environment) -> Result<LoxValue, (String, &Token)> {
        Ok(self.value.clone())
    }
}

pub struct Unary {
    pub(crate) operator: Token,
    pub(crate) right: Box<dyn Expr>,
}

impl Expr for Unary {
    fn evaluate(&self, env: &mut Environment) -> Result<LoxValue, (String, &Token)> {
        let right = self.right.evaluate(env)?;
        match self.operator.token_type {
            TokenType::Minus => match right {
                LoxValue::Number(a) => Ok(LoxValue::Number(-a.clone())),
                _ => Err((String::from("Only know numbers to minus!"), &self.operator)),
            },
            TokenType::Bang => is_truthy(right, true),
            _ => Err((String::from("Unknown unary operation"), &self.operator)),
        }
    }
}

pub struct Variable {
    pub(crate) name: Token,
}

impl Expr for Variable {
    fn evaluate(&self, env: &mut Environment) -> Result<LoxValue, (String, &Token)> {
        match env.get(&self.name) {
            Ok(val) => Ok(val.clone()),
            Err(e) => Err((e, &self.name)),
        }
    }
}

pub struct NoOp {
    //Is fine for init variable without value will result in value none bound.
}

impl Expr for NoOp {
    fn evaluate(&self, _env: &mut Environment) -> Result<LoxValue, (String, &Token)> {
        Ok(LoxValue::None)
    }
}

fn is_truthy(val: LoxValue, invert: bool) -> Result<LoxValue, (String, &'static Token)> {
    match val {
        LoxValue::Bool(a) => {
            if invert {
                Ok(LoxValue::Bool(!a.clone()))
            } else {
                Ok(val.clone())
            }
        }
        LoxValue::None => Ok(LoxValue::Bool(false)),
        _ => Ok(LoxValue::Bool(true)),
    }
}

fn is_equal(val1: LoxValue, val2: LoxValue, invert: bool) -> LoxValue {
    if invert {
        LoxValue::Bool(val1 != val2)
    } else {
        LoxValue::Bool(val1 == val2)
    }
}
