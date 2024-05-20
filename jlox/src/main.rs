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
      .run_file(args.get(1).expect("ERROR READING INPUT."))
      .expect("ERROR OPENING FILE."),
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

    self.interpreter.interpret(
      &statements
        .into_iter()
        .map(Rc::new)
        .collect::<Vec<Rc<Stmt>>>()
        .into(),
    )?;

    Ok(())
  }
}
