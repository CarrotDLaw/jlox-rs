use std::rc::Rc;

use crate::{error::*, expr::*, stmt::*, token::*};

pub struct Parser<'a> {
  tokens: &'a [Token],
  current: usize,
  loop_depth: usize,
  had_error: bool,
}

impl<'a> Parser<'a> {
  pub fn new(tokens: &[Token]) -> Parser {
    Parser {
      tokens,
      current: 0,
      loop_depth: 0,
      had_error: false,
    }
  }

  pub fn parse(&mut self) -> Result<Vec<Stmt>, LoxError> {
    let mut statements = Vec::new();

    while !self.is_at_end() {
      // if self.declaration() returns error, mark self.had_error as true
      // but do not immediately return
      if let Ok(s) = self.declaration() {
        statements.push(s);
        continue;
      }

      self.had_error = true;
    }

    if !self.had_error {
      return Ok(statements);
    }

    Err(LoxError::new_parse_failure())
  }

  fn expression(&mut self) -> Result<Expr, LoxError> {
    self.assignment()
  }

  fn declaration(&mut self) -> Result<Stmt, LoxError> {
    let res = if self.is_match(&[&TokenType::Class]) {
      self.class_declaration()
    } else if self.is_match(&[&TokenType::Fun]) {
      self.function("function")
    } else if self.is_match(&[&TokenType::Var]) {
      self.var_declaration()
    } else {
      self.statement()
    };

    if res.is_err() {
      self.synchronise();
    }

    res
  }

  fn class_declaration(&mut self) -> Result<Stmt, LoxError> {
    let name = self
      .consume(&TokenType::Identifier, "Expect class name.")?
      .clone();
    let superclass = if self.is_match(&[&TokenType::Less]) {
      self.consume(&TokenType::Identifier, "Expect superclass name.")?;
      Some(Expr::Variable(
        VariableExpr {
          name: self.previous().clone(),
        }
        .into(),
      ))
    } else {
      None
    }
    .map(Rc::new);

    self.consume(&TokenType::LeftBrace, "Expect '{' before class body.")?;

    let mut methods = Vec::new();
    while !self.check(&TokenType::RightBrace) && !self.is_at_end() {
      methods.push(self.function("method")?);
    }
    let methods = methods
      .into_iter()
      .map(Rc::new)
      .collect::<Vec<Rc<Stmt>>>()
      .into();

    self.consume(&TokenType::RightBrace, "Expect '}' after class body.")?;
    Ok(Stmt::Class(
      ClassStmt {
        name,
        methods,
        superclass,
      }
      .into(),
    ))
  }

  fn statement(&mut self) -> Result<Stmt, LoxError> {
    if self.is_match(&[&TokenType::For]) {
      return self.for_statement();
    }

    if self.is_match(&[&TokenType::If]) {
      return self.if_statement();
    }

    if self.is_match(&[&TokenType::Print]) {
      return self.print_statement();
    }

    if self.is_match(&[&TokenType::Return]) {
      return self.return_statement();
    }

    if self.is_match(&[&TokenType::While]) {
      return self.while_statement();
    }

    if self.is_match(&[&TokenType::LeftBrace]) {
      return Ok(Stmt::Block(
        BlockStmt {
          statements: self
            .block()?
            .into_iter()
            .map(Rc::new)
            .collect::<Vec<Rc<Stmt>>>()
            .into(),
        }
        .into(),
      ));
    }

    if self.is_match(&[&TokenType::Break]) {
      return self.break_statement();
    }

    self.expression_statement()
  }

  fn break_statement(&mut self) -> Result<Stmt, LoxError> {
    if self.loop_depth.eq(&0) {
      self.had_error = true;
      LoxError::parse_error(self.previous(), "Must be inside a loop to use 'break'.");
    }
    self.consume(&TokenType::Semicolon, "Expect ';' after 'break'.")?;
    return Ok(Stmt::Break(
      BreakStmt {
        token: self.peek().clone(),
      }
      .into(),
    ));
  }

