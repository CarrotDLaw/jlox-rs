use crate::token::*;

#[derive(Debug)]
pub enum LoxError {
  GeneralError { line: usize, message: String },
  ParseError { token: Token, message: String },
  ParseFailure,
  RuntimeError { token: Token, message: String },
  TypeError,
}

impl LoxError {
  pub fn general_error(line: usize, message: &str) -> LoxError {
    let err = LoxError::GeneralError {
      line,
      message: message.to_string(),
    };
    err.report();
    err
  }

  pub fn parse_error(token: &Token, message: &str) -> LoxError {
    let err = LoxError::ParseError {
      token: token.clone(),
      message: message.to_string(),
    };
    err.report();
    err
  }

  pub fn runtime_error(token: &Token, message: &str) -> LoxError {
    let err = LoxError::RuntimeError {
      token: token.clone(),
      message: message.to_string(),
    };
    err.report();
    err
  }

  fn report(&self) {
    match self {
      LoxError::GeneralError { line, message } => {
        eprintln!("[line {line}] Error: {message}")
      }
      LoxError::ParseError { token, message } => {
        let line = token.get_line();
        let location = if token.is_type(&TokenType::Eof) {
          "end".to_string()
        } else {
          format!("'{}'", token.get_lexeme())
        };

        eprintln!("[line {line}] Error at {location}: {message}");
      }
      LoxError::RuntimeError { token, message } => {
        let line = token.get_line();

        eprintln!("{message}");
        eprintln!("[line {line}]");
      }
      _ => (),
    }
  }
}
