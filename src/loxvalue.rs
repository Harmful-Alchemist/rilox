use crate::environment::Environment;
use crate::token::Token;
use std::borrow::Borrow;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::fmt::{Debug, Formatter};
use std::rc::Rc;

#[derive(Debug, Clone)]
pub enum LoxValue {
    String(String),
    Number(f64),
    Bool(bool),
    None,
    Callable(Rc<Callable>),
    Return(Box<LoxValue>),
    Class(Rc<Klass>),
    Instance(Rc<InstanceValue>),
}

#[derive(Debug, Clone)]
pub struct InstanceValue {
    pub(crate) klass: Rc<Klass>,
    pub(crate) fields: RefCell<HashMap<String, LoxValue>>,
}

impl InstanceValue {
    pub fn get_value(&self, name: &Token) -> Result<LoxValue, (String, Token)> {
        match self.fields.borrow_mut().get(&*name.lexeme) {
            None => Err((
                format!("Undefined property '{}'.", name.lexeme),
                name.clone(),
            )),
            // TODO like want a mutable class property? Hmm. I think so then each Loxvalue needs to be rc, pfew
            // extra reference in the map. Ok-ish for now to make immutable to just keep going
            Some(value) => Ok(value.clone()),
        }
    }

    pub fn set_value(&self, name: String, value: LoxValue) {
        self.fields.borrow_mut().insert(name, value);
    }
}

#[derive(Debug, Clone)]
pub struct Klass {
    pub(crate) name: String,
    pub(crate) arity: usize,
}

impl Klass {
    pub(crate) fn call(&self, _arguments: Vec<LoxValue>) -> Result<LoxValue, (String, Token)> {
        Ok(LoxValue::Instance(Rc::new(InstanceValue {
            klass: Rc::new(self.clone()),
            fields: RefCell::new(HashMap::new()),
        })))
    }
}

pub struct Callable {
    pub(crate) arity: usize,
    pub(crate) function:
        Rc<dyn Fn(Vec<LoxValue>, Rc<Environment>) -> Result<LoxValue, (String, Token)>>,
    pub(crate) string: String,
    pub(crate) name: Token,
    // Below environment is the closure
    pub(crate) environment: Rc<Environment>,
}

impl Debug for Callable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Callable")
            .field("string", &self.string)
            .field("arity", &self.arity)
            .field("name", &self.name)
            .finish()
    }
}

impl Clone for Callable {
    fn clone(&self) -> Callable {
        let borrow: &Environment = self.environment.borrow();
        let env_clone = Rc::new(borrow.clone());
        Callable {
            arity: self.arity,
            function: Rc::clone(&self.function),
            string: self.string.clone(),
            name: self.name.clone(),
            environment: env_clone,
        }
    }
}

impl Callable {
    pub(crate) fn call(&self, arguments: Vec<LoxValue>) -> Result<LoxValue, (String, Token)> {
        // let mut call_env = self.environment.clone();
        self.environment.define(
            self.name.lexeme.clone(),
            LoxValue::Callable(Rc::new(self.clone())),
        );
        (self.function)(arguments, Rc::clone(&self.environment))
    }
}

impl PartialEq for LoxValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (LoxValue::String(a), LoxValue::String(b)) => a == b,
            (LoxValue::Number(a), LoxValue::Number(b)) => a == b,
            (LoxValue::None, LoxValue::None) => true,
            (LoxValue::Bool(a), LoxValue::Bool(b)) => a == b,
            (LoxValue::Callable(a), LoxValue::Callable(b)) => Rc::ptr_eq(a, b),
            _ => false,
        }
    }
}

impl Eq for LoxValue {}

impl fmt::Display for LoxValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            LoxValue::String(a) => write!(f, "\"{}\"", a),
            LoxValue::Number(a) => write!(f, "{}", a),
            LoxValue::Bool(a) => write!(f, "{}", a),
            LoxValue::None => write!(f, "nil"),
            LoxValue::Callable(a) => write!(f, "{}", a.string),
            LoxValue::Return(a) => write!(f, "<return {}>", a),
            LoxValue::Class(a) => write!(f, "{}", a.name),
            LoxValue::Instance(a) => write!(f, "{} instance", a.klass.name),
        }
    }
}
