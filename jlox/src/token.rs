use std::fmt;

// #[derive(Debug)]
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

  pub fn add_eof(line: usize) -> Token {
    Token {
      token_type: TokenType::Eof,
      lexeme: "".to_string(),
      literal: None,
      line,
    }
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

// #[derive(Debug)]
pub enum Object {
  Number(f64),
  String(String),
  Boolean(bool),
  Nil,
}

impl fmt::Display for Object {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      Object::Number(n) => write!(f, "{n}"),
      Object::String(s) => write!(f, "\"{s}\""),
      Object::Boolean(b) => write!(f, "{b}"),
      Object::Nil => write!(f, "nil"),
    }
  }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TokenType {
  LeftBracket,
  RightBracket,
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
