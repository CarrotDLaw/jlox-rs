use crate::token::*;

#[derive(Debug)]
pub struct LoxError {
  line: usize,
  message: String,
}

impl LoxError {
  fn new(line: usize, message: &str) -> LoxError {
    LoxError {
      line,
      message: message.to_string(),
    }
  }

  pub fn error(line: usize, message: &str) -> LoxError {
    let err = LoxError::new(line, message);
    err.report("");
    err
  }

  pub fn parse_error(token: &Token, message: &str) -> LoxError {
    let err = LoxError::new(token.get_line(), message);
    let location = if token.is_type(&TokenType::Eof) {
      " at end".to_string()
    } else {
      format!(" at '{}'", token.get_lexeme())
    };
    err.report(&location);
    err
  }

  pub fn report(&self, location: &str) {
    eprintln!("[line {}] Error{}: {}", self.line, location, self.message);
  }
}
