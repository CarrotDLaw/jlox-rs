use std::{fmt, rc::Rc, time::SystemTime};

use crate::{error::*, interpreter::*, lox_callable::*, lox_class::*, token::*};

#[derive(Clone)]
pub struct LoxNativeFunction {
  pub fun: Rc<dyn LoxCallable>,
}

impl fmt::Debug for LoxNativeFunction {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "<DEBUG NATIVE FN>")
  }
}

impl fmt::Display for LoxNativeFunction {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "<native fn>")
  }
}

impl PartialEq for LoxNativeFunction {
  fn eq(&self, other: &Self) -> bool {
    Rc::ptr_eq(&self.fun, &other.fun)
  }
}

pub struct Clock;

impl LoxCallable for Clock {
  fn arity(&self) -> u8 {
    0
  }

  fn call(
    &self,
    _interpreter: &Interpreter,
    _arguments: &[Literal],
    _class: Option<Rc<LoxClass>>,
  ) -> Result<Literal, LoxError> {
    if let Ok(d) = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
      return Ok(Literal::Number(d.as_secs_f64()));
    }

    Err(LoxError::system_error("SYSTEM CLOCK ERROR."))
  }
}
