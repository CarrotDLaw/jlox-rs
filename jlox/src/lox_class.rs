use std::{cell::RefCell, collections::HashMap, fmt};

use crate::{error::*, interpreter::*, lox_callable::*, lox_instance::*, token::*};

#[derive(Debug, Clone, PartialEq)]
pub struct LoxClass {
  name: String,
  methods: RefCell<HashMap<String, Literal>>,
}

impl LoxClass {
  pub fn new(name: &str, methods: &HashMap<String, Literal>) -> LoxClass {
    LoxClass {
      name: name.to_string(),
      methods: RefCell::new(methods.clone()),
    }
  }

  pub fn find_method(&self, name: &str) -> Option<Literal> {
    self.methods.borrow().get(name).cloned()
  }
}

impl LoxCallable for LoxClass {
  fn arity(&self) -> u8 {
    0
  }

  fn call(&self, _interpreter: &Interpreter, _arguments: &[Literal]) -> Result<Literal, LoxError> {
    Ok(Literal::Instance(
      LoxInstance::new(&self.clone().into()).into(),
    ))
  }
}

impl fmt::Display for LoxClass {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.name)
  }
}
