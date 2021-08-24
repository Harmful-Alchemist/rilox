use crate::loxvalue::LoxValue;
use crate::token::Token;
use crate::tokentype::TokenType;

pub trait Expr {
    fn pretty_print(&self) -> String;
    fn evaluate(&self) -> Result<LoxValue, (&'static str, &Token)>;
}

pub struct Binary {
    pub(crate) left: Box<dyn Expr>,
    pub(crate) operator: Token,
    pub(crate) right: Box<dyn Expr>,
}

impl Expr for Binary {
    fn pretty_print(&self) -> String {
        format!(
            "({} {} {})",
            self.operator.lexeme,
            self.left.pretty_print(),
            self.right.pretty_print()
        )
    }

    fn evaluate(&self) -> Result<LoxValue, (&'static str, &Token)> {
        let left = self.left.evaluate()?;
        let right = self.right.evaluate()?;
        let token = &self.operator;
        match self.operator.token_type {
            TokenType::BangEqual => Ok(is_equal(left, right, true)),
            TokenType::EqualEqual => Ok(is_equal(left, right, false)),
            TokenType::Greater => match (left, right) {
                (LoxValue::Number(a), LoxValue::Number(b)) => Ok(LoxValue::Bool(a > b)),
                _ => Err(("Can only compare two numbers.", token)),
            },
            TokenType::GreaterEqual => match (left, right) {
                (LoxValue::Number(a), LoxValue::Number(b)) => Ok(LoxValue::Bool(a >= b)),
                _ => Err(("Can only compare two numbers.", token)),
            },
            TokenType::Less => match (left, right) {
                (LoxValue::Number(a), LoxValue::Number(b)) => Ok(LoxValue::Bool(a < b)),
                _ => Err(("Can only compare two numbers.", token)),
            },
            TokenType::LessEqual => match (left, right) {
                (LoxValue::Number(a), LoxValue::Number(b)) => Ok(LoxValue::Bool(a <= b)),
                _ => Err(("Can only compare two numbers.", token)),
            },
            TokenType::Minus => match (left, right) {
                (LoxValue::Number(a), LoxValue::Number(b)) => {
                    Ok(LoxValue::Number(a.clone() - b.clone()))
                }
                _ => Err(("Can only subtract two numbers.", token)),
            },
            TokenType::Plus => match (left, right) {
                (LoxValue::Number(a), LoxValue::Number(b)) => {
                    Ok(LoxValue::Number(a.clone() + b.clone()))
                }
                (LoxValue::String(a), LoxValue::String(b)) => {
                    Ok(LoxValue::String(format!("{}{}", a, b)))
                }
                _ => Err((
                    "Can only add two numbers or concatenate two strings.",
                    token,
                )),
            },
            TokenType::Slash => match (left, right) {
                (LoxValue::Number(a), LoxValue::Number(b)) => {
                    Ok(LoxValue::Number(a.clone() / b.clone()))
                }
                _ => Err(("Can only divide two numbers.", token)),
            },
            TokenType::Star => match (left, right) {
                (LoxValue::Number(a), LoxValue::Number(b)) => {
                    Ok(LoxValue::Number(a.clone() * b.clone()))
                }
                _ => Err(("Can only multiply two numbers.", token)),
            },
            _ => Err(("Unknown binary operation.", token)),
        }
    }
}

pub struct Grouping {
    pub(crate) expression: Box<dyn Expr>,
}

impl Expr for Grouping {
    fn pretty_print(&self) -> String {
        format!("(group {})", self.expression.pretty_print())
    }

    fn evaluate(&self) -> Result<LoxValue, (&'static str, &Token)> {
        self.expression.evaluate()
    }
}

pub struct Literal {
    pub(crate) value: crate::loxvalue::LoxValue,
}

impl Expr for Literal {
    fn pretty_print(&self) -> String {
        format!("{}", self.value)
    }

    fn evaluate(&self) -> Result<LoxValue, (&'static str, &Token)> {
        Ok(self.value.clone())
    }
}

pub struct Unary {
    pub(crate) operator: Token,
    pub(crate) right: Box<dyn Expr>,
}

impl Expr for Unary {
    fn pretty_print(&self) -> String {
        format!("({} {})", self.operator.lexeme, self.right.pretty_print())
    }

    fn evaluate(&self) -> Result<LoxValue, (&'static str, &Token)> {
        let right = self.right.evaluate()?;
        match self.operator.token_type {
            TokenType::Minus => match right {
                LoxValue::Number(a) => Ok(LoxValue::Number(-a.clone())),
                _ => Err(("Only know numbers to minus!", &self.operator)),
            },
            TokenType::Bang => is_truthy(right, true),
            _ => Err(("Unknown unary operation", &self.operator)),
        }
    }
}

fn is_truthy(val: LoxValue, invert: bool) -> Result<LoxValue, (&'static str, &'static Token)> {
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
