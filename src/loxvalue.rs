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
    Class(Rc<Class>),
    Instance(Rc<InstanceValue>),
}

#[derive(Debug, Clone)]
pub struct InstanceValue {
    pub(crate) class: Rc<Class>,
    pub(crate) fields: RefCell<HashMap<String, LoxValue>>,
}

impl InstanceValue {
    pub fn get_value(&self, name: &Token) -> Result<LoxValue, (String, Token)> {
        match self.class.find_method(name.clone().lexeme) {
            None => {}
            Some(callable) => {
                let updated_method = callable.clone();
                updated_method.bind(LoxValue::Instance(Rc::new(self.clone())));
                return Ok(LoxValue::Callable(updated_method));
            }
        }

        match self.fields.borrow_mut().get(&*name.lexeme) {
            None => Err((
                format!("Undefined property '{}'.", name.lexeme),
                name.clone(),
            )),
            Some(value) => Ok(value.clone()),
        }
    }

    pub fn set_value(&self, name: String, value: LoxValue) {
        self.fields.borrow_mut().insert(name, value);
    }
}

#[derive(Debug)]
pub struct Class {
    pub(crate) name: String,
    pub(crate) arity: usize,
    pub(crate) methods: RefCell<HashMap<String, LoxValue>>,
    pub(crate) super_class: Option<Rc<Class>>,
}

impl Clone for Class {
    fn clone(&self) -> Self {
        Class {
            name: self.name.clone(),
            arity: self.arity,
            methods: RefCell::clone(&self.methods),
            super_class: self.super_class.clone(),
        }
    }
}

impl Class {
    pub(crate) fn call(&self, arguments: Vec<LoxValue>) -> Result<LoxValue, (String, Token)> {
        let instance = Rc::new(InstanceValue {
            class: Rc::new(self.clone()),
            fields: RefCell::new(HashMap::new()),
        });
        match self.methods.borrow().get("init") {
            Some(a) => match a {
                LoxValue::Callable(callable) => {
                    callable.bind(LoxValue::Instance(Rc::clone(&instance)));
                    return callable.call(arguments);
                }
                _ => {}
            },
            _ => {}
        }
        Ok(LoxValue::Instance(instance))
    }

    pub(crate) fn find_method(&self, name: String) -> Option<Rc<Callable>> {
        match self.methods.borrow().get(&*name) {
            None => match &self.super_class {
                None => None,
                Some(a) => a.find_method(name),
            },
            Some(method) => match method {
                LoxValue::Callable(callable) => Some(Rc::clone(callable)),
                _ => None,
            },
        }
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
    pub(crate) is_initializer: RefCell<bool>,
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
            is_initializer: RefCell::new(*self.is_initializer.borrow()),
        }
    }
}

impl Callable {
    pub(crate) fn call(&self, arguments: Vec<LoxValue>) -> Result<LoxValue, (String, Token)> {
        if self.arity != arguments.len() {
            return Err((
                format!(
                    "Expected {} argument(s) but got {}.",
                    self.arity,
                    arguments.len()
                ),
                self.name.clone(),
            ));
        };

        self.environment.define(
            self.name.lexeme.clone(),
            LoxValue::Callable(Rc::new(self.clone())),
        );

        let result = (self.function)(arguments, Rc::clone(&self.environment));

        if *self.is_initializer.borrow() {
            match self.environment.get_by_string(String::from("this")) {
                Ok(a) => Ok(a),
                Err(msg) => Err((msg, self.name.clone())),
            }
        } else {
            result
        }
    }

    pub(crate) fn bind(&self, instance: LoxValue) {
        self.environment.define(String::from("this"), instance);
    }

    pub(crate) fn bind_super(&self, instance: LoxValue) {
        self.environment.define(String::from("super"), instance);
    }

    pub(crate) fn set_initializer(&self) {
        self.is_initializer.swap(&RefCell::new(true));
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
            LoxValue::Instance(a) => write!(f, "{} instance", a.class.name),
        }
    }
}
