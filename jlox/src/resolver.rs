use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{error::*, expr::*, interpreter::*, stmt::*, token::*};

pub struct Resolver {
  interpreter: Interpreter,
  scopes: RefCell<Vec<RefCell<HashMap<String, bool>>>>,
  current_function: RefCell<FunctionType>,
  had_error: RefCell<bool>,
}

impl Resolver {
  pub fn new(interpreter: &Interpreter) -> Resolver {
    Resolver {
      interpreter: interpreter.clone(),
      scopes: RefCell::new(Vec::new()),
      current_function: RefCell::new(FunctionType::None),
      had_error: RefCell::new(false),
    }
  }

  pub fn resolve(&self, statements: &Rc<&[Rc<Stmt>]>) -> Result<(), LoxError> {
    for statement in statements.iter() {
      self.resolve_stmt(statement)?;
    }

    if *self.had_error.borrow() {
      return Err(LoxError::system_error(""));
    }

    Ok(())
  }

  fn resolve_expr(&self, expr: &Rc<Expr>) -> Result<(), LoxError> {
    expr.accept(&expr.clone(), self)
  }

  fn resolve_stmt(&self, stmt: &Rc<Stmt>) -> Result<(), LoxError> {
    stmt.accept(&stmt.clone(), self)
  }

  fn begin_scope(&self) {
    self.scopes.borrow_mut().push(RefCell::new(HashMap::new()))
  }

  fn end_scope(&self) {
    self.scopes.borrow_mut().pop();
  }

  fn declare(&self, name: &Token) {
    if let Some(s) = self.scopes.borrow().last() {
      if s.borrow().contains_key(name.get_lexeme()) {
        self.had_error.replace(true);
        LoxError::runtime_error(name, "Already a variable with this name in this scope.");
      }

      s.borrow_mut().insert(name.get_lexeme().to_string(), false);
    }
  }

  fn define(&self, name: &Token) {
    if let Some(s) = self.scopes.borrow().last() {
      s.borrow_mut().insert(name.get_lexeme().to_string(), true);
    }
  }

  fn resolve_function(
    &self,
    function: &FunctionStmt,
    function_type: &FunctionType,
  ) -> Result<(), LoxError> {
    let enclosing_function = self.current_function.replace(function_type.clone());
    self.begin_scope();

    for param in &function.params {
      self.declare(param);
      self.define(param);
    }

    self.resolve(&function.body.as_slice().into())?;
    self.end_scope();
    self.current_function.replace(enclosing_function);
    Ok(())
  }

  fn resolve_local(&self, expr: &Rc<Expr>, name: &Token) {
    for (scope, map) in self.scopes.borrow().iter().rev().enumerate() {
      if map.borrow().contains_key(name.get_lexeme()) {
        self.interpreter.resolve(expr, scope);
      }
    }
  }
}

impl ExprVisitor<()> for Resolver {
  fn visit_assign_expr(&self, wrapper: &Rc<Expr>, expr: &AssignExpr) -> Result<(), LoxError> {
    self.resolve_expr(&expr.value)?;
    self.resolve_local(wrapper, &expr.name);
    Ok(())
  }

  fn visit_binary_expr(&self, _wrapper: &Rc<Expr>, expr: &BinaryExpr) -> Result<(), LoxError> {
    self.resolve_expr(&expr.left)?;
    self.resolve_expr(&expr.right)?;
    Ok(())
  }

  fn visit_call_expr(&self, _wrapper: &Rc<Expr>, expr: &CallExpr) -> Result<(), LoxError> {
    self.resolve_expr(&expr.callee)?;

    for argument in expr.arguments.iter() {
      self.resolve_expr(argument)?;
    }

    Ok(())
  }

  fn visit_grouping_expr(&self, _wrapper: &Rc<Expr>, expr: &GroupingExpr) -> Result<(), LoxError> {
    self.resolve_expr(&expr.expression)
  }

  fn visit_literal_expr(&self, _wrapper: &Rc<Expr>, _expr: &LiteralExpr) -> Result<(), LoxError> {
    Ok(())
  }

  fn visit_logical_expr(&self, _wrapper: &Rc<Expr>, expr: &LogicalExpr) -> Result<(), LoxError> {
    self.resolve_expr(&expr.left)?;
    self.resolve_expr(&expr.right)?;
    Ok(())
  }

  fn visit_unary_expr(&self, _wrapper: &Rc<Expr>, expr: &UnaryExpr) -> Result<(), LoxError> {
    self.resolve_expr(&expr.right)
  }

  fn visit_variable_expr(&self, wrapper: &Rc<Expr>, expr: &VariableExpr) -> Result<(), LoxError> {
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

    self.resolve_local(wrapper, &expr.name);
    Ok(())
  }
}

impl StmtVisitor<()> for Resolver {
  fn visit_block_stmt(&self, _wrapper: &Rc<Stmt>, stmt: &BlockStmt) -> Result<(), LoxError> {
    self.begin_scope();
    self.resolve(&stmt.statements.as_slice().into())?;
    self.end_scope();
    Ok(())
  }

  fn visit_break_stmt(&self, _wrapper: &Rc<Stmt>, _stmt: &BreakStmt) -> Result<(), LoxError> {
    Ok(())
  }

  fn visit_expression_stmt(
    &self,
    _wrapper: &Rc<Stmt>,
    stmt: &ExpressionStmt,
  ) -> Result<(), LoxError> {
    self.resolve_expr(&stmt.expression)
  }

  fn visit_function_stmt(&self, _wrapper: &Rc<Stmt>, stmt: &FunctionStmt) -> Result<(), LoxError> {
    self.declare(&stmt.name);
    self.define(&stmt.name);

    self.resolve_function(stmt, &FunctionType::Function)?;
    Ok(())
  }

  fn visit_if_stmt(&self, _wrapper: &Rc<Stmt>, stmt: &IfStmt) -> Result<(), LoxError> {
    self.resolve_expr(&stmt.condition)?;
    self.resolve_stmt(&stmt.then_branch)?;

    if let Some(b) = &stmt.else_branch {
      self.resolve_stmt(b)?;
    }

    Ok(())
  }

  fn visit_print_stmt(&self, _wrapper: &Rc<Stmt>, stmt: &PrintStmt) -> Result<(), LoxError> {
    self.resolve_expr(&stmt.expression)
  }

  fn visit_return_stmt(&self, _wrapper: &Rc<Stmt>, stmt: &ReturnStmt) -> Result<(), LoxError> {
    if self.current_function.borrow().is_type(&FunctionType::None) {
      self.had_error.replace(true);
      LoxError::runtime_error(&stmt.keyword, "Can't return from top-level code.");
    }

    if let Some(v) = &stmt.value {
      self.resolve_expr(v)?;
    }

    Ok(())
  }

  fn visit_var_stmt(&self, _wrapper: &Rc<Stmt>, stmt: &VarStmt) -> Result<(), LoxError> {
    self.declare(&stmt.name);

    if let Some(i) = &stmt.initialiser {
      self.resolve_expr(i)?;
    }

    self.define(&stmt.name);
    Ok(())
  }

  fn visit_while_stmt(&self, _wrapper: &Rc<Stmt>, stmt: &WhileStmt) -> Result<(), LoxError> {
    self.resolve_expr(&stmt.condition)?;
    self.resolve_stmt(&stmt.body)?;
    Ok(())
  }
}

#[derive(Clone, PartialEq)]
enum FunctionType {
  Function,
  None,
}

impl FunctionType {
  fn is_type(&self, function_type: &FunctionType) -> bool {
    self.eq(function_type)
  }
}
