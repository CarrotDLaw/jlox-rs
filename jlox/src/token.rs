use std::fmt;

use crate::error::*;

#[derive(Debug, Clone)]
pub struct Token {
  token_type: TokenType,
  lexeme: String,
  literal: Option<Object>,
  line: usize,
}

impl Token {
  pub fn new(token_type: TokenType, lexeme: &str, literal: Option<Object>, line: usize) -> Token {
    Token {
      token_type,
      lexeme: lexeme.to_string(),
      literal,
      line,
    }
  }

  pub fn new_eof(line: usize) -> Token {
    Token {
      token_type: TokenType::Eof,
      lexeme: "".to_string(),
      literal: None,
      line,
    }
  }

  pub fn get_type(&self) -> &TokenType {
    &self.token_type
  }

  pub fn get_lexeme(&self) -> &String {
    &self.lexeme
  }

  pub fn get_literal(&self) -> &Option<Object> {
    &self.literal
  }

  pub fn get_line(&self) -> usize {
    self.line
  }

  pub fn is_type(&self, token_type: &TokenType) -> bool {
    &self.token_type == token_type
  }

  pub fn is_types(&self, token_types: &[&TokenType]) -> bool {
    for token_type in token_types {
      if &&self.token_type == token_type {
        return true;
      }
    }

    false
  }
}

impl fmt::Display for Token {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(
      f,
      "{:?} {} {}",
      self.token_type,
      self.lexeme,
      if let Some(literal) = &self.literal {
        literal.to_string()
      } else {
        "None".to_string()
      }
    )
  }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Object {
  Number(f64),
  String(String),
  Boolean(bool),
  Nil,
}

impl Object {
  pub fn get_number(&self) -> Result<f64, LoxError> {
    if let &Object::Number(n) = self {
      return Ok(n);
    }

    Err(LoxError::TypeError)
  }
}

impl fmt::Display for Object {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      Object::Number(n) => write!(f, "{n}"),
      Object::String(s) => write!(f, "{s}"),
      Object::Boolean(b) => write!(f, "{b}"),
      Object::Nil => write!(f, "nil"),
    }
  }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
  LeftParen,
  RightParen,
  LeftBrace,
  RightBrace,
  Comma,
  Dot,
  Plus,
  Minus,
  Star,
  Semicolon,
  Eof,
  BangEqual,
  Bang,
  EqualEqual,
  Assign,
  LessEqual,
  Less,
  GreaterEqual,
  Greater,
  Slash,
  String,
  Number,
  Identifier,
  And,
  Class,
  Else,
  False,
  For,
  Fun,
  If,
  Nil,
  Or,
  Print,
  Return,
  Super,
  This,
  True,
  Var,
  While,
}

#[cfg(test)]
mod test {
  use super::*;

  #[test]
  fn test_token_is_type() {
    let token = Token::new(TokenType::Number, "123", Some(Object::Number(123.0)), 1);
    assert!(token.is_type(&TokenType::Number));
    assert!(!token.is_type(&TokenType::Nil));
  }
}
