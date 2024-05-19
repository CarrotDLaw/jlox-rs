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

    if self.is_match(&[&TokenType::If]) {
      return self.if_statement();
    }

    if self.is_match(&[&TokenType::LeftBrace]) {
      return Ok(Stmt::Block(
        BlockStmt {
          statements: self.block()?.into_iter().map(Rc::new).collect(),
        }
        .into(),
      ));
    }

    self.expression_statement()
  }

  fn if_statement(&mut self) -> Result<Stmt, LoxError> {
    self.consume(&TokenType::LeftParen, "Expect '(' after 'if'.")?;
    let condition = self.expression()?.into();
    self.consume(&TokenType::RightParen, "Expect ')' after if condition.")?;

    let then_branch = self.statement()?.into();
    let else_branch = if self.is_match(&[&TokenType::Else]) {
      Some(self.statement()?.into())
    } else {
      None
    };

    Ok(Stmt::If(
      IfStmt {
        condition,
        then_branch,
        else_branch,
      }
      .into(),
    ))
  }

  fn print_statement(&mut self) -> Result<Stmt, LoxError> {
    let value = self.expression()?;
    self.consume(&TokenType::Semicolon, "Expect ';' after value.")?;
    Ok(Stmt::Print(
      PrintStmt {
        expression: value.into(),
      }
      .into(),
    ))
  }

  fn var_declaration(&mut self) -> Result<Stmt, LoxError> {
    let name = self
      .consume(&TokenType::Identifier, "Expect variable name.")?
      .clone();

    let initialiser = if self.is_match(&[&TokenType::Assign]) {
      Some(self.expression()?.into())
    } else {
      None
    };

    self.consume(
      &TokenType::Semicolon,
      "Expect ';' after variable declaration.",
    )?;
    Ok(Stmt::Var(VarStmt { name, initialiser }.into()))
  }

  fn expression_statement(&mut self) -> Result<Stmt, LoxError> {
    let value = self.expression()?;
    self.consume(&TokenType::Semicolon, "Expect ';' after expression.")?;
    Ok(Stmt::Expression(
      ExpressionStmt {
        expression: value.into(),
      }
      .into(),
    ))
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
    let expr = self.or()?;

    if self.is_match(&[&TokenType::Assign]) {
      let equals = self.previous().clone();
      let value = self.assignment()?;

      if let Expr::Variable(v) = expr {
        return Ok(Expr::Assign(
          AssignExpr {
            name: v.name.clone(),
            value: value.into(),
          }
          .into(),
        ));
      }

      self.had_error = true;
      LoxError::parse_error(&equals, "Invalid assignment target.");
    }

    Ok(expr)
  }

  fn or(&mut self) -> Result<Expr, LoxError> {
    let mut expr = self.and()?;

    while self.is_match(&[&TokenType::Or]) {
      expr = Expr::Logical(
        LogicalExpr {
          left: expr.into(),
          operator: self.previous().clone(),
          right: self.and()?.into(),
        }
        .into(),
      )
    }

    Ok(expr)
  }

  fn and(&mut self) -> Result<Expr, LoxError> {
    let mut expr = self.equality()?;

    while self.is_match(&[&TokenType::And]) {
      expr = Expr::Logical(
        LogicalExpr {
          left: expr.into(),
          operator: self.previous().clone(),
          right: self.equality()?.into(),
        }
        .into(),
      )
    }

    Ok(expr)
  }

  fn equality(&mut self) -> Result<Expr, LoxError> {
    let mut expr = self.comparison()?;

    while self.is_match(&[&TokenType::BangEqual, &TokenType::EqualEqual]) {
      expr = Expr::Binary(
        BinaryExpr {
          left: expr.into(),
          operator: self.previous().clone(),
          right: self.comparison()?.into(),
        }
        .into(),
      );
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
      expr = Expr::Binary(
        BinaryExpr {
          left: expr.into(),
          operator: self.previous().clone(),
          right: self.term()?.into(),
        }
        .into(),
      );
    }

    Ok(expr)
  }

  fn term(&mut self) -> Result<Expr, LoxError> {
    let mut expr = self.factor()?;

    while self.is_match(&[&TokenType::Minus, &TokenType::Plus]) {
      expr = Expr::Binary(
        BinaryExpr {
          left: expr.into(),
          operator: self.previous().clone(),
          right: self.term()?.into(),
        }
        .into(),
      );
    }

    Ok(expr)
  }

  fn factor(&mut self) -> Result<Expr, LoxError> {
    let mut expr = self.unary()?;

    while self.is_match(&[&TokenType::Slash, &TokenType::Star]) {
      expr = Expr::Binary(
        BinaryExpr {
          left: expr.into(),
          operator: self.previous().clone(),
          right: self.unary()?.into(),
        }
        .into(),
      );
    }

    Ok(expr)
  }

  fn unary(&mut self) -> Result<Expr, LoxError> {
    if self.is_match(&[&TokenType::Bang, &TokenType::Minus]) {
      return Ok(Expr::Unary(
        UnaryExpr {
          operator: self.previous().clone(),
          right: self.unary()?.into(),
        }
        .into(),
      ));
    }

    self.primary()
  }

  fn primary(&mut self) -> Result<Expr, LoxError> {
    if self.is_match(&[&TokenType::False]) {
      return Ok(Expr::Literal(
        LiteralExpr {
          value: Some(Object::Boolean(false)),
        }
        .into(),
      ));
    }

    if self.is_match(&[&TokenType::True]) {
      return Ok(Expr::Literal(
        LiteralExpr {
          value: Some(Object::Boolean(true)),
        }
        .into(),
      ));
    }

    if self.is_match(&[&TokenType::Nil]) {
      return Ok(Expr::Literal(
        LiteralExpr {
          value: Some(Object::Nil),
        }
        .into(),
      ));
    }

    if self.is_match(&[&TokenType::Number, &TokenType::String]) {
      return Ok(Expr::Literal(
        LiteralExpr {
          value: self.previous().get_literal().clone(),
        }
        .into(),
      ));
    }

    if self.is_match(&[&TokenType::Identifier]) {
      return Ok(Expr::Variable(
        VariableExpr {
          name: self.previous().clone(),
        }
        .into(),
      ));
    }

    if self.is_match(&[&TokenType::LeftParen]) {
      let expr = self.expression()?;
      self.consume(&TokenType::RightParen, "Expect ')' after expression.")?;
      return Ok(Expr::Grouping(
        GroupingExpr {
          expression: expr.into(),
        }
        .into(),
      ));
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
