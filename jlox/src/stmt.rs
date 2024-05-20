use std::rc::Rc;

use crate::error::*;
use crate::expr::*;
use crate::token::*;

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
  pub fn accept<T>(&self, stmt_visitor: &dyn StmtVisitor<T>) -> Result<T, LoxError> {
    match self {
      Stmt::Block(stmt) => stmt_visitor.visit_block_stmt(stmt),
      Stmt::Break(stmt) => stmt_visitor.visit_break_stmt(stmt),
      Stmt::Expression(stmt) => stmt_visitor.visit_expression_stmt(stmt),
      Stmt::Function(stmt) => stmt_visitor.visit_function_stmt(stmt),
      Stmt::If(stmt) => stmt_visitor.visit_if_stmt(stmt),
      Stmt::Print(stmt) => stmt_visitor.visit_print_stmt(stmt),
      Stmt::Return(stmt) => stmt_visitor.visit_return_stmt(stmt),
      Stmt::Var(stmt) => stmt_visitor.visit_var_stmt(stmt),
      Stmt::While(stmt) => stmt_visitor.visit_while_stmt(stmt),
    }
  }
}

pub struct BlockStmt {
  pub statements: Vec<Rc<Stmt>>,
}

pub struct BreakStmt {
  pub token: Token,
}

pub struct ExpressionStmt {
  pub expression: Rc<Expr>,
}

pub struct FunctionStmt {
  pub name: Token,
  pub params: Vec<Token>,
  pub body: Vec<Rc<Stmt>>,
}

pub struct IfStmt {
  pub condition: Rc<Expr>,
  pub then_branch: Rc<Stmt>,
  pub else_branch: Option<Rc<Stmt>>,
}

pub struct PrintStmt {
  pub expression: Rc<Expr>,
}

pub struct ReturnStmt {
  pub keyword: Token,
  pub value: Option<Rc<Expr>>,
}

pub struct VarStmt {
  pub name: Token,
  pub initialiser: Option<Rc<Expr>>,
}

pub struct WhileStmt {
  pub condition: Rc<Expr>,
  pub body: Rc<Stmt>,
}

pub trait StmtVisitor<T> {
  fn visit_block_stmt(&self, stmt: &BlockStmt) -> Result<T, LoxError>;
  fn visit_break_stmt(&self, stmt: &BreakStmt) -> Result<T, LoxError>;
  fn visit_expression_stmt(&self, stmt: &ExpressionStmt) -> Result<T, LoxError>;
  fn visit_function_stmt(&self, stmt: &FunctionStmt) -> Result<T, LoxError>;
  fn visit_if_stmt(&self, stmt: &IfStmt) -> Result<T, LoxError>;
  fn visit_print_stmt(&self, stmt: &PrintStmt) -> Result<T, LoxError>;
  fn visit_return_stmt(&self, stmt: &ReturnStmt) -> Result<T, LoxError>;
  fn visit_var_stmt(&self, stmt: &VarStmt) -> Result<T, LoxError>;
  fn visit_while_stmt(&self, stmt: &WhileStmt) -> Result<T, LoxError>;
}
