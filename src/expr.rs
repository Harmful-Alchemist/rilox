use crate::environment::Environment;
use crate::loxvalue::LoxValue;
use crate::token::Token;
use crate::tokentype::TokenType;
use std::rc::Rc;

pub trait Expr {
    fn evaluate(&self, env: Rc<Environment>) -> Result<LoxValue, (String, Token)>;
    fn kind(&self) -> Kind;
}

pub enum Kind {
    Binary,
    Grouping,
    Literal,
    Unary,
    Variable(Token),
    NoOp,
    Assign,
    Logical,
    Call,
    Get(Token, Rc<dyn Expr>),
    Set,
}

pub struct Binary {
    pub(crate) left: Rc<dyn Expr>,
    pub(crate) operator: Token,
    pub(crate) right: Rc<dyn Expr>,
}

impl Expr for Binary {
    fn evaluate(&self, env: Rc<Environment>) -> Result<LoxValue, (String, Token)> {
        let left = self.left.evaluate(Rc::clone(&env))?;
        let right = self.right.evaluate(Rc::clone(&env))?;
        let token = self.operator.clone();
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

    fn kind(&self) -> Kind {
        Kind::Binary
    }
}

pub struct Grouping {
    pub(crate) expression: Rc<dyn Expr>,
}

impl Expr for Grouping {
    fn evaluate(&self, env: Rc<Environment>) -> Result<LoxValue, (String, Token)> {
        self.expression.evaluate(env)
    }

    fn kind(&self) -> Kind {
        Kind::Grouping
    }
}

pub struct Literal {
    pub(crate) value: crate::loxvalue::LoxValue,
}

impl Expr for Literal {
    fn evaluate(&self, _env: Rc<Environment>) -> Result<LoxValue, (String, Token)> {
        Ok(self.value.clone())
    }

    fn kind(&self) -> Kind {
        Kind::Literal
    }
}

pub struct Unary {
    pub(crate) operator: Token,
    pub(crate) right: Rc<dyn Expr>,
}

impl Expr for Unary {
    fn evaluate(&self, env: Rc<Environment>) -> Result<LoxValue, (String, Token)> {
        let right = self.right.evaluate(env)?;
        match self.operator.token_type {
            TokenType::Minus => match right {
                LoxValue::Number(a) => Ok(LoxValue::Number(-a.clone())),
                _ => Err((
                    String::from("Only know numbers to minus!"),
                    self.operator.clone(),
                )),
            },
            TokenType::Bang => is_truthy(right, true),
            _ => Err((
                String::from("Unknown unary operation"),
                self.operator.clone(),
            )),
        }
    }

    fn kind(&self) -> Kind {
        Kind::Unary
    }
}

pub struct Variable {
    pub(crate) name: Token,
}

impl Expr for Variable {
    fn evaluate(&self, env: Rc<Environment>) -> Result<LoxValue, (String, Token)> {
        match env.get(&self.name) {
            Ok(val) => Ok(val.clone()),
            Err(e) => Err((e, self.name.clone())),
        }
    }

    fn kind(&self) -> Kind {
        Kind::Variable(self.name.clone())
    }
}

pub struct NoOp {
    //Is fine for init variable without value will result in value none bound.
}

impl Expr for NoOp {
    fn evaluate(&self, _env: Rc<Environment>) -> Result<LoxValue, (String, Token)> {
        Ok(LoxValue::None)
    }

    fn kind(&self) -> Kind {
        Kind::NoOp
    }
}

pub struct Assign {
    pub(crate) name: Token,
    pub(crate) value: Rc<dyn Expr>,
}

impl Expr for Assign {
    fn evaluate(&self, env: Rc<Environment>) -> Result<LoxValue, (String, Token)> {
        let value = self.value.evaluate(Rc::clone(&env))?;
        match env.assign(&self.name, value.clone()) {
            Ok(_) => Ok(value.clone()),
            Err((msg, _token)) => Err((msg, self.name.clone())),
        }
    }

