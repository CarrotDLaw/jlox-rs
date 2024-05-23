use crate::{error::*, token::*};

pub struct Scanner {
  source: Vec<char>,
  tokens: Vec<Token>,
  start: usize,
  current: usize,
  line: usize,
}

impl Scanner {
  pub fn new(source: &str) -> Scanner {
    Scanner {
      source: source.chars().collect(),
      tokens: Vec::new(),
      start: 0,
      current: 0,
      line: 1,
    }
  }

  pub fn scan_tokens(&mut self) -> Result<&Vec<Token>, LoxError> {
    while !self.is_at_end() {
      self.start = self.current;
      self.scan_token()?;
    }

    self.tokens.push(Token::new_eof(self.line));

    Ok(&self.tokens)
  }

  fn is_at_end(&self) -> bool {
    self.current >= self.source.len()
  }

  fn scan_token(&mut self) -> Result<(), LoxError> {
    let c = self.advance();
    match c {
      '(' => self.add_token(TokenType::LeftBracket),
      ')' => self.add_token(TokenType::RightBracket),
      '{' => self.add_token(TokenType::LeftBrace),
      '}' => self.add_token(TokenType::RightBrace),
      ',' => self.add_token(TokenType::Comma),
      '.' => self.add_token(TokenType::Dot),
      '+' => self.add_token(TokenType::Plus),
      '-' => self.add_token(TokenType::Minus),
      '*' => self.add_token(TokenType::Star),
      ';' => self.add_token(TokenType::Semicolon),
      '!' => {
        let token = if self.is_match('=') {
          TokenType::BangEqual
        } else {
          TokenType::Bang
        };
        self.add_token(token);
      }
      '=' => {
        let token = if self.is_match('=') {
          TokenType::EqualEqual
        } else {
          TokenType::Assign
        };
        self.add_token(token);
      }
      '<' => {
        let token = if self.is_match('=') {
          TokenType::LessEqual
        } else {
          TokenType::Less
        };
        self.add_token(token);
      }
      '>' => {
        let token = if self.is_match('=') {
          TokenType::GreaterEqual
        } else {
          TokenType::Greater
        };
        self.add_token(token);
      }
      '/' => {
        if self.is_match('/') {
          while let Some(&c) = self.peek() {
            if c.ne(&'\n') {
              self.advance();
            } else {
              break;
            }
          }
        } else if self.is_match('*') {
          self.comment()?;
        } else {
          self.add_token(TokenType::Slash);
        }
      }
      ' ' | '\r' | '\t' => (),
      '\n' => self.line += 1,
      '"' => self.string()?,
      '0'..='9' => self.number()?,
      _ if c.is_valid_for_lox_identifier() => self.identifier(),
      _ => return Err(LoxError::general_error(self.line, "Unexpected character.")),
    }

    Ok(())
  }

  fn identifier(&mut self) {
    while self
      .peek()
      .is_some_and(CheckCharacter::is_valid_for_lox_identifier)
    {
      self.advance();
    }

    self.add_token(
      self
        .source
        .get(self.start..self.current)
        .map_or_else(String::new, |s| s.iter().collect())
        .match_lox_keyword(),
    );
  }

  fn number(&mut self) -> Result<(), LoxError> {
    while self.peek().is_some_and(char::is_ascii_digit) {
      self.advance();
    }

    if self.peek().eq(&Some(&'.')) && self.peek_next().is_some_and(char::is_ascii_digit) {
      self.advance();

      while self.peek().is_some_and(char::is_ascii_digit) {
        self.advance();
      }
    }

    self.add_token_and_literal(
      TokenType::Number,
      Some(Literal::Number(
        self
          .source
          .get(self.start..self.current)
          .map_or_else(String::new, |s| s.iter().collect())
          .parse::<f64>()
          .map_err(|_| LoxError::general_error(self.line, "COULD NOT PARSE NUMBER."))?,
      )),
    );

    Ok(())
  }

