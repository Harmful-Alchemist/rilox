use crate::loxvalue::LoxValue;
use crate::token::Token;
use std::collections::HashMap;

pub struct Environment {
    pub(crate) enclosing: Option<Box<Environment>>,
    pub(crate) values: HashMap<String, LoxValue>,
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
            values: HashMap::new(),
        }
    }

    pub fn new_child(env: &mut Environment) -> Self {
        Environment {
            enclosing: Some(Box::from(env.clone())),
            values: HashMap::new(),
        }
    }

    pub(crate) fn define(&mut self, key: String, value: LoxValue) {
        self.values.insert(key, value);
    }

    pub(crate) fn get(&self, name: &Token) -> Result<&LoxValue, String> {
        match self.values.get(&*name.lexeme) {
            None => match &self.enclosing {
                None => Err(format!("Undefined variable '{}'.", name.lexeme)),
                Some(parent) => parent.get(name),
            },
            Some(a) => Ok(a),
        }
    }

    pub(crate) fn assign(&mut self, name: &Token, value: LoxValue) -> Result<(), (String, Token)> {
        let lexeme = &*name.lexeme;
        if self.values.contains_key(lexeme) {
            self.values.insert(String::from(lexeme), value);
            Ok(())
        } else {
            match &mut self.enclosing {
                None => {
                    let msg = format!("Undefined variable '{}'.", name.lexeme);
                    Err((msg, name.clone()))
                }
                Some(parent) => parent.assign(name, value),
            }
        }
    }
}
