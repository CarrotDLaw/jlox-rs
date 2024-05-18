use std::rc::Rc;

use crate::{error::*, expr::*, stmt::*, token::*};

pub struct Parser<'a> {
  tokens: &'a [Token],
  current: usize,
  had_error: bool,
}

impl<'a> Parser<'a> {
  pub fn new(tokens: &[Token]) -> Parser {
    Parser {
      tokens,
      current: 0,
      had_error: false,
    }
  }

  pub fn success(&self) -> bool {
    !self.had_error
  }

  pub fn parse(&mut self) -> Result<Vec<Stmt>, LoxError> {
    let mut statements = Vec::new();

    while !self.is_at_end() {
      statements.push(self.declaration()?)
    }

    Ok(statements)
  }

  fn expression(&mut self) -> Result<Expr, LoxError> {
    self.assignment()
  }

  fn declaration(&mut self) -> Result<Stmt, LoxError> {
    let res = if self.is_match(&[&TokenType::Var]) {
      self.var_declaration()
    } else {
      self.statement()
    };

    if res.is_err() {
      self.synchronise();
    }

    res
  }

  fn statement(&mut self) -> Result<Stmt, LoxError> {
    if self.is_match(&[&TokenType::Print]) {
      return self.print_statement();
    }

    if self.is_match(&[&TokenType::LeftBrace]) {
      return Ok(Stmt::Block(Rc::new(BlockStmt {
        statements: self.block()?.into_iter().map(Rc::new).collect(),
      })));
    }

    self.expression_statement()
  }

  fn print_statement(&mut self) -> Result<Stmt, LoxError> {
    let value = self.expression()?;
    self.consume(&TokenType::Semicolon, "Expect ';' after value.")?;
    Ok(Stmt::Print(Rc::new(PrintStmt {
      expression: Rc::new(value),
    })))
  }

  fn var_declaration(&mut self) -> Result<Stmt, LoxError> {
    let name = self
      .consume(&TokenType::Identifier, "Expect variable name.")?
      .clone();

    let initialiser = if self.is_match(&[&TokenType::Assign]) {
      Some(Rc::new(self.expression()?))
    } else {
      None
    };

    self.consume(
      &TokenType::Semicolon,
      "Expect ';' after variable declaration.",
    )?;
    Ok(Stmt::Var(Rc::new(VarStmt { name, initialiser })))
  }

  fn expression_statement(&mut self) -> Result<Stmt, LoxError> {
    let value = self.expression()?;
    self.consume(&TokenType::Semicolon, "Expect ';' after expression.")?;
    Ok(Stmt::Expression(Rc::new(ExpressionStmt {
      expression: Rc::new(value),
    })))
  }

  fn block(&mut self) -> Result<Vec<Stmt>, LoxError> {
    let mut statements = Vec::new();

    while !self.check(&TokenType::RightBrace) && !self.is_at_end() {
      statements.push(self.declaration()?)
    }

    self.consume(&TokenType::RightBrace, "Expect '}' after block.")?;
    Ok(statements)
  }

  fn assignment(&mut self) -> Result<Expr, LoxError> {
    let expr = self.equality()?;

    if self.is_match(&[&TokenType::Assign]) {
      let equals = self.previous().clone();
      let value = self.assignment()?;

      if let Expr::Variable(v) = expr {
        return Ok(Expr::Assign(Rc::new(AssignExpr {
          name: v.name.clone(),
          value: Rc::new(value),
        })));
      }

      self.had_error = true;
      LoxError::parse_error(&equals, "Invalid assignment target.");
    }

    Ok(expr)
  }

  fn equality(&mut self) -> Result<Expr, LoxError> {
    let mut expr = self.comparison()?;

    while self.is_match(&[&TokenType::BangEqual, &TokenType::EqualEqual]) {
      expr = Expr::Binary(Rc::new(BinaryExpr {
        left: Rc::new(expr),
        operator: self.previous().clone(),
        right: Rc::new(self.comparison()?),
      }));
    }

    Ok(expr)
  }

  fn comparison(&mut self) -> Result<Expr, LoxError> {
    let mut expr = self.term()?;

    while self.is_match(&[
      &TokenType::Greater,
      &TokenType::GreaterEqual,
      &TokenType::Less,
      &TokenType::LessEqual,
    ]) {
      expr = Expr::Binary(Rc::new(BinaryExpr {
        left: Rc::new(expr),
        operator: self.previous().clone(),
        right: Rc::new(self.term()?),
      }));
    }

    Ok(expr)
  }