  fn string(&mut self) -> Result<(), LoxError> {
    while let Some(&c) = self.peek() {
      match c {
        '\n' => self.line += 1,
        '"' => break,
        _ => (),
      }
      self.advance();
    }

    if self.is_at_end() {
      return Err(LoxError::general_error(self.line, "Unterminated string."));
    }

    self.advance();

    let value = self
      .source
      .get(self.start + 1..self.current - 1)
      .map_or_else(String::new, |s| s.iter().collect());
    self.add_token_and_literal(TokenType::String, Some(Literal::String(value)));

    Ok(())
  }

  fn comment(&mut self) -> Result<(), LoxError> {
    // consume the '*'
    self.advance();

    let mut counter = 1_usize;
    loop {
      if counter.eq(&0) {
        break;
      }

      match self.peek() {
        Some('/') => {
          self.advance();
          if self.peek().eq(&Some(&'*')) {
            self.advance();
            counter += 1;
          }
        }
        Some('*') => {
          self.advance();
          if self.peek().eq(&Some(&'/')) {
            self.advance();
            counter -= 1;
          }
        }
        Some('\n') => {
          self.advance();
          self.line += 1;
        }
        None => {
          return Err(LoxError::general_error(
            self.line,
            "Unterminated block comment.",
          ));
        }
        _ => {
          self.advance();
        }
      }
    }

    Ok(())
  }

  fn is_match(&mut self, expected: char) -> bool {
    let res = matches!(self.source.get(self.current), Some(&c) if c.eq(&expected));
    if res {
      self.advance();
    }
    res
  }

  fn peek(&self) -> Option<&char> {
    self.source.get(self.current)
  }

  fn peek_next(&self) -> Option<&char> {
    self.source.get(self.current + 1)
  }

  fn advance(&mut self) -> char {
    if let Some(&c) = self.source.get(self.current) {
      self.current += 1;
      return c;
    }

    unreachable!()
  }

  fn add_token(&mut self, token_type: TokenType) {
    self.add_token_and_literal(token_type, None);
  }

  fn add_token_and_literal(&mut self, token_type: TokenType, literal: Option<Literal>) {
    let text = self
      .source
      .get(self.start..self.current)
      .map_or_else(String::new, |s| s.iter().collect());
    self
      .tokens
      .push(Token::new(token_type, &text, literal, self.line));
  }
}

trait CheckCharacter {
  fn is_valid_for_lox_identifier(&self) -> bool;
}

impl CheckCharacter for char {
  fn is_valid_for_lox_identifier(&self) -> bool {
    self.is_ascii_alphanumeric() || self.eq(&'_')
  }
}

trait MatchIdentifier {
  fn match_lox_keyword(&self) -> TokenType;
}

impl MatchIdentifier for str {
  fn match_lox_keyword(&self) -> TokenType {
    match self {
      "and" => TokenType::And,
      "class" => TokenType::Class,
      "else" => TokenType::Else,
      "false" => TokenType::False,
      "for" => TokenType::For,
      "fun" => TokenType::Fun,
      "if" => TokenType::If,
      "nil" => TokenType::Nil,
      "or" => TokenType::Or,
      "print" => TokenType::Print,
      "return" => TokenType::Return,
      "super" => TokenType::Super,
      "this" => TokenType::This,
      "true" => TokenType::True,
      "var" => TokenType::Var,
      "while" => TokenType::While,
      "break" => TokenType::Break,
      _ => TokenType::Identifier,
    }
  }
}

#[cfg(test)]
mod test {
  use super::*;

  #[test]
  fn test_scanner() -> Result<(), LoxError> {
    let source = "-123 * (45.67)";
    let mut scanner = Scanner::new(source);
    let tokens = scanner.scan_tokens()?;

    for token in tokens {
      println!("{token}");
    }

    Ok(())
  }
}
