use std::rc::Rc;

use crate::error::*;
use crate::expr::*;
use crate::token::*;

pub enum Stmt {
  Block(Rc<BlockStmt>),
  Expression(Rc<ExpressionStmt>),
  Print(Rc<PrintStmt>),
  Var(Rc<VarStmt>),
}

impl Stmt {
  pub fn accept<T>(&self, stmt_visitor: &dyn StmtVisitor<T>) -> Result<T, LoxError> {
    match self {
      Stmt::Block(stmt) => stmt_visitor.visit_block_stmt(stmt),
      Stmt::Expression(stmt) => stmt_visitor.visit_expression_stmt(stmt),
      Stmt::Print(stmt) => stmt_visitor.visit_print_stmt(stmt),
      Stmt::Var(stmt) => stmt_visitor.visit_var_stmt(stmt),
    }
  }
}

pub struct BlockStmt {
  pub statements: Vec<Rc<Stmt>>,
}

pub struct ExpressionStmt {
  pub expression: Rc<Expr>,
}

pub struct PrintStmt {
  pub expression: Rc<Expr>,
}

pub struct VarStmt {
  pub name: Token,
  pub initialiser: Option<Rc<Expr>>,
}

pub trait StmtVisitor<T> {
  fn visit_block_stmt(&self, stmt: &BlockStmt) -> Result<T, LoxError>;
  fn visit_expression_stmt(&self, stmt: &ExpressionStmt) -> Result<T, LoxError>;
  fn visit_print_stmt(&self, stmt: &PrintStmt) -> Result<T, LoxError>;
  fn visit_var_stmt(&self, stmt: &VarStmt) -> Result<T, LoxError>;
}