    fn kind(&self) -> Kind {
        Kind::Assign
    }
}

pub struct Logical {
    pub(crate) left: Rc<dyn Expr>,
    pub(crate) operator: Token,
    pub(crate) right: Rc<dyn Expr>,
}

impl Expr for Logical {
    fn evaluate(&self, env: Rc<Environment>) -> Result<LoxValue, (String, Token)> {
        let left = self.left.evaluate(Rc::clone(&env))?;
        match self.operator.token_type {
            TokenType::Or => match is_truthy(left.clone(), false)? {
                LoxValue::Bool(true) => Ok(left.clone()),
                _ => Ok(self.right.evaluate(Rc::clone(&env))?),
            },
            _ => match is_truthy(left.clone(), true)? {
                LoxValue::Bool(true) => Ok(left.clone()),
                _ => Ok(self.right.evaluate(Rc::clone(&env))?),
            },
        }
    }

    fn kind(&self) -> Kind {
        Kind::Logical
    }
}

pub struct Call {
    pub(crate) callee: Rc<dyn Expr>,
    pub(crate) paren: Token,
    pub(crate) arguments: Vec<Rc<dyn Expr>>,
}

impl Expr for Call {
    fn evaluate(&self, env: Rc<Environment>) -> Result<LoxValue, (String, Token)> {
        let function = self.callee.evaluate(Rc::clone(&env))?;
        let mut arguments: Vec<LoxValue> = Vec::new();
        for argument in &self.arguments {
            arguments.push(argument.evaluate(Rc::clone(&env))?);
        }
        match function {
            LoxValue::Callable(callable) => {
                if callable.arity != arguments.len() {
                    Err((
                        format!(
                            "Expected {} arguments but got {}.",
                            callable.arity,
                            arguments.len()
                        ),
                        self.paren.clone(),
                    ))
                } else {
                    match callable.call(arguments) {
                        Ok(a) => Ok(a),
                        Err((msg, token)) => Err((msg, token.clone())),
                    }
                }
            }
            LoxValue::Class(klass) => match klass.call(arguments) {
                Ok(a) => Ok(a),
                Err((msg, token)) => Err((msg, token.clone())),
            },
            _ => Err((
                String::from("Can only call functions and classes."),
                self.paren.clone(),
            )),
        }
    }

    fn kind(&self) -> Kind {
        Kind::Call
    }
}

pub struct Get {
    pub(crate) object: Rc<dyn Expr>,
    pub(crate) name: Token,
}

impl Expr for Get {
    fn evaluate(&self, env: Rc<Environment>) -> Result<LoxValue, (String, Token)> {
        let object = self.object.evaluate(env)?;
        match object {
            LoxValue::Instance(instance) => instance.get_value(&self.name),

            _ => Err((
                String::from("Only instances have properties."),
                self.name.clone(),
            )),
        }
    }

    fn kind(&self) -> Kind {
        Kind::Get(self.name.clone(), Rc::clone(&self.object))
    }
}

pub struct Set {
    pub(crate) object: Rc<dyn Expr>,
    pub(crate) name: Token,
    pub(crate) value: Rc<dyn Expr>,
}

impl Expr for Set {
    fn evaluate(&self, env: Rc<Environment>) -> Result<LoxValue, (String, Token)> {
        let object = self.object.evaluate(Rc::clone(&env))?;
        match object {
            LoxValue::Instance(a) => {
                let value = self.value.evaluate(Rc::clone(&env))?;
                a.set_value(self.name.lexeme.clone(), value.clone());
                Ok(value)
            }
            _ => Err((
                String::from("Only instances have fields."),
                self.name.clone(),
            )),
        }
    }

    fn kind(&self) -> Kind {
        Kind::Set
    }
}

pub fn is_truthy(val: LoxValue, invert: bool) -> Result<LoxValue, (String, Token)> {
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
