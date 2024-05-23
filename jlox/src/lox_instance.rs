use std::{cell::RefCell, collections::HashMap, fmt, rc::Rc};

use crate::{error::*, lox_class::*, token::*};

#[derive(Debug, Clone, PartialEq)]
pub struct LoxInstance {
  class: Rc<LoxClass>,
  fields: RefCell<HashMap<String, Literal>>,
}

impl LoxInstance {
  pub fn new(class: &Rc<LoxClass>) -> LoxInstance {
    LoxInstance {
      class: class.clone(),
      fields: RefCell::new(HashMap::new()),
    }
  }

  pub fn get(&self, name: &Token) -> Result<Literal, LoxError> {
    if let Some(f) = self.fields.borrow().get(name.get_lexeme()) {
      return Ok(f.clone());
    }

    if let Some(m) = self.class.find_method(name.get_lexeme()) {
      return Ok(m.clone())
    }

    Err(LoxError::runtime_error(
      name,
      &format!("Undefined property '{}'.", name.get_lexeme()),
    ))
  }

  pub fn set(&self, name: &Token, value: &Literal) {
    self
      .fields
      .borrow_mut()
      .insert(name.get_lexeme().to_string(), value.clone());
  }
}

impl fmt::Display for LoxInstance {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{} instance", self.class)
  }
}
