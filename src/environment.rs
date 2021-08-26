use crate::loxvalue::LoxValue;
use crate::token::Token;
use std::collections::HashMap;

pub struct Environment {
    values: HashMap<String, LoxValue>,
}

impl Environment {
    pub fn new() -> Self {
        Environment {
            values: HashMap::new(),
        }
    }

    pub(crate) fn define(&mut self, key: String, value: LoxValue) {
        self.values.insert(key, value);
    }

    pub(crate) fn get(&mut self, name: &Token) -> Result<&LoxValue, String> {
        match self.values.get(&*name.lexeme) {
            None => Err(format!("Undefined variable '{}'.", name.lexeme)),
            Some(a) => Ok(a),
        }
    }
}
