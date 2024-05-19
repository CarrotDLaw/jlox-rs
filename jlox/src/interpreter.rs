use std::{cell::RefCell, rc::Rc};

use crate::{environment::*, error::*, expr::*, stmt::*, token::*};

#[derive(Default)]
pub struct Interpreter {
  environment: RefCell<Rc<RefCell<Environment>>>,
}

impl Interpreter {
  pub fn new() -> Interpreter {
    Interpreter {
      environment: RefCell::new(Rc::new(RefCell::new(Environment::new()))),
    }
  }

  pub fn interpret(&self, statements: &[Rc<Stmt>]) -> Result<(), LoxError> {
    for statement in statements {
      self.execute(statement)?;
    }

    Ok(())
  }

  pub fn print_environment(&self) {
    dbg!(&self.environment);
  }

  fn evaluate(&self, expr: &Rc<Expr>) -> Result<Literal, LoxError> {
    expr.accept(self)
  }

  fn execute(&self, stmt: &Rc<Stmt>) -> Result<(), LoxError> {
    stmt.accept(self)
  }

  fn execute_block(
    &self,
    statements: &[Rc<Stmt>],
    environment: Environment,
  ) -> Result<(), LoxError> {
    let previous = self.environment.replace(RefCell::new(environment).into());

    let res = statements.iter().try_for_each(|s| self.execute(s));

    self.environment.replace(previous);
    res
  }

  fn is_truthy(&self, object: &Literal) -> bool {
    !matches!(object, Literal::Nil | Literal::Boolean(false))
  }

  fn internal_error(operator: &Token) -> LoxError {
    LoxError::runtime_error(operator, "INTERPRETER INTERNAL ERROR.")
  }

  fn number_error(operator: &Token) -> LoxError {
    LoxError::runtime_error(operator, "Operand must be a number.")
  }

  fn numbers_error(operator: &Token) -> LoxError {
    LoxError::runtime_error(operator, "Operands must be numbers.")
  }

  fn numbers_or_strings_error(operator: &Token) -> LoxError {
    LoxError::runtime_error(operator, "Operands must be two numbers or two strings.")
  }
}

impl ExprVisitor<Literal> for Interpreter {
  fn visit_assign_expr(&self, expr: &AssignExpr) -> Result<Literal, LoxError> {
    let value = self.evaluate(&expr.value)?;
    self
      .environment
      .borrow()
      .borrow_mut()
      .assign(&expr.name, &value)?;
    Ok(value)
  }

  fn visit_binary_expr(&self, expr: &BinaryExpr) -> Result<Literal, LoxError> {
    let left = self.evaluate(&expr.left)?;
    let right = self.evaluate(&expr.right)?;
    let operator_type = expr.operator.get_type();

    if let (Literal::Number(left), Literal::Number(right)) = (&left, &right) {
      return match operator_type {
        TokenType::Plus => Ok(Literal::Number(left + right)),
        TokenType::Minus => Ok(Literal::Number(left - right)),
        TokenType::Star => Ok(Literal::Number(left * right)),
        TokenType::Slash => Ok(Literal::Number(left / right)),
        TokenType::Greater => Ok(Literal::Boolean(left > right)),
        TokenType::GreaterEqual => Ok(Literal::Boolean(left >= right)),
        TokenType::Less => Ok(Literal::Boolean(left < right)),
        TokenType::LessEqual => Ok(Literal::Boolean(left <= right)),
        TokenType::BangEqual => Ok(Literal::Boolean(left != right)),
        TokenType::EqualEqual => Ok(Literal::Boolean(left == right)),
        _ => Err(Interpreter::internal_error(&expr.operator)),
      };
    }

    if let (Literal::String(left), Literal::String(right)) = (&left, &right) {
      return match operator_type {
        TokenType::Plus => Ok(Literal::String(format!("{left}{right}"))),
        TokenType::BangEqual => Ok(Literal::Boolean(left != right)),
        TokenType::EqualEqual => Ok(Literal::Boolean(left == right)),
        _ => Err(Interpreter::numbers_error(&expr.operator)),
      };
    }

    if let (Literal::String(left), Literal::Number(right)) = (&left, &right) {
      return match operator_type {
        TokenType::Plus => Ok(Literal::String(format!("{left}{right}"))),
        TokenType::BangEqual => Ok(Literal::Boolean(true)),
        TokenType::EqualEqual => Ok(Literal::Boolean(false)),
        _ => Err(Interpreter::numbers_error(&expr.operator)),
      };
    }

    if let (Literal::Number(left), Literal::String(right)) = (&left, &right) {
      return match operator_type {
        TokenType::Plus => Ok(Literal::String(format!("{left}{right}"))),
        TokenType::BangEqual => Ok(Literal::Boolean(true)),
        TokenType::EqualEqual => Ok(Literal::Boolean(false)),
        _ => Err(Interpreter::numbers_error(&expr.operator)),
      };
    }

    if let (Literal::Boolean(left), Literal::Boolean(right)) = (&left, &right) {
      return match operator_type {
        TokenType::BangEqual => Ok(Literal::Boolean(left != right)),
        TokenType::EqualEqual => Ok(Literal::Boolean(left == right)),
        TokenType::Plus => Err(Interpreter::numbers_or_strings_error(&expr.operator)),
        _ => Err(Interpreter::numbers_error(&expr.operator)),
      };
    }

    if let (Literal::Nil, Literal::Nil) = (&left, &right) {
      return match operator_type {
        TokenType::BangEqual => Ok(Literal::Boolean(false)),
        TokenType::EqualEqual => Ok(Literal::Boolean(true)),
        TokenType::Plus => Err(Interpreter::numbers_or_strings_error(&expr.operator)),
        _ => Err(Interpreter::numbers_error(&expr.operator)),
      };
    }

    match operator_type {
      TokenType::BangEqual => Ok(Literal::Boolean(true)),
      TokenType::EqualEqual => Ok(Literal::Boolean(false)),
      TokenType::Plus => Err(Interpreter::numbers_or_strings_error(&expr.operator)),
      _ => Err(Interpreter::numbers_error(&expr.operator)),
    }
  }

