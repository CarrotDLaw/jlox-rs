use std::{fmt, rc::Rc};

use crate::{error::*, interpreter::*, lox_class::*, token::*};

#[derive(Clone)]
pub struct Callable {
  pub fun: Rc<dyn LoxCallable>,
}

pub trait LoxCallable {
  fn arity(&self) -> u8;
  fn call(
    &self,
    interpreter: &Interpreter,
    arguments: &[Literal],
    class: Option<Rc<LoxClass>>,
  ) -> Result<Literal, LoxError>;
}

impl fmt::Debug for Callable {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "<DEBUG CALLABLE>")
  }
}

impl fmt::Display for Callable {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "<callable>")
  }
}

impl PartialEq for Callable {
  fn eq(&self, other: &Self) -> bool {
    Rc::ptr_eq(&self.fun, &other.fun)
  }
}
