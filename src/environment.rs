use crate::loxvalue::LoxValue;
use crate::token::Token;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

pub struct Environment {
    pub(crate) enclosing: Option<Rc<Environment>>,
    pub(crate) values: RefCell<HashMap<String, LoxValue>>,
}

impl Clone for Environment {
    fn clone(&self) -> Self {
        Environment {
            enclosing: self.enclosing.clone(),
            values: self.values.clone(),
        }
    }

    fn clone_from(&mut self, source: &Self) {
        self.values = source.values.clone();
        self.enclosing = source.enclosing.clone();
    }
}

impl Environment {
    pub fn new() -> Self {
        Environment {
            enclosing: None,
            values: RefCell::new(HashMap::new()),
        }
    }

    pub fn new_child(env: Rc<Environment>) -> Self {
        Environment {
            enclosing: Some(env.clone()),
            values: RefCell::new(HashMap::new()),
        }
    }

    pub(crate) fn define(&self, key: String, value: LoxValue) {
        self.values.borrow_mut().insert(key, value);
    }

    pub(crate) fn get(&self, name: &Token) -> Result<LoxValue, String> {
        match self.values.borrow_mut().get(&*name.lexeme) {
            None => match &self.enclosing {
                None => Err(format!("Undefined variable '{}'.", name.lexeme)),
                Some(parent) => parent.get(name),
            },
            Some(a) => Ok(a.clone()),
        }
    }

    pub(crate) fn get_by_string(&self, name: String) -> Result<LoxValue, String> {
        match self.values.borrow_mut().get(&*name) {
            None => match &self.enclosing {
                None => Err(format!("Undefined variable '{}'.", name)),
                Some(parent) => parent.get_by_string(name),
            },
            Some(a) => Ok(a.clone()),
        }
    }

    pub(crate) fn assign(&self, name: &Token, value: LoxValue) -> Result<(), (String, Token)> {
        let lexeme = &*name.lexeme;
        if self.values.borrow_mut().contains_key(lexeme) {
            self.values.borrow_mut().insert(String::from(lexeme), value);
            return Ok(());
        }
        match &self.enclosing {
            None => {
                let msg = format!("Undefined variable '{}'.", name.lexeme);
                Err((msg, name.clone()))
            }
            Some(parent) => parent.assign(name, value),
        }
    }
}
