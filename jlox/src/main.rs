use std::{env::args, process::exit};

use jlox::lox::*;

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
