use std::{cell::RefCell, fmt, rc::Rc};

use crate::{environment::*, error::*, interpreter::*, lox_callable::*, stmt::*, token::*};

pub struct LoxFunction {
  closure: Rc<RefCell<Environment>>,
  name: Token,
  params: Rc<Vec<Token>>,
  body: Rc<Vec<Rc<Stmt>>>,
}

impl LoxFunction {
  pub fn new(closure: &Rc<RefCell<Environment>>, declaration: &FunctionStmt) -> LoxFunction {
    LoxFunction {
      closure: closure.clone(),
      name: declaration.name.clone(),
      params: declaration.params.clone().into(),
      body: declaration.body.clone(),
    }
  }
}

impl LoxCallable for LoxFunction {
  fn arity(&self) -> u8 {
    self.params.len() as u8
  }

  fn call(&self, interpreter: &Interpreter, arguments: &[Literal]) -> Result<Literal, LoxError> {
    let mut environment = Environment::new_with_enclosing(&self.closure.clone());

    for (param, arg) in self.params.iter().zip(arguments.iter()) {
      environment.define(param.get_lexeme(), arg.clone());
    }

    if let Err(v) = interpreter.execute_block(&self.body.as_slice().into(), environment) {
      return v.get_return_value();
    }

    Ok(Literal::Nil)
  }
}

impl fmt::Display for LoxFunction {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "<fn {}>", self.name.get_lexeme())
  }
}
