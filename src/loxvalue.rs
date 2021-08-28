use crate::token::Token;
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
}

pub struct Callable {
    pub(crate) arity: usize,
    pub(crate) call: Rc<dyn Fn(Vec<LoxValue>) -> LoxValue>,
    pub(crate) string: String,
    pub(crate) name: Token,
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

// impl Clone for Callable {
//     fn clone(&self) -> Self {
//         Callable {
//             arity: self.arity,
//             call: self.call.clone(),
//             string: self.string.clone(),
//             name: self.name.clone(),
//         }
//     }
// }

impl PartialEq for LoxValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (LoxValue::String(a), LoxValue::String(b)) => a == b,
            (LoxValue::Number(a), LoxValue::Number(b)) => a == b,
            (LoxValue::None, LoxValue::None) => true,
            (LoxValue::Bool(a), LoxValue::Bool(b)) => a == b,
            (LoxValue::Callable(_a), LoxValue::Callable(_b)) => false,
            //TODO Can't compare functions I guess, maybe with the Rc?
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
        }
    }
}