  fn for_statement(&mut self) -> Result<Stmt, LoxError> {
    self.consume(&TokenType::LeftBracket, "Expect '(' after 'for'.")?;

    let initialiser = if self.is_match(&[&TokenType::Semicolon]) {
      None
    } else if self.is_match(&[&TokenType::Var]) {
      Some(self.var_declaration()?)
    } else {
      Some(self.expression_statement()?)
    };

    let condition = if self.check(&TokenType::Semicolon) {
      None
    } else {
      Some(self.expression()?)
    };

    self.consume(&TokenType::Semicolon, "Expect ';' after loop condition.")?;

    let increment = if self.check(&TokenType::RightBracket) {
      None
    } else {
      Some(self.expression()?)
    };

    self.consume(&TokenType::RightBracket, "Expect ')' after for clauses.")?;

    self.loop_depth += 1;
    let body = self.statement();
    if body.is_err() {
      self.loop_depth -= 1;
      return body;
    }

    let mut body = body?;
    if let Some(i) = increment {
      body = Stmt::Block(
        BlockStmt {
          statements: vec![
            body.into(),
            Stmt::Expression(
              ExpressionStmt {
                expression: i.into(),
              }
              .into(),
            )
            .into(),
          ]
          .into(),
        }
        .into(),
      );
    }

    body = Stmt::While(
      WhileStmt {
        condition: condition.map_or_else(
          || {
            Expr::Literal(
              LiteralExpr {
                value: Some(Literal::Boolean(true)),
              }
              .into(),
            )
            .into()
          },
          |c| c.into(),
        ),
        body: body.into(),
      }
      .into(),
    );

    if let Some(i) = initialiser {
      body = Stmt::Block(
        BlockStmt {
          statements: vec![i.into(), body.into()].into(),
        }
        .into(),
      );
    }

    self.loop_depth -= 1;
    Ok(body)
  }

  fn if_statement(&mut self) -> Result<Stmt, LoxError> {
    self.consume(&TokenType::LeftBracket, "Expect '(' after 'if'.")?;
    let condition = self.expression()?.into();
    self.consume(&TokenType::RightBracket, "Expect ')' after if condition.")?;

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

  fn return_statement(&mut self) -> Result<Stmt, LoxError> {
    let keyword = self.previous().clone();
    let value = if self.check(&TokenType::Semicolon) {
      None
    } else {
      Some(self.expression()?.into())
    };

    self.consume(&TokenType::Semicolon, "Expect ';' after return value.")?;
    Ok(Stmt::Return(ReturnStmt { keyword, value }.into()))
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

  fn while_statement(&mut self) -> Result<Stmt, LoxError> {
    self.consume(&TokenType::LeftBracket, "Expect '(' after 'while'.")?;
    let condition = self.expression()?;
    self.consume(&TokenType::RightBracket, "Expect ')' after condition.")?;

    self.loop_depth += 1;
    let body = self.statement();
    if body.is_err() {
      self.loop_depth -= 1;
      return body;
    }

    self.loop_depth -= 1;
    Ok(Stmt::While(
      WhileStmt {
        condition: condition.into(),
        body: body?.into(),
      }
      .into(),
    ))
  }

  fn expression_statement(&mut self) -> Result<Stmt, LoxError> {
    let expr = self.expression()?;
    self.consume(&TokenType::Semicolon, "Expect ';' after expression.")?;
    Ok(Stmt::Expression(
      ExpressionStmt {
        expression: expr.into(),
      }
      .into(),
    ))
  }

  fn function(&mut self, kind: &str) -> Result<Stmt, LoxError> {
    let name = self
      .consume(&TokenType::Identifier, &format!("Expect {kind} name."))?
      .clone();

    self.consume(
      &TokenType::LeftBracket,
      &format!("Expect '(' after {kind} name."),
    )?;

    let mut params = Vec::new();
    if !self.check(&TokenType::RightBracket) {
      params.push(
        self
          .consume(&TokenType::Identifier, "Expect parameter name.")?
          .clone(),
      );
      while self.is_match(&[&TokenType::Comma]) {
        if params.len() >= 255 {
          self.had_error = true;
          LoxError::parse_error(self.peek(), "Can't have more than 255 parameters.");
        }

        params.push(
          self
            .consume(&TokenType::Identifier, "Expect parameter name.")?
            .clone(),
        );
      }
    }

    self.consume(&TokenType::RightBracket, "Expect ')' after parameters.")?;

    self.consume(
      &TokenType::LeftBrace,
      &format!("Expect '{{' before {kind} body."),
    )?;

    let body = self
      .block()?
      .into_iter()
      .map(Rc::new)
      .collect::<Vec<Rc<Stmt>>>()
      .into();

    Ok(Stmt::Function(FunctionStmt { name, params, body }.into()))
  }

  fn block(&mut self) -> Result<Vec<Stmt>, LoxError> {
    let mut statements = Vec::new();

    while !self.check(&TokenType::RightBrace) && !self.is_at_end() {
      statements.push(self.declaration()?);
    }

    self.consume(&TokenType::RightBrace, "Expect '}' after block.")?;
    Ok(statements)
  }

  fn assignment(&mut self) -> Result<Expr, LoxError> {
    let expr = self.or()?;

    if !self.is_match(&[&TokenType::Assign]) {
      return Ok(expr);
    }

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

    if let Expr::Get(g) = expr {
      return Ok(Expr::Set(
        SetExpr {
          object: g.object.clone(),
          name: g.name.clone(),
          value: value.into(),
        }
        .into(),
      ));
    }

    self.had_error = true;
    LoxError::parse_error(&equals, "Invalid assignment target.");
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
      );
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
      );
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

    self.call()
  }

