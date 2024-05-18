use std::{
  env::args,
  fs::read_to_string,
  io::{self, stdin, stdout, Write},
  process::exit,
  rc::Rc,
};

use jlox::{error::*, interpreter::*, parser::*, scanner::*, stmt::*};

fn main() {
  let args = args().collect::<Vec<String>>();
  let lox = Lox::new();

  match args.len() {
    1 => lox.run_prompt(),
    2 => lox
      .run_file(args.get(1).expect("Error reading input."))
      .expect("Error opening file."),
    _ => {
      eprintln!("Usage: jlox [script]");
      exit(64);
    }
  }
}

struct Lox {
  interpreter: Interpreter,
}

impl Lox {
  fn new() -> Lox {
    Lox {
      interpreter: Interpreter::new(),
    }
  }

  fn run_file(&self, path: &str) -> io::Result<()> {
    let bytes = read_to_string(path)?;
    if self.run(&bytes).is_err() {
      exit(65);
    }

    Ok(())
  }

  fn run_prompt(&self) {
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

  fn run(&self, source: &str) -> Result<(), LoxError> {
    if source.trim() == "@" {
      self.interpreter.print_environment();
      return Ok(());
    }

    let mut scanner = Scanner::new(source);
    let tokens = scanner.scan_tokens()?;
    let mut parser = Parser::new(tokens);
    let statements = parser.parse()?;
    if !parser.success() {
      return Err(LoxError::ParseFailure);
    }

    self.interpreter.interpret(
      &statements
        .into_iter()
        .map(Rc::new)
        .collect::<Vec<Rc<Stmt>>>(),
    )?;

    Ok(())
  }
}