  fn term(&mut self) -> Result<Expr, LoxError> {
    let mut expr = self.factor()?;

    while self.is_match(&[&TokenType::Minus, &TokenType::Plus]) {
      expr = Expr::Binary(Rc::new(BinaryExpr {
        left: Rc::new(expr),
        operator: self.previous().clone(),
        right: Rc::new(self.term()?),
      }));
    }

    Ok(expr)
  }

  fn factor(&mut self) -> Result<Expr, LoxError> {
    let mut expr = self.unary()?;

    while self.is_match(&[&TokenType::Slash, &TokenType::Star]) {
      expr = Expr::Binary(Rc::new(BinaryExpr {
        left: Rc::new(expr),
        operator: self.previous().clone(),
        right: Rc::new(self.unary()?),
      }));
    }

    Ok(expr)
  }

  fn unary(&mut self) -> Result<Expr, LoxError> {
    if self.is_match(&[&TokenType::Bang, &TokenType::Minus]) {
      return Ok(Expr::Unary(Rc::new(UnaryExpr {
        operator: self.previous().clone(),
        right: Rc::new(self.unary()?),
      })));
    }

    self.primary()
  }

  fn primary(&mut self) -> Result<Expr, LoxError> {
    if self.is_match(&[&TokenType::False]) {
      return Ok(Expr::Literal(Rc::new(LiteralExpr {
        value: Some(Object::Boolean(false)),
      })));
    }

    if self.is_match(&[&TokenType::True]) {
      return Ok(Expr::Literal(Rc::new(LiteralExpr {
        value: Some(Object::Boolean(true)),
      })));
    }

    if self.is_match(&[&TokenType::Nil]) {
      return Ok(Expr::Literal(Rc::new(LiteralExpr {
        value: Some(Object::Nil),
      })));
    }

    if self.is_match(&[&TokenType::Number, &TokenType::String]) {
      return Ok(Expr::Literal(Rc::new(LiteralExpr {
        value: self.previous().get_literal().clone(),
      })));
    }

    if self.is_match(&[&TokenType::Identifier]) {
      return Ok(Expr::Variable(Rc::new(VariableExpr {
        name: self.previous().clone(),
      })));
    }

    if self.is_match(&[&TokenType::LeftParen]) {
      let expr = self.expression()?;
      self.consume(&TokenType::RightParen, "Expect ')' after expression.")?;
      return Ok(Expr::Grouping(Rc::new(GroupingExpr {
        expression: Rc::new(expr),
      })));
    }

    Err(LoxError::parse_error(self.peek(), "Expect expression."))
  }

  fn is_match(&mut self, token_types: &[&TokenType]) -> bool {
    for token_type in token_types {
      if self.check(token_type) {
        self.advance();
        return true;
      }
    }

    false
  }

  fn consume(&mut self, token_type: &TokenType, message: &str) -> Result<&Token, LoxError> {
    if self.check(token_type) {
      return Ok(self.advance());
    }

    Err(LoxError::parse_error(self.peek(), message))
  }

  fn check(&self, token_type: &TokenType) -> bool {
    if self.is_at_end() {
      return false;
    }

    return self.peek().is_type(token_type);
  }

  fn advance(&mut self) -> &Token {
    if !self.is_at_end() {
      self.current += 1;
    }

    self.previous()
  }

  fn is_at_end(&self) -> bool {
    self.peek().is_type(&TokenType::Eof)
  }

  fn peek(&self) -> &Token {
    if let Some(t) = self.tokens.get(self.current) {
      return t;
    }

    unreachable!()
  }

  fn previous(&self) -> &Token {
    if let Some(t) = self.tokens.get(self.current - 1) {
      return t;
    }

    unreachable!()
  }

  fn synchronise(&mut self) {
    self.advance();

    while !self.is_at_end() {
      if self.previous().is_type(&TokenType::Semicolon) {
        return;
      }

      if self.peek().is_types(&[
        &TokenType::Class,
        &TokenType::Fun,
        &TokenType::Var,
        &TokenType::For,
        &TokenType::If,
        &TokenType::While,
        &TokenType::Print,
        &TokenType::Return,
      ]) {
        return;
      }

      self.advance();
    }
  }
}
