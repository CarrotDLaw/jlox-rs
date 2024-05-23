use crate::token::*;

#[derive(Debug, Clone)]
pub struct LoxError(LoxErrorType);

#[derive(Debug, Clone)]
enum LoxErrorType {
  Break,
  GeneralErr { line: usize, message: String },
  ParseErr { token: Token, message: String },
  Return { value: Literal },
  RuntimeErr { token: Token, message: String },
  SystemErr { message: String },
  TypeErr,
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
    let err = LoxError(LoxErrorType::GeneralErr {
      line,
      message: message.to_string(),
    });
    err.report();
    err
  }

  pub fn parse_error(token: &Token, message: &str) -> LoxError {
    let err = LoxError(LoxErrorType::ParseErr {
      token: token.clone(),
      message: message.to_string(),
    });
    err.report();
    err
  }

  pub fn new_parse_failure() -> LoxError {
    LoxError(LoxErrorType::ParseErr {
      token: Token::new_eof(0),
      message: String::new(),
    })
  }

  pub fn new_return(literal: &Literal) -> LoxError {
    LoxError(LoxErrorType::Return {
      value: literal.clone(),
    })
  }

  pub fn runtime_error(token: &Token, message: &str) -> LoxError {
    let err = LoxError(LoxErrorType::RuntimeErr {
      token: token.clone(),
      message: message.to_string(),
    });
    err.report();
    err
  }

  pub fn system_error(message: &str) -> LoxError {
    let err = LoxError(LoxErrorType::SystemErr {
      message: message.to_string(),
    });
    err.report();
    err
  }

  pub fn new_type_error() -> LoxError {
    LoxError(LoxErrorType::TypeErr)
  }

  pub fn is_return(&self) -> bool {
    if matches!(self.0, LoxErrorType::Return { .. }) {
      return true;
    }

    false
  }

  pub fn is_runtime_error(&self) -> bool {
    if matches!(self.0, LoxErrorType::RuntimeErr { .. }) {
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
      LoxError(LoxErrorType::GeneralErr { line, message }) => {
        eprintln!("[line {line}] Error: {message}");
      }
      LoxError(LoxErrorType::ParseErr { token, message }) => {
        let line = token.get_line();
        let location = if token.is_type(&TokenType::Eof) {
          "end".to_string()
        } else {
          format!("'{}'", token.get_lexeme())
        };

        eprintln!("[line {line}] Error at {location}: {message}");
      }
      LoxError(LoxErrorType::RuntimeErr { token, message }) => {
        let line = token.get_line();

        eprintln!("{message}");
        eprintln!("[line {line}]");
      }
      LoxError(LoxErrorType::SystemErr { message }) => eprintln!("{message}"),
      _ => (),
    }
  }
}
