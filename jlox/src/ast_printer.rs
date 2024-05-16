use std::rc::Rc;

use crate::error::*;
use crate::expr::*;

pub struct AstPrinter;

impl AstPrinter {
  pub fn print(&self, expr: &Expr) -> Result<String, LoxError> {
    expr.accept(self)
  }

  fn parenthesise(&self, name: &str, exprs: &[&Rc<Expr>]) -> Result<String, LoxError> {
    let mut builder = format!("({name}");

    for expr in exprs {
      builder = format!("{} {}", builder, expr.accept(self)?);
    }
    builder.push(')');

    Ok(builder)
  }
}

impl ExprVisitor<String> for AstPrinter {
  fn visit_binary_expr(&self, expr: &BinaryExpr) -> Result<String, LoxError> {
    self.parenthesise(&expr.operator.get_lexeme(), &[&expr.left, &expr.right])
  }

  fn visit_grouping_expr(&self, expr: &GroupingExpr) -> Result<String, LoxError> {
    self.parenthesise("group", &[&expr.expression])
  }

  fn visit_literal_expr(&self, expr: &LiteralExpr) -> Result<String, LoxError> {
    if let Some(v) = &expr.value {
      Ok(v.to_string())
    } else {
      Ok("nil".to_string())
    }
  }

  fn visit_unary_expr(&self, expr: &UnaryExpr) -> Result<String, LoxError> {
    self.parenthesise(&expr.operator.get_lexeme(), &[&expr.right])
  }
}

#[cfg(test)]
mod test {
  use super::*;

  use crate::token::*;

  #[test]
  fn test_ast_printer() {
    let expr = Expr::Binary(Rc::new(BinaryExpr {
      left: Rc::new(Expr::Unary(Rc::new(UnaryExpr {
        operator: Token::new(TokenType::Minus, "-", None, 1),
        right: Rc::new(Expr::Literal(Rc::new(LiteralExpr {
          value: Some(Object::Number(123.0)),
        }))),
      }))),
      operator: Token::new(TokenType::Star, "*", None, 1),
      right: Rc::new(Expr::Grouping(Rc::new(GroupingExpr {
        expression: Rc::new(Expr::Literal(Rc::new(LiteralExpr {
          value: Some(Object::Number(45.67)),
        }))),
      }))),
    }));

    println!(
      "{}",
      AstPrinter
        .print(&expr)
        .unwrap_or("AST PRINTER ERROR".to_string())
    );
  }
}
