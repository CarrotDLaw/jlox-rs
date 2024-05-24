use std::{cell::RefCell, fmt, rc::Rc};

use crate::{
  environment::*, error::*, interpreter::*, lox_callable::*, lox_class::*, stmt::*, token::*,
};

#[derive(Debug, Clone)]
pub struct LoxFunction {
  closure: Rc<RefCell<Environment>>,
  name: Token,
  params: Rc<Vec<Token>>,
  body: Rc<Vec<Rc<Stmt>>>,
  is_initialiser: bool,
}

impl LoxFunction {
  pub fn new(
    closure: &Rc<RefCell<Environment>>,
    declaration: &FunctionStmt,
    is_initialiser: bool,
  ) -> LoxFunction {
    LoxFunction {
      closure: closure.clone(),
      name: declaration.name.clone(),
      params: declaration.params.clone().into(),
      body: declaration.body.clone(),
      is_initialiser,
    }
  }

  pub fn bind(&self, instance: &Literal) -> Literal {
    let environment = RefCell::new(Environment::new_with_enclosing(&self.closure));
    environment.borrow_mut().define("this", instance.clone());
    Literal::Function(
      LoxFunction {
        closure: environment.into(),
        name: self.name.clone(),
        params: self.params.clone(),
        body: self.body.clone(),
        is_initialiser: self.is_initialiser,
      }
      .into(),
    )
  }
}

impl LoxCallable for LoxFunction {
  fn arity(&self) -> u8 {
    self.params.len() as u8
  }

  fn call(
    &self,
    interpreter: &Interpreter,
    arguments: &[Literal],
    _class: Option<Rc<LoxClass>>,
  ) -> Result<Literal, LoxError> {
    let mut environment = Environment::new_with_enclosing(&self.closure.clone());

    for (param, arg) in self.params.iter().zip(arguments.iter()) {
      environment.define(param.get_lexeme(), arg.clone());
    }

    if self.is_initialiser {
      return self.closure.borrow().get_at(0, "this");
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

impl PartialEq for LoxFunction {
  fn eq(&self, other: &Self) -> bool {
    self.name.get_lexeme().eq(other.name.get_lexeme())
      && Rc::ptr_eq(&self.closure, &other.closure)
      && Rc::ptr_eq(&self.params, &other.params)
      && Rc::ptr_eq(&self.body, &other.body)
  }
}
