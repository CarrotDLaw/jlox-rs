use std::hash::Hash;
use std::hash::Hasher;
use std::mem::discriminant;
use std::rc::Rc;

use crate::error::*;
use crate::expr::*;
use crate::token::*;

#[derive(Debug)]
pub enum Stmt {
  Block(Rc<BlockStmt>),
  Break(Rc<BreakStmt>),
  Expression(Rc<ExpressionStmt>),
  Function(Rc<FunctionStmt>),
  If(Rc<IfStmt>),
  Print(Rc<PrintStmt>),
  Return(Rc<ReturnStmt>),
  Var(Rc<VarStmt>),
  While(Rc<WhileStmt>),
}

impl Stmt {
  pub fn accept<T>(&self, wrapper: &Rc<Stmt>, stmt_visitor: &dyn StmtVisitor<T>) -> Result<T, LoxError> {
    match self {
      Stmt::Block(stmt) => stmt_visitor.visit_block_stmt(wrapper, stmt),
      Stmt::Break(stmt) => stmt_visitor.visit_break_stmt(wrapper, stmt),
      Stmt::Expression(stmt) => stmt_visitor.visit_expression_stmt(wrapper, stmt),
      Stmt::Function(stmt) => stmt_visitor.visit_function_stmt(wrapper, stmt),
      Stmt::If(stmt) => stmt_visitor.visit_if_stmt(wrapper, stmt),
      Stmt::Print(stmt) => stmt_visitor.visit_print_stmt(wrapper, stmt),
      Stmt::Return(stmt) => stmt_visitor.visit_return_stmt(wrapper, stmt),
      Stmt::Var(stmt) => stmt_visitor.visit_var_stmt(wrapper, stmt),
      Stmt::While(stmt) => stmt_visitor.visit_while_stmt(wrapper, stmt),
    }
  }
}

impl PartialEq for Stmt {
  fn eq(&self, other: &Stmt) -> bool {
    match (self, other) {
      (Stmt::Block(l0), Stmt::Block(r0)) => Rc::ptr_eq(l0, r0),
      (Stmt::Break(l0), Stmt::Break(r0)) => Rc::ptr_eq(l0, r0),
      (Stmt::Expression(l0), Stmt::Expression(r0)) => Rc::ptr_eq(l0, r0),
      (Stmt::Function(l0), Stmt::Function(r0)) => Rc::ptr_eq(l0, r0),
      (Stmt::If(l0), Stmt::If(r0)) => Rc::ptr_eq(l0, r0),
      (Stmt::Print(l0), Stmt::Print(r0)) => Rc::ptr_eq(l0, r0),
      (Stmt::Return(l0), Stmt::Return(r0)) => Rc::ptr_eq(l0, r0),
      (Stmt::Var(l0), Stmt::Var(r0)) => Rc::ptr_eq(l0, r0),
      (Stmt::While(l0), Stmt::While(r0)) => Rc::ptr_eq(l0, r0),
      _ => false,
    }
  }
}

impl Eq for Stmt {}

impl Hash for Stmt {
  fn hash<H: Hasher>(&self, state: &mut H) {
    discriminant(self).hash(state);
  }
}

#[derive(Debug)]
pub struct BlockStmt {
  pub statements: Rc<Vec<Rc<Stmt>>>,
}

#[derive(Debug)]
pub struct BreakStmt {
  pub token: Token,
}

#[derive(Debug)]
pub struct ExpressionStmt {
  pub expression: Rc<Expr>,
}

#[derive(Debug)]
pub struct FunctionStmt {
  pub name: Token,
  pub params: Vec<Token>,
  pub body: Rc<Vec<Rc<Stmt>>>,
}

#[derive(Debug)]
pub struct IfStmt {
  pub condition: Rc<Expr>,
  pub then_branch: Rc<Stmt>,
  pub else_branch: Option<Rc<Stmt>>,
}

#[derive(Debug)]
pub struct PrintStmt {
  pub expression: Rc<Expr>,
}

#[derive(Debug)]
pub struct ReturnStmt {
  pub keyword: Token,
  pub value: Option<Rc<Expr>>,
}

#[derive(Debug)]
pub struct VarStmt {
  pub name: Token,
  pub initialiser: Option<Rc<Expr>>,
}

#[derive(Debug)]
pub struct WhileStmt {
  pub condition: Rc<Expr>,
  pub body: Rc<Stmt>,
}

pub trait StmtVisitor<T> {
  fn visit_block_stmt(&self, wrapper: &Rc<Stmt>, stmt: &BlockStmt) -> Result<T, LoxError>;
  fn visit_break_stmt(&self, wrapper: &Rc<Stmt>, stmt: &BreakStmt) -> Result<T, LoxError>;
  fn visit_expression_stmt(&self, wrapper: &Rc<Stmt>, stmt: &ExpressionStmt) -> Result<T, LoxError>;
  fn visit_function_stmt(&self, wrapper: &Rc<Stmt>, stmt: &FunctionStmt) -> Result<T, LoxError>;
  fn visit_if_stmt(&self, wrapper: &Rc<Stmt>, stmt: &IfStmt) -> Result<T, LoxError>;
  fn visit_print_stmt(&self, wrapper: &Rc<Stmt>, stmt: &PrintStmt) -> Result<T, LoxError>;
  fn visit_return_stmt(&self, wrapper: &Rc<Stmt>, stmt: &ReturnStmt) -> Result<T, LoxError>;
  fn visit_var_stmt(&self, wrapper: &Rc<Stmt>, stmt: &VarStmt) -> Result<T, LoxError>;
  fn visit_while_stmt(&self, wrapper: &Rc<Stmt>, stmt: &WhileStmt) -> Result<T, LoxError>;
}
