use std::{
  fs::read_to_string,
  io::{self, stdin, stdout, Write},
  process::exit,
  rc::Rc,
};

use crate::{error::*, interpreter::*, parser::*, resolver::Resolver, scanner::*, stmt::*};

#[derive(Default)]
pub struct Lox {
  interpreter: Interpreter,
}

impl Lox {
  pub fn new() -> Lox {
    Lox {
      interpreter: Interpreter::new(),
    }
  }

  pub fn run_file(&self, path: &str) -> io::Result<()> {
    let bytes = read_to_string(path)?;

    if let Err(e) = self.run(&bytes) {
      if e.is_runtime_error() {
        exit(70);
      }

      if !e.is_runtime_error() {
        exit(65);
      }
    }

    exit(0)
  }

  pub fn run_prompt(&self) {
    loop {
      print!("> ");
      if stdout().flush().is_err() {
        break;
      }

      let mut line = String::new();
      if stdin().read_line(&mut line).is_err() || line.is_empty() {
        break;
      }

      self.run(&line).ok();
    }
  }

  pub fn run(&self, source: &str) -> Result<(), LoxError> {
    if source.trim().eq("!") {
      exit(0);
    }

    if source.trim().eq("@") {
      self.interpreter.print_environment();
      return Ok(());
    }

    let mut scanner = Scanner::new(source);
    let tokens = scanner.scan_tokens()?;
    let mut parser = Parser::new(tokens);

    let statements = parser.parse()?;
    let statements = statements
      .into_iter()
      .map(Rc::new)
      .collect::<Vec<Rc<Stmt>>>();
    let statements = Rc::new(statements.as_slice());

    let resolver = Resolver::new(&self.interpreter);
    resolver.resolve(&statements.clone())?;

    self.interpreter.interpret(&statements.clone())?;

    Ok(())
  }
}
