use std::collections::HashMap;

use crate::{error::*, token::*};

#[derive(Default)]
pub struct Environment {
  values: HashMap<String, Object>,
}

impl Environment {
  pub fn new() -> Environment {
    Environment {
      values: HashMap::new(),
    }
  }

  pub fn get(&self, name: &Token) -> Result<Object, LoxError> {
    let key = name.get_lexeme();
    Ok(
      self
        .values
        .get(key)
        .ok_or_else(|| LoxError::runtime_error(name, &format!("Undefined variable '{key}'.")))?
        .clone(),
    )
  }

  pub fn assign(&mut self, name: &Token, value: &Object) -> Result<(), LoxError> {
    let key = name.get_lexeme();
    if self.values.contains_key(key) {
      self.values.insert(key.to_string(), value.clone());
      return Ok(());
    }

    Err(LoxError::RuntimeError {
      token: name.clone(),
      message: format!("Undefined variable '{key}'."),
    })
  }

  pub fn define(&mut self, name: &str, value: Object) {
    self.values.insert(name.to_string(), value);
  }
}