  fn finish_call(&mut self, callee: Rc<Expr>) -> Result<Expr, LoxError> {
    let mut arguments = Vec::new();

    if !self.check(&TokenType::RightBracket) {
      arguments.push(self.expression()?);
      while self.is_match(&[&TokenType::Comma]) {
        if arguments.len() >= 255 {
          self.had_error = true;
          return Err(LoxError::parse_error(
            self.peek(),
            "Can't have more than 255 arguments.",
          ));
        }
        arguments.push(self.expression()?);
      }
    }

    let bracket = self.consume(&TokenType::RightBracket, "Expect ')' after arguments.")?;

    Ok(Expr::Call(
      CallExpr {
        callee,
        bracket: bracket.clone(),
        arguments: arguments
          .into_iter()
          .map(Rc::new)
          .collect::<Vec<Rc<Expr>>>()
          .into(),
      }
      .into(),
    ))
  }

  fn call(&mut self) -> Result<Expr, LoxError> {
    let mut expr = self.primary()?;

    loop {
      if self.is_match(&[&TokenType::LeftBracket]) {
        expr = self.finish_call(expr.into())?;
      } else if self.is_match(&[&TokenType::Dot]) {
        let name = self.consume(&TokenType::Identifier, "Expect property name after '.'.")?;
        expr = Expr::Get(
          GetExpr {
            object: expr.into(),
            name: name.clone(),
          }
          .into(),
        );
      } else {
        break;
      }
    }

    Ok(expr)
  }

  fn primary(&mut self) -> Result<Expr, LoxError> {
    if self.is_match(&[&TokenType::False]) {
      return Ok(Expr::Literal(
        LiteralExpr {
          value: Some(Literal::Boolean(false)),
        }
        .into(),
      ));
    }

    if self.is_match(&[&TokenType::True]) {
      return Ok(Expr::Literal(
        LiteralExpr {
          value: Some(Literal::Boolean(true)),
        }
        .into(),
      ));
    }

    if self.is_match(&[&TokenType::Nil]) {
      return Ok(Expr::Literal(
        LiteralExpr {
          value: Some(Literal::Nil),
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

    if self.is_match(&[&TokenType::Super]) {
      let keyword = self.previous().clone();
      self.consume(&TokenType::Dot, "Expect '.' after 'super'.")?;
      let method = self
        .consume(&TokenType::Identifier, "Expect superclass method name.")?
        .clone();
      return Ok(Expr::Super(SuperExpr { keyword, method }.into()));
    }

    if self.is_match(&[&TokenType::This]) {
      return Ok(Expr::This(
        ThisExpr {
          keyword: self.previous().clone(),
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

    if self.is_match(&[&TokenType::LeftBracket]) {
      let expr = self.expression()?;
      self.consume(&TokenType::RightBracket, "Expect ')' after expression.")?;
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
