use std::rc::Rc;

use crate::error::*;
use crate::token::*;

pub enum Expr {
  Assign(Rc<AssignExpr>),
  Binary(Rc<BinaryExpr>),
  Grouping(Rc<GroupingExpr>),
  Literal(Rc<LiteralExpr>),
  Unary(Rc<UnaryExpr>),
  Variable(Rc<VariableExpr>),
}

impl Expr {
  pub fn accept<T>(&self, expr_visitor: &dyn ExprVisitor<T>) -> Result<T, LoxError> {
    match self {
      Expr::Assign(expr) => expr_visitor.visit_assign_expr(expr),
      Expr::Binary(expr) => expr_visitor.visit_binary_expr(expr),
      Expr::Grouping(expr) => expr_visitor.visit_grouping_expr(expr),
      Expr::Literal(expr) => expr_visitor.visit_literal_expr(expr),
      Expr::Unary(expr) => expr_visitor.visit_unary_expr(expr),
      Expr::Variable(expr) => expr_visitor.visit_variable_expr(expr),
    }
  }
}

pub struct AssignExpr {
  pub name: Token,
  pub value: Rc<Expr>,
}

pub struct BinaryExpr {
  pub left: Rc<Expr>,
  pub operator: Token,
  pub right: Rc<Expr>,
}

pub struct GroupingExpr {
  pub expression: Rc<Expr>,
}

pub struct LiteralExpr {
  pub value: Option<Object>,
}

pub struct UnaryExpr {
  pub operator: Token,
  pub right: Rc<Expr>,
}

pub struct VariableExpr {
  pub name: Token,
}

pub trait ExprVisitor<T> {
  fn visit_assign_expr(&self, expr: &AssignExpr) -> Result<T, LoxError>;
  fn visit_binary_expr(&self, expr: &BinaryExpr) -> Result<T, LoxError>;
  fn visit_grouping_expr(&self, expr: &GroupingExpr) -> Result<T, LoxError>;
  fn visit_literal_expr(&self, expr: &LiteralExpr) -> Result<T, LoxError>;
  fn visit_unary_expr(&self, expr: &UnaryExpr) -> Result<T, LoxError>;
  fn visit_variable_expr(&self, expr: &VariableExpr) -> Result<T, LoxError>;
}
