use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{
  environment::*, error::*, expr::*, lox_callable::*, lox_class::*, lox_function::*,
  lox_native_function::*, stmt::*, token::*,
};

#[derive(Default, Clone)]
pub struct Interpreter {
  globals: Rc<RefCell<Environment>>,
  environment: RefCell<Rc<RefCell<Environment>>>,
  locals: RefCell<HashMap<Rc<Expr>, usize>>,
}

impl Interpreter {
  pub fn new() -> Interpreter {
    let globals = Rc::new(RefCell::new(Environment::new()));

    globals.borrow_mut().define(
      "clock",
      Literal::NativeFunction(
        LoxNativeFunction {
          fun: Rc::new(Clock),
        }
        .into(),
      ),
    );

    Interpreter {
      globals: globals.clone(),
      environment: RefCell::new(globals),
      locals: RefCell::new(HashMap::new()),
    }
  }

  pub fn interpret(&self, statements: &Rc<&[Rc<Stmt>]>) -> Result<(), LoxError> {
    for statement in statements.iter() {
      self.execute(statement)?;
    }

    Ok(())
  }

  pub fn get_globals(&self) -> &Rc<RefCell<Environment>> {
    &self.globals
  }

  pub fn print_environment(&self) {
    dbg!(&self.environment);
  }

  fn evaluate(&self, expr: &Rc<Expr>) -> Result<Literal, LoxError> {
    expr.accept(expr, self)
  }

  fn execute(&self, stmt: &Rc<Stmt>) -> Result<(), LoxError> {
    stmt.accept(stmt, self)
  }

  pub fn resolve(&self, expr: &Rc<Expr>, depth: usize) {
    self.locals.borrow_mut().insert(expr.clone(), depth);
  }

  fn look_up_variable(&self, name: &Token, expr: &Rc<Expr>) -> Result<Literal, LoxError> {
    if let Some(&distance) = self.locals.borrow().get(expr) {
      return self
        .environment
        .borrow()
        .borrow()
        .get_at(distance, name.get_lexeme());
    }

    self.globals.borrow().get(name)
  }

