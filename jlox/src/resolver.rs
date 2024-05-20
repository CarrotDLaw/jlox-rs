use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{error::*, expr::*, interpreter::*, stmt::*, token::*};

pub struct Resolver {
  interpreter: Interpreter,
  scopes: RefCell<Vec<RefCell<HashMap<String, bool>>>>,
}

impl Resolver {
  pub fn new(interpreter: &Interpreter) -> Resolver {
    Resolver {
      interpreter: interpreter.clone(),
      scopes: RefCell::new(Vec::new()),
    }
  }

  fn resolve(&self, statements: &[Rc<Stmt>]) -> Result<(), LoxError> {
    for statement in statements {
      self.resolve_stmt(statement)?;
    }

    Ok(())
  }

  fn resolve_expr(&self, expr: &Expr) -> Result<(), LoxError> {
    expr.accept(self)
  }

  fn resolve_stmt(&self, stmt: &Stmt) -> Result<(), LoxError> {
    stmt.accept(self)
  }

  fn begin_scope(&self) {
    self.scopes.borrow_mut().push(RefCell::new(HashMap::new()))
  }

  fn end_scope(&self) {
    self.scopes.borrow_mut().pop();
  }

  fn declare(&self, name: &Token) {
    if let Some(s) = self.scopes.borrow().last() {
      s.borrow_mut().insert(name.get_lexeme().to_string(), false);
    }
  }

  fn define(&self, name: &Token) {
    if let Some(s) = self.scopes.borrow().last() {
      s.borrow_mut().insert(name.get_lexeme().to_string(), true);
    }
  }

  fn resolve_local(&self, expr: &Expr, name: &Token) {
    for (scope, map) in self.scopes.borrow().iter().rev().enumerate() {
      if map.borrow().contains_key(name.get_lexeme()) {
        // self.interpreter.resolve(expr, scope);
        return;
      }
    }
  }
}

impl ExprVisitor<()> for Resolver {
  fn visit_assign_expr(&self, expr: &AssignExpr) -> Result<(), LoxError> {
    todo!()
  }

  fn visit_binary_expr(&self, expr: &BinaryExpr) -> Result<(), LoxError> {
    todo!()
  }

  fn visit_call_expr(&self, expr: &CallExpr) -> Result<(), LoxError> {
    todo!()
  }

  fn visit_grouping_expr(&self, expr: &GroupingExpr) -> Result<(), LoxError> {
    todo!()
  }

  fn visit_literal_expr(&self, expr: &LiteralExpr) -> Result<(), LoxError> {
    todo!()
  }

  fn visit_logical_expr(&self, expr: &LogicalExpr) -> Result<(), LoxError> {
    todo!()
  }

  fn visit_unary_expr(&self, expr: &UnaryExpr) -> Result<(), LoxError> {
    todo!()
  }

  fn visit_variable_expr(&self, expr: &VariableExpr) -> Result<(), LoxError> {
    if !self.scopes.borrow().is_empty()
      && self
        .scopes
        .borrow()
        .last()
        .ok_or_else(|| LoxError::runtime_error(&expr.name, "INTERPRETER INTERNAL ERROR."))?
        .borrow()
        .get(expr.name.get_lexeme())
        .is_some_and(|&b| !b)
    {
      return Err(LoxError::runtime_error(
        &expr.name,
        "Can't read local variable in its own initialiser.",
      ));
    }

    // self.resolve_local(expr, &expr.name);
    Ok(())
  }
}

impl StmtVisitor<()> for Resolver {
  fn visit_block_stmt(&self, stmt: &BlockStmt) -> Result<(), LoxError> {
    self.begin_scope();
    self.resolve(&stmt.statements)?;
    self.end_scope();
    Ok(())
  }

  fn visit_break_stmt(&self, stmt: &BreakStmt) -> Result<(), LoxError> {
    todo!()
  }

  fn visit_expression_stmt(&self, stmt: &ExpressionStmt) -> Result<(), LoxError> {
    todo!()
  }

  fn visit_function_stmt(&self, stmt: &FunctionStmt) -> Result<(), LoxError> {
    todo!()
  }

  fn visit_if_stmt(&self, stmt: &IfStmt) -> Result<(), LoxError> {
    todo!()
  }

  fn visit_print_stmt(&self, stmt: &PrintStmt) -> Result<(), LoxError> {
    todo!()
  }

  fn visit_return_stmt(&self, stmt: &ReturnStmt) -> Result<(), LoxError> {
    todo!()
  }

  fn visit_var_stmt(&self, stmt: &VarStmt) -> Result<(), LoxError> {
    self.declare(&stmt.name);
    if let Some(i) = &stmt.initialiser {
      self.resolve_expr(i)?;
    }
    self.define(&stmt.name);
    Ok(())
  }

  fn visit_while_stmt(&self, stmt: &WhileStmt) -> Result<(), LoxError> {
    todo!()
  }
}
