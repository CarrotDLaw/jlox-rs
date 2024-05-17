use std::rc::Rc;

use crate::{error::*, expr::*, token::*};

pub struct Interpreter {}

impl Interpreter {
  pub fn new() -> Interpreter {
    Interpreter {}
  }

  pub fn interpret(&self, expr: &Rc<Expr>) -> Result<Object, LoxError> {
    let value = self.evaluate(expr)?;
    println!("{}", value);
    Ok(value)
  }

  fn evaluate(&self, expr: &Rc<Expr>) -> Result<Object, LoxError> {
    expr.accept(self)
  }

  fn is_truthy(&self, object: &Object) -> bool {
    !matches!(object, Object::Nil | Object::Boolean(false))
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

impl ExprVisitor<Object> for Interpreter {
  fn visit_binary_expr(&self, expr: &BinaryExpr) -> Result<Object, LoxError> {
    let left = self.evaluate(&expr.left)?;
    let right = self.evaluate(&expr.right)?;
    let operator_type = expr.operator.get_type();

    if let (Object::Number(left), Object::Number(right)) = (&left, &right) {
      return match operator_type {
        TokenType::Plus => Ok(Object::Number(left + right)),
        TokenType::Minus => Ok(Object::Number(left - right)),
        TokenType::Star => Ok(Object::Number(left * right)),
        TokenType::Slash => Ok(Object::Number(left / right)),
        TokenType::Greater => Ok(Object::Boolean(left > right)),
        TokenType::GreaterEqual => Ok(Object::Boolean(left >= right)),
        TokenType::Less => Ok(Object::Boolean(left < right)),
        TokenType::LessEqual => Ok(Object::Boolean(left <= right)),
        TokenType::BangEqual => Ok(Object::Boolean(left != right)),
        TokenType::EqualEqual => Ok(Object::Boolean(left == right)),
        _ => Err(Interpreter::internal_error(&expr.operator)),
      };
    }

    if let (Object::String(left), Object::String(right)) = (&left, &right) {
      return match operator_type {
        TokenType::Plus => Ok(Object::String(format!("{left}{right}"))),
        TokenType::BangEqual => Ok(Object::Boolean(left != right)),
        TokenType::EqualEqual => Ok(Object::Boolean(left == right)),
        _ => Err(Interpreter::numbers_error(&expr.operator)),
      };
    }

    if let (Object::String(left), _) = (&left, &right) {
      return match operator_type {
        TokenType::Plus => Ok(Object::String(format!("{left}{right}"))),
        TokenType::BangEqual => Ok(Object::Boolean(true)),
        TokenType::EqualEqual => Ok(Object::Boolean(false)),
        _ => Err(Interpreter::numbers_error(&expr.operator)),
      };
    }

    if let (_, Object::String(right)) = (&left, &right) {
      return match operator_type {
        TokenType::Plus => Ok(Object::String(format!("{left}{right}"))),
        TokenType::BangEqual => Ok(Object::Boolean(true)),
        TokenType::EqualEqual => Ok(Object::Boolean(false)),
        _ => Err(Interpreter::numbers_error(&expr.operator)),
      };
    }

    if let (Object::Boolean(left), Object::Boolean(right)) = (&left, &right) {
      return match operator_type {
        TokenType::BangEqual => Ok(Object::Boolean(left != right)),
        TokenType::EqualEqual => Ok(Object::Boolean(left == right)),
        TokenType::Plus => Err(Interpreter::numbers_or_strings_error(&expr.operator)),
        _ => Err(Interpreter::numbers_error(&expr.operator)),
      };
    }

    if let (Object::Nil, Object::Nil) = (&left, &right) {
      return match operator_type {
        TokenType::BangEqual => Ok(Object::Boolean(false)),
        TokenType::EqualEqual => Ok(Object::Boolean(true)),
        TokenType::Plus => Err(Interpreter::numbers_or_strings_error(&expr.operator)),
        _ => Err(Interpreter::numbers_error(&expr.operator)),
      };
    }

    match operator_type {
      TokenType::BangEqual => Ok(Object::Boolean(true)),
      TokenType::EqualEqual => Ok(Object::Boolean(false)),
      TokenType::Plus => Err(Interpreter::numbers_or_strings_error(&expr.operator)),
      _ => Err(Interpreter::numbers_error(&expr.operator)),
    }
  }

  fn visit_grouping_expr(&self, expr: &GroupingExpr) -> Result<Object, LoxError> {
    self.evaluate(&expr.expression)
  }

  fn visit_literal_expr(&self, expr: &LiteralExpr) -> Result<Object, LoxError> {
    if let Some(v) = &expr.value {
      return Ok(v.clone());
    }

    unreachable!()
  }

  fn visit_unary_expr(&self, expr: &UnaryExpr) -> Result<Object, LoxError> {
    let right = self.evaluate(&expr.right)?;

    match expr.operator.get_type() {
      TokenType::Bang => Ok(Object::Boolean(!self.is_truthy(&right))),
      TokenType::Minus => Ok(Object::Number(
        -right
          .get_number()
          .map_err(|_| Interpreter::number_error(&expr.operator))?,
      )),
      _ => Err(Interpreter::internal_error(&expr.operator)),
    }
  }
}

#[cfg(test)]
mod test {
  use super::*;

  use crate::{parser::*, scanner::*};

  #[test]
  fn test_interpreter() -> Result<(), LoxError> {
    let source = "-123 * (45.67)";
    let mut scanner = Scanner::new(source);
    let tokens = scanner.scan_tokens()?;
    let mut parser = Parser::new(tokens);
    let expression = parser.parse()?;
    let mut interpreter = Interpreter::new();
    let value = interpreter.interpret(&Rc::new(expression))?;

    assert!(matches!(value, Object::Number(n) if n == -5617.41));

    Ok(())
  }
}