  fn visit_grouping_expr(&self, expr: &GroupingExpr) -> Result<Literal, LoxError> {
    self.evaluate(&expr.expression)
  }

  fn visit_literal_expr(&self, expr: &LiteralExpr) -> Result<Literal, LoxError> {
    if let Some(v) = &expr.value {
      return Ok(v.clone());
    }

    unreachable!()
  }

  fn visit_logical_expr(&self, expr: &LogicalExpr) -> Result<Literal, LoxError> {
    let left = self.evaluate(&expr.left)?;

    if expr.operator.is_type(&TokenType::Or) {
      if self.is_truthy(&left) {
        return Ok(left);
      }
    } else if !self.is_truthy(&left) {
      return Ok(left);
    }

    self.evaluate(&expr.right)
  }

  fn visit_unary_expr(&self, expr: &UnaryExpr) -> Result<Literal, LoxError> {
    let right = self.evaluate(&expr.right)?;

    match expr.operator.get_type() {
      TokenType::Bang => Ok(Literal::Boolean(!self.is_truthy(&right))),
      TokenType::Minus => Ok(Literal::Number(
        -right
          .get_number()
          .map_err(|_| Interpreter::number_error(&expr.operator))?,
      )),
      _ => Err(Interpreter::internal_error(&expr.operator)),
    }
  }

  fn visit_variable_expr(&self, expr: &VariableExpr) -> Result<Literal, LoxError> {
    self.environment.borrow().borrow().get(&expr.name)
  }
}

impl StmtVisitor<()> for Interpreter {
  fn visit_block_stmt(&self, stmt: &BlockStmt) -> Result<(), LoxError> {
    let env = Environment::new_with_enclosing(&self.environment.borrow().clone());
    self.execute_block(&stmt.statements, env)
  }

  fn visit_expression_stmt(&self, stmt: &ExpressionStmt) -> Result<(), LoxError> {
    self.evaluate(&stmt.expression)?;
    Ok(())
  }

  fn visit_if_stmt(&self, stmt: &IfStmt) -> Result<(), LoxError> {
    if self.is_truthy(&self.evaluate(&stmt.condition)?) {
      return self.execute(&stmt.then_branch);
    }

    if let Some(b) = &stmt.else_branch {
      return self.execute(b);
    }

    Ok(())
  }

  fn visit_print_stmt(&self, stmt: &PrintStmt) -> Result<(), LoxError> {
    println!("{}", self.evaluate(&stmt.expression)?);
    Ok(())
  }

  fn visit_var_stmt(&self, stmt: &VarStmt) -> Result<(), LoxError> {
    let value = if let Some(i) = &stmt.initialiser {
      self.evaluate(i)?
    } else {
      Literal::Nil
    };

    self
      .environment
      .borrow()
      .borrow_mut()
      .define(stmt.name.get_lexeme(), value);
    Ok(())
  }

  fn visit_while_stmt(&self, stmt: &WhileStmt) -> Result<(), LoxError> {
    while self.is_truthy(&self.evaluate(&stmt.condition)?) {
      self.execute(&stmt.body)?;
    }

    Ok(())
  }
}

#[cfg(test)]
mod test {
  use super::*;

  use crate::{parser::*, scanner::*};

  #[test]
  fn test_interpreter() -> Result<(), LoxError> {
    let interpreter = Interpreter::new();
    let source = "print -123 * (45.67);";
    let mut scanner = Scanner::new(source);
    let tokens = scanner.scan_tokens()?;
    let mut parser = Parser::new(tokens);
    let statements = parser.parse()?;

    interpreter.interpret(
      &statements
        .into_iter()
        .map(Rc::new)
        .collect::<Vec<Rc<Stmt>>>(),
    )?;

    Ok(())
  }

  #[test]
  fn test_global_variable() -> Result<(), LoxError> {
    let interpreter = Interpreter::new();
    let source = "var a = 1;\nvar b = 2;\nprint a + b;".to_string();
    let mut scanner = Scanner::new(&source);
    let tokens = scanner.scan_tokens()?;
    let mut parser = Parser::new(tokens);
    let statements = parser.parse()?;

    interpreter.interpret(
      &statements
        .into_iter()
        .map(Rc::new)
        .collect::<Vec<Rc<Stmt>>>(),
    )?;

    Ok(())
  }
}
