use crate::token::*;

#[derive(Debug, Clone)]
pub struct LoxError(LoxErrorType);

#[derive(Debug, Clone)]
enum LoxErrorType {
  Break,
  General { line: usize, message: String },
  Parse { token: Token, message: String },
  Return { value: Literal },
  Runtime { token: Token, message: String },
  System { message: String },
  Type,
}

impl LoxError {
  pub fn new_break() -> LoxError {
    LoxError(LoxErrorType::Break)
  }

  pub fn is_break(&self) -> bool {
    if matches!(self.0, LoxErrorType::Break) {
      return true;
    }

    false
  }

  pub fn general_error(line: usize, message: &str) -> LoxError {
    let err = LoxError(LoxErrorType::General {
      line,
      message: message.to_string(),
    });
    err.report();
    err
  }

  pub fn parse_error(token: &Token, message: &str) -> LoxError {
    let err = LoxError(LoxErrorType::Parse {
      token: token.clone(),
      message: message.to_string(),
    });
    err.report();
    err
  }

  pub fn new_parse_failure() -> LoxError {
    LoxError(LoxErrorType::Parse {
      token: Token::new_eof(0),
      message: "".to_string(),
    })
  }

  pub fn new_return(literal: &Literal) -> LoxError {
    LoxError(LoxErrorType::Return {
      value: literal.clone(),
    })
  }

  pub fn runtime_error(token: &Token, message: &str) -> LoxError {
    let err = LoxError(LoxErrorType::Runtime {
      token: token.clone(),
      message: message.to_string(),
    });
    err.report();
    err
  }

  pub fn system_error(message: &str) -> LoxError {
    let err = LoxError(LoxErrorType::System {
      message: message.to_string(),
    });
    err.report();
    err
  }

  pub fn new_type_error() -> LoxError {
    LoxError(LoxErrorType::Type)
  }

  pub fn is_return(&self) -> bool {
    if matches!(self.0, LoxErrorType::Return { .. }) {
      return true;
    }

    false
  }

  pub fn is_runtime_error(&self) -> bool {
    if matches!(self.0, LoxErrorType::Runtime { .. }) {
      return true;
    }

    false
  }

  pub fn get_return_value(&self) -> Result<Literal, LoxError> {
    if let LoxError(LoxErrorType::Return { value }) = self {
      return Ok(value.clone());
    }

    Err(self.clone())
  }

  fn report(&self) {
    match self {
      LoxError(LoxErrorType::General { line, message }) => {
        eprintln!("[line {line}] Error: {message}")
      }
      LoxError(LoxErrorType::Parse { token, message }) => {
        let line = token.get_line();
        let location = if token.is_type(&TokenType::Eof) {
          "end".to_string()
        } else {
          format!("'{}'", token.get_lexeme())
        };

        eprintln!("[line {line}] Error at {location}: {message}");
      }
      LoxError(LoxErrorType::Runtime { token, message }) => {
        let line = token.get_line();

        eprintln!("{message}");
        eprintln!("[line {line}]");
      }
      LoxError(LoxErrorType::System { message }) => eprintln!("{message}"),
      _ => (),
    }
  }
}
