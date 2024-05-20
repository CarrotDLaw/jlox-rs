use std::time::SystemTime;

use crate::{error::*, interpreter::*, lox_callable::*, token::*};

pub struct Clock;

impl LoxCallable for Clock {
  fn arity(&self) -> u8 {
    0
  }

  fn call(&self, _interpreter: &Interpreter, _arguments: &[Literal]) -> Result<Literal, LoxError> {
    if let Ok(d) = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
      return Ok(Literal::Number(d.as_secs_f64()));
    }

    Err(LoxError::system_error("SYSTEM CLOCK ERROR."))
  }
}