  pub fn execute_block(
    &self,
    statements: &Rc<&[Rc<Stmt>]>,
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
  fn visit_assign_expr(&self, wrapper: &Rc<Expr>, expr: &AssignExpr) -> Result<Literal, LoxError> {
    let value = self.evaluate(&expr.value)?;
    if let Some(&distance) = self.locals.borrow().get(wrapper) {
      self
        .environment
        .borrow()
        .borrow_mut()
        .assign_at(distance, &expr.name, &value)?;
    } else {
      self.globals.borrow_mut().assign(&expr.name, &value)?;
    }

    Ok(value)
  }

  fn visit_binary_expr(&self, _wrapper: &Rc<Expr>, expr: &BinaryExpr) -> Result<Literal, LoxError> {
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

  fn visit_call_expr(&self, _wrapper: &Rc<Expr>, expr: &CallExpr) -> Result<Literal, LoxError> {
    let callee = self.evaluate(&expr.callee)?;

    let mut arguments = Vec::new();
    for argument in expr.arguments.as_slice() {
      arguments.push(self.evaluate(argument)?);
    }

    let (function, class): (Option<Rc<dyn LoxCallable>>, Option<Rc<LoxClass>>) = match callee {
      Literal::Function(f) => (Some(f), None),
      Literal::Class(c) => (Some(c.clone()), Some(c.clone())),
      _ => (None, None),
    };

    if let Some(callable) = function {
      if arguments.len().ne(&(callable.arity() as usize)) {
        return Err(LoxError::runtime_error(
          &expr.bracket,
          &format!(
            "Expected {} arguments but got {}.",
            callable.arity(),
            arguments.len()
          ),
        ));
      }

      return callable.call(self, &arguments, class);
    }

    Err(LoxError::runtime_error(
      &expr.bracket,
      "Can only call functions and classes.",
    ))
  }

  fn visit_get_expr(&self, _wrapper: &Rc<Expr>, expr: &GetExpr) -> Result<Literal, LoxError> {
    let object = self.evaluate(&expr.object)?;
    if let Literal::Instance(i) = object {
      return i.get(&expr.name, &i);
    }

    Err(LoxError::runtime_error(
      &expr.name,
      "Only instances have properties.",
    ))
  }

  fn visit_grouping_expr(
    &self,
    _wrapper: &Rc<Expr>,
    expr: &GroupingExpr,
  ) -> Result<Literal, LoxError> {
    self.evaluate(&expr.expression)
  }

  fn visit_literal_expr(
    &self,
    _wrapper: &Rc<Expr>,
    expr: &LiteralExpr,
  ) -> Result<Literal, LoxError> {
    if let Some(v) = &expr.value {
      return Ok(v.clone());
    }

    unreachable!()
  }

  fn visit_logical_expr(
    &self,
    _wrapper: &Rc<Expr>,
    expr: &LogicalExpr,
  ) -> Result<Literal, LoxError> {
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

  fn visit_set_expr(&self, _wrapper: &Rc<Expr>, expr: &SetExpr) -> Result<Literal, LoxError> {
    let object = self.evaluate(&expr.object)?;

    if let Literal::Instance(i) = object {
      let value = self.evaluate(&expr.value)?;
      i.set(&expr.name, &value);
      return Ok(value);
    }

    Err(LoxError::runtime_error(
      &expr.name,
      "Only instances have fields.",
    ))
  }

  fn visit_super_expr(&self, wrapper: &Rc<Expr>, expr: &SuperExpr) -> Result<Literal, LoxError> {
    let distance = self
      .locals
      .borrow()
      .get(wrapper)
      .map_or_else(|| 0_usize, |&v| v);
    let superclass = self
      .environment
      .borrow()
      .borrow()
      .get_at(distance, "super")?;
    let superclass = if let Literal::Class(c) = superclass {
      c
    } else {
      panic!()
    };
    let object = self
      .environment
      .borrow()
      .borrow()
      .get_at(distance - 1, "this")?;
    let method = superclass
      .find_method(expr.method.get_lexeme())
      .ok_or_else(|| {
        LoxError::runtime_error(
          &expr.method,
          &format!("Undefined property '{}'.", expr.method.get_lexeme()),
        )
      })?;
    if let Literal::Function(f) = method {
      return Ok(f.bind(&object));
    }

    unreachable!()
  }

  fn visit_this_expr(&self, wrapper: &Rc<Expr>, expr: &ThisExpr) -> Result<Literal, LoxError> {
    self.look_up_variable(&expr.keyword, wrapper)
  }

  fn visit_unary_expr(&self, _wrapper: &Rc<Expr>, expr: &UnaryExpr) -> Result<Literal, LoxError> {
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

  fn visit_variable_expr(
    &self,
    wrapper: &Rc<Expr>,
    expr: &VariableExpr,
  ) -> Result<Literal, LoxError> {
    self.look_up_variable(&expr.name, wrapper)
  }
}

impl StmtVisitor<()> for Interpreter {
  fn visit_block_stmt(&self, _wrapper: &Rc<Stmt>, stmt: &BlockStmt) -> Result<(), LoxError> {
    let environment = Environment::new_with_enclosing(&self.environment.borrow().clone());
    self.execute_block(&stmt.statements.as_slice().into(), environment)
  }

  fn visit_break_stmt(&self, _wrapper: &Rc<Stmt>, _stmt: &BreakStmt) -> Result<(), LoxError> {
    Err(LoxError::new_break())
  }

  fn visit_class_stmt(&self, _wrapper: &Rc<Stmt>, stmt: &ClassStmt) -> Result<(), LoxError> {
    let superclass = if let Some(s) = &stmt.superclass {
      let superclass = self.evaluate(s)?;
      if let Literal::Class(c) = superclass {
        Some(c)
      } else if let Expr::Variable(v) = s.as_ref() {
        return Err(LoxError::runtime_error(
          &v.name,
          "Superclass must be a class.",
        ));
      } else {
        unreachable!()
      }
    } else {
      None
    };

    self
      .environment
      .borrow()
      .borrow_mut()
      .define(stmt.name.get_lexeme(), Literal::Nil);

    let enclosing = if let Some(s) = superclass.as_ref() {
      let mut environment = Environment::new_with_enclosing(&self.environment.borrow().clone());
      environment.define("super", Literal::Class(s.clone()));
      Some(self.environment.replace(RefCell::new(environment).into()))
    } else {
      None
    };

    let mut methods = HashMap::new();
    for method in stmt.methods.iter() {
      if let Stmt::Function(m) = method.as_ref() {
        let function = Literal::Function(
          LoxFunction::new(
            &self.environment.borrow(),
            m,
            m.name.get_lexeme().eq("init"),
          )
          .into(),
        );
        methods.insert(m.name.get_lexeme().to_string(), function);
      }
    }

    let class = Literal::Class(LoxClass::new(stmt.name.get_lexeme(), &superclass, &methods).into());

    if let Some(e) = enclosing {
      self.environment.replace(e);
    }

    self
      .environment
      .borrow()
      .borrow_mut()
      .assign(&stmt.name, &class)
  }

  fn visit_expression_stmt(
    &self,
    _wrapper: &Rc<Stmt>,
    stmt: &ExpressionStmt,
  ) -> Result<(), LoxError> {
    self.evaluate(&stmt.expression)?;
    Ok(())
  }

  fn visit_function_stmt(&self, _wrapper: &Rc<Stmt>, stmt: &FunctionStmt) -> Result<(), LoxError> {
    let function = LoxFunction::new(&self.environment.borrow(), stmt, false);

    self
      .environment
      .borrow()
      .borrow_mut()
      .define(stmt.name.get_lexeme(), Literal::Function(function.into()));

    Ok(())
  }

  fn visit_if_stmt(&self, _wrapper: &Rc<Stmt>, stmt: &IfStmt) -> Result<(), LoxError> {
    if self.is_truthy(&self.evaluate(&stmt.condition)?) {
      return self.execute(&stmt.then_branch);
    }

    if let Some(b) = &stmt.else_branch {
      return self.execute(b);
    }

    Ok(())
  }

  fn visit_print_stmt(&self, _wrapper: &Rc<Stmt>, stmt: &PrintStmt) -> Result<(), LoxError> {
    println!("{}", self.evaluate(&stmt.expression)?);
    Ok(())
  }

  fn visit_return_stmt(&self, _wrapper: &Rc<Stmt>, stmt: &ReturnStmt) -> Result<(), LoxError> {
    if let Some(v) = &stmt.value {
      Err(LoxError::new_return(&self.evaluate(v)?))
    } else {
      Err(LoxError::new_return(&Literal::Nil))
    }
  }

  fn visit_var_stmt(&self, _wrapper: &Rc<Stmt>, stmt: &VarStmt) -> Result<(), LoxError> {
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

  fn visit_while_stmt(&self, _wrapper: &Rc<Stmt>, stmt: &WhileStmt) -> Result<(), LoxError> {
    while self.is_truthy(&self.evaluate(&stmt.condition)?) {
      let mut body = self.execute(&stmt.body);
      if body.as_mut().is_err_and(|e| e.is_break()) {
        break;
      }

      body?;
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
        .collect::<Vec<Rc<Stmt>>>()
        .as_slice()
        .into(),
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
        .collect::<Vec<Rc<Stmt>>>()
        .as_slice()
        .into(),
    )?;

    Ok(())
  }
}
