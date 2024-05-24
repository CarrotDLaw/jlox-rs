use std::{collections::HashMap, fmt, rc::Rc};

use crate::{error::*, interpreter::*, lox_callable::*, lox_instance::*, token::*};

#[derive(Debug, Clone, PartialEq)]
pub struct LoxClass {
  superclass: Option<Rc<LoxClass>>,
  name: String,
  methods: HashMap<String, Literal>,
}

impl LoxClass {
  pub fn new(
    name: &str,
    superclass: &Option<Rc<LoxClass>>,
    methods: &HashMap<String, Literal>,
  ) -> LoxClass {
    LoxClass {
      superclass: superclass.clone(),
      name: name.to_string(),
      methods: methods.clone(),
    }
  }

  pub fn find_method(&self, name: &str) -> Option<Literal> {
    if let Some(m) = self.methods.get(name) {
      return Some(m.clone());
    }

    if let Some(s) = &self.superclass {
      return s.find_method(name);
    }

    None
  }
}

impl LoxCallable for LoxClass {
  fn arity(&self) -> u8 {
    if let Some(Literal::Function(initialiser)) = self.find_method("init") {
      return initialiser.arity();
    }

    0
  }

  fn call(
    &self,
    interpreter: &Interpreter,
    arguments: &[Literal],
    class: Option<Rc<LoxClass>>,
  ) -> Result<Literal, LoxError> {
    let instance = Literal::Instance(LoxInstance::new(&self.clone().into()).into());

    if let Some(Literal::Function(initialiser)) = self.find_method("init") {
      if let Literal::Function(m) = initialiser.bind(&instance) {
        m.call(interpreter, arguments, class)?;
      }
    }

    Ok(instance)
  }
}

impl fmt::Display for LoxClass {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.name)
  }
}
