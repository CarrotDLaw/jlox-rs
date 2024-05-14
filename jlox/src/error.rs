pub struct LoxError {
  line: usize,
  message: String,
}

impl LoxError {
  fn new(line: usize, message: &str) -> LoxError {
    LoxError {
      line,
      message: message.to_string(),
    }
  }

  pub fn error(line: usize, message: &str) -> LoxError {
    let err = LoxError::new(line, message);
    err.report("");
    err
  }

  pub fn report(&self, location: &str) {
    eprintln!("[line {}] Error{}: {}", self.line, location, self.message);
  }
}
