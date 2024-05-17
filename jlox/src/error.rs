use crate::token::*;

#[derive(Debug)]
pub enum LoxError {
  GeneralError { line: usize, message: String },
  ParseError { token: Token, message: String },
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
        eprintln!("[line {}] Error: {}", line, message)
      }
      LoxError::ParseError { token, message } => {
        eprintln!(
          "[line {}] Error at {}: {}",
          token.get_line(),
          if token.is_type(&TokenType::Eof) {
            "end".to_string()
          } else {
            format!("'{}'", token.get_lexeme())
          },
          message
        )
      }
      LoxError::RuntimeError { token, message } => {
        eprintln!("{}\n[line {}]", message, token.get_line())
      }
      LoxError::TypeError => (),
    }
  }
}
