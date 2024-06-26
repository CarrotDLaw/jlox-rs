use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{error::*, token::*};

#[derive(Default, Debug)]
pub struct Environment {
  values: HashMap<String, Literal>,
  enclosing: Option<Rc<RefCell<Environment>>>,
}

impl Environment {
  pub fn new() -> Environment {
    Environment {
      values: HashMap::new(),
      enclosing: None,
    }
  }

  pub fn new_with_enclosing(enclosing: &Rc<RefCell<Environment>>) -> Environment {
    Environment {
      values: HashMap::new(),
      enclosing: Some(enclosing.clone()),
    }
  }

  pub fn get(&self, name: &Token) -> Result<Literal, LoxError> {
    let key = name.get_lexeme();

    if let Some(o) = self.values.get(key) {
      return Ok(o.clone());
    }

    if let Some(e) = &self.enclosing {
      return e.borrow().get(name);
    }

    Err(LoxError::runtime_error(
      name,
      &format!("Undefined variable '{key}'."),
    ))
  }

  pub fn assign(&mut self, name: &Token, value: &Literal) -> Result<(), LoxError> {
    let key = name.get_lexeme();

    if self.values.contains_key(key) {
      self.values.insert(key.to_string(), value.clone());
      return Ok(());
    }

    if let Some(e) = &self.enclosing {
      return e.borrow_mut().assign(name, value);
    }

    Err(LoxError::runtime_error(
      name,
      &format!("Undefined variable '{key}'."),
    ))
  }

  pub fn define(&mut self, name: &str, value: Literal) {
    self.values.insert(name.to_string(), value);
  }

  pub fn get_at(&self, distance: usize, name: &str) -> Result<Literal, LoxError> {
    if distance.eq(&0) {
      if let Some(l) = self.values.get(name) {
        return Ok(l.clone());
      }
    }

    if let Some(e) = self.enclosing.as_ref() {
      return e.borrow().get_at(distance - 1, name);
    }

    unreachable!()
  }

  pub fn assign_at(
    &mut self,
    distance: usize,
    name: &Token,
    value: &Literal,
  ) -> Result<(), LoxError> {
    if distance.eq(&0) {
      self
        .values
        .insert(name.get_lexeme().to_string(), value.clone());
      return Ok(());
    }

    if let Some(e) = self.enclosing.as_ref() {
      e.borrow_mut().assign_at(distance - 1, name, value)?;
      return Ok(());
    }

    unreachable!()
  }
}

#[cfg(test)]
mod test {
  use super::*;

  #[test]
  fn test_define_variable() {
    let mut env = Environment::new();
    env.define("foo", Literal::Boolean(true));
    assert!(env.values.contains_key("foo"));
    assert!(matches!(env.values.get("foo"), Some(&Literal::Boolean(b)) if b));
  }

  #[test]
  fn test_redefine_variable() {
    let mut env = Environment::new();
    env.define("foo", Literal::Boolean(true));
    env.define("foo", Literal::Number(12.3));
    assert!(matches!(env.values.get("foo"), Some(&Literal::Number(n)) if n == 12.3));
  }

  #[test]
  fn test_get_variable() {
    let mut env = Environment::new();
    env.define("foo", Literal::String("Foo".to_string()));
    let foo_tok = Token::new(TokenType::Identifier, "foo", None, 0);
    assert!(matches!(env.get(&foo_tok), Ok(Literal::String(s)) if s == "Foo"));
  }

  #[test]
  fn test_get_undefined_variable() {
    let env = Environment::new();
    let foo_tok = Token::new(TokenType::Identifier, "foo", None, 0);
    assert!(env.get(&foo_tok).is_err());
  }

  #[test]
  fn test_assign_to_undefined_variable() {
    let mut env = Environment::new();
    let foo_tok = Token::new(TokenType::Identifier, "foo", None, 0);
    assert!(env.assign(&foo_tok, &Literal::Nil).is_err());
  }

  #[test]
  fn test_reassign_to_defined_variable() {
    let mut env = Environment::new();
    let foo_tok = Token::new(TokenType::Identifier, "foo", None, 0);
    env.define("foo", Literal::Number(73.1));
    assert!(env.assign(&foo_tok, &Literal::Number(89.5)).is_ok());
    assert!(matches!(env.get(&foo_tok), Ok(Literal::Number(n)) if n == 89.5));
  }

  #[test]
  fn test_new_enclosed_environment() {
    let enc = Rc::new(RefCell::new(Environment::new()));
    let env = Environment::new_with_enclosing(&enc.clone());
    assert_eq!(env.enclosing.unwrap().borrow().values, enc.borrow().values);
  }

  #[test]
  fn test_get_from_enclosed_environment() {
    let enc = Rc::new(RefCell::new(Environment::new()));
    enc.borrow_mut().define("foo", Literal::Number(77.8));
    let env = Environment::new_with_enclosing(&enc.clone());
    let foo_tok = Token::new(TokenType::Identifier, "foo", None, 0);
    assert!(matches!(env.get(&foo_tok), Ok(Literal::Number(n)) if n == 77.8))
  }

  #[test]
  fn test_assign_to_enclosed_environment() {
    let enc = Rc::new(RefCell::new(Environment::new()));
    enc.borrow_mut().define("foo", Literal::Number(77.8));
    let mut env = Environment::new_with_enclosing(&enc.clone());
    let foo_tok = Token::new(TokenType::Identifier, "foo", None, 0);
    assert!(env.assign(&foo_tok, &Literal::Number(89.5)).is_ok());
    assert!(matches!(env.get(&foo_tok), Ok(Literal::Number(n)) if n == 89.5));
  }
}
