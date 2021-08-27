use crate::loxvalue::LoxValue;
use crate::token::Token;
use std::collections::HashMap;

pub struct Environment<'a> {
    enclosing: Option<&'a mut Environment<'a>>,
    values: HashMap<String, LoxValue>,
}

impl<'a> Environment<'a> {
    pub fn new() -> Self {
        Environment {
            enclosing: None,
            values: HashMap::new(),
        }
    }

    pub fn new_child(env: &'a mut Environment<'a>) -> Self {
        Environment {
            enclosing: Some(env),
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
                    let msg = format!("Undefined variable {}.", name.lexeme);
                    Err((msg, name.clone()))
                }
                Some(parent) => {
                    parent.assign(name, value);
                    Ok(())
                }
            }
        }
    }
}
