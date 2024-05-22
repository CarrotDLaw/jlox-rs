use std::hash::Hash;
use std::hash::Hasher;
use std::rc::Rc;

use crate::error::*;
use crate::token::*;

pub enum Expr {
  Assign(Rc<AssignExpr>),
  Binary(Rc<BinaryExpr>),
  Call(Rc<CallExpr>),
  Grouping(Rc<GroupingExpr>),
  Literal(Rc<LiteralExpr>),
  Logical(Rc<LogicalExpr>),
  Unary(Rc<UnaryExpr>),
  Variable(Rc<VariableExpr>),
}

impl Expr {
  pub fn accept<T>(&self, wrapper: &Rc<Expr>, expr_visitor: &dyn ExprVisitor<T>) -> Result<T, LoxError> {
    match self {
      Expr::Assign(expr) => expr_visitor.visit_assign_expr(wrapper, expr),
      Expr::Binary(expr) => expr_visitor.visit_binary_expr(wrapper, expr),
      Expr::Call(expr) => expr_visitor.visit_call_expr(wrapper, expr),
      Expr::Grouping(expr) => expr_visitor.visit_grouping_expr(wrapper, expr),
      Expr::Literal(expr) => expr_visitor.visit_literal_expr(wrapper, expr),
      Expr::Logical(expr) => expr_visitor.visit_logical_expr(wrapper, expr),
      Expr::Unary(expr) => expr_visitor.visit_unary_expr(wrapper, expr),
      Expr::Variable(expr) => expr_visitor.visit_variable_expr(wrapper, expr),
    }
  }
}

impl PartialEq for Expr {
  fn eq(&self, other: &Expr) -> bool {
    match (self, other) {
     (Expr::Assign(l0), Expr::Assign(r0)) => Rc::ptr_eq(l0, r0),
     (Expr::Binary(l0), Expr::Binary(r0)) => Rc::ptr_eq(l0, r0),
     (Expr::Call(l0), Expr::Call(r0)) => Rc::ptr_eq(l0, r0),
     (Expr::Grouping(l0), Expr::Grouping(r0)) => Rc::ptr_eq(l0, r0),
     (Expr::Literal(l0), Expr::Literal(r0)) => Rc::ptr_eq(l0, r0),
     (Expr::Logical(l0), Expr::Logical(r0)) => Rc::ptr_eq(l0, r0),
     (Expr::Unary(l0), Expr::Unary(r0)) => Rc::ptr_eq(l0, r0),
     (Expr::Variable(l0), Expr::Variable(r0)) => Rc::ptr_eq(l0, r0),
      _ => false,
    }
  }
}

impl Eq for Expr {}

impl Hash for Expr {
  fn hash<H: Hasher>(&self, state: &mut H) {
    match self {
       Expr::Assign(e) => state.write_usize(Rc::as_ptr(e) as usize),
       Expr::Binary(e) => state.write_usize(Rc::as_ptr(e) as usize),
       Expr::Call(e) => state.write_usize(Rc::as_ptr(e) as usize),
       Expr::Grouping(e) => state.write_usize(Rc::as_ptr(e) as usize),
       Expr::Literal(e) => state.write_usize(Rc::as_ptr(e) as usize),
       Expr::Logical(e) => state.write_usize(Rc::as_ptr(e) as usize),
       Expr::Unary(e) => state.write_usize(Rc::as_ptr(e) as usize),
       Expr::Variable(e) => state.write_usize(Rc::as_ptr(e) as usize),
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

pub struct CallExpr {
  pub callee: Rc<Expr>,
  pub bracket: Token,
  pub arguments: Rc<Vec<Rc<Expr>>>,
}

pub struct GroupingExpr {
  pub expression: Rc<Expr>,
}

pub struct LiteralExpr {
  pub value: Option<Literal>,
}

pub struct LogicalExpr {
  pub left: Rc<Expr>,
  pub operator: Token,
  pub right: Rc<Expr>,
}

pub struct UnaryExpr {
  pub operator: Token,
  pub right: Rc<Expr>,
}

pub struct VariableExpr {
  pub name: Token,
}

pub trait ExprVisitor<T> {
  fn visit_assign_expr(&self, wrapper: &Rc<Expr>, expr: &AssignExpr) -> Result<T, LoxError>;
  fn visit_binary_expr(&self, wrapper: &Rc<Expr>, expr: &BinaryExpr) -> Result<T, LoxError>;
  fn visit_call_expr(&self, wrapper: &Rc<Expr>, expr: &CallExpr) -> Result<T, LoxError>;
  fn visit_grouping_expr(&self, wrapper: &Rc<Expr>, expr: &GroupingExpr) -> Result<T, LoxError>;
  fn visit_literal_expr(&self, wrapper: &Rc<Expr>, expr: &LiteralExpr) -> Result<T, LoxError>;
  fn visit_logical_expr(&self, wrapper: &Rc<Expr>, expr: &LogicalExpr) -> Result<T, LoxError>;
  fn visit_unary_expr(&self, wrapper: &Rc<Expr>, expr: &UnaryExpr) -> Result<T, LoxError>;
  fn visit_variable_expr(&self, wrapper: &Rc<Expr>, expr: &VariableExpr) -> Result<T, LoxError>;
}
