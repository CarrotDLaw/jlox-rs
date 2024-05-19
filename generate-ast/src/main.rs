use std::{
  env::args,
  fs::File,
  io::{self, Write},
  process::exit,
};

struct TreeType {
  class_name: String,
  fields: Vec<String>,
}

fn main() -> io::Result<()> {
  let args = args().collect::<Vec<String>>();
  if args.len() != 2 {
    eprintln!("Usage: generate_ast <output directory>");
    exit(64);
  }
  let output_dir = args.get(1).expect("Error reading input.");

  define_ast(
    output_dir,
    "Expr",
    &["error", "token"],
    &[
      "Assign   : Token name, Rc<Expr> value",
      "Binary   : Rc<Expr> left, Token operator, Rc<Expr> right",
      "Grouping : Rc<Expr> expression",
      "Literal  : Option<Object> value",
      "Logical  : Rc<Expr> left, Token operator, Rc<Expr> right",
      "Unary    : Token operator, Rc<Expr> right",
      "Variable : Token name",
    ],
  )?;

  define_ast(
    output_dir,
    "Stmt",
    &["error", "expr", "token"],
    &[
      "Block      : Vec<Rc<Stmt>> statements",
      "Expression : Rc<Expr> expression",
      "If         : Rc<Expr> condition, Rc<Stmt> then_branch, Option<Rc<Stmt>> else_branch",
      "Print      : Rc<Expr> expression",
      "Var        : Token name, Option<Rc<Expr>> initialiser",
    ],
  )?;

  Ok(())
}

fn define_ast(
  output_dir: &str,
  base_name: &str,
  imports: &[&str],
  token_types: &[&str],
) -> io::Result<()> {
  let path = format!("{}/{}.rs", output_dir, base_name.to_lowercase());
  let mut file = File::create(path)?;
  let mut tree_types = Vec::new();

  writeln!(file, "use std::rc::Rc;")?;
  writeln!(file)?;
  for import in imports {
    writeln!(file, "use crate::{import}::*;")?;
  }

  for token_type in token_types {
    let (class_name, fields) = token_type.split_once(':').unwrap();
    let class_name = class_name.trim().to_string();
    let fields = fields
      .trim()
      .split(", ")
      .map(|x| {
        let (field_type, field_name) = x.trim().split_once(' ').unwrap();
        format!("{field_name}: {field_type}")
      })
      .collect::<Vec<String>>();
    tree_types.push(TreeType { class_name, fields })
  }

  writeln!(file)?;
  writeln!(file, "pub enum {base_name} {{")?;
  for tree_type in &tree_types {
    writeln!(file, "  {0}(Rc<{0}{1}>),", tree_type.class_name, base_name)?;
  }
  writeln!(file, "}}")?;

  writeln!(file)?;
  writeln!(file, "impl {base_name} {{")?;
  writeln!(
    file,
    "  pub fn accept<T>(&self, {}_visitor: &dyn {}Visitor<T>) -> Result<T, LoxError> {{",
    base_name.to_lowercase(),
    base_name
  )?;
  writeln!(file, "    match self {{")?;
  for tree_type in &tree_types {
    writeln!(
      file,
      "      {0}::{1}({2}) => {2}_visitor.visit_{3}_{2}({2}),",
      base_name,
      tree_type.class_name,
      base_name.to_lowercase(),
      tree_type.class_name.to_lowercase(),
    )?;
  }
  writeln!(file, "    }}")?;
  writeln!(file, "  }}")?;
  writeln!(file, "}}")?;

  for tree_type in &tree_types {
    writeln!(file)?;
    writeln!(file, "pub struct {}{} {{", tree_type.class_name, base_name)?;
    for field in &tree_type.fields {
      writeln!(file, "  pub {field},")?;
    }
    writeln!(file, "}}")?;
  }

  writeln!(file)?;
  writeln!(file, "pub trait {base_name}Visitor<T> {{")?;
  for tree_type in &tree_types {
    writeln!(
      file,
      "  fn visit_{0}_{1}(&self, {1}: &{2}{3}) -> Result<T, LoxError>;",
      tree_type.class_name.to_lowercase(),
      base_name.to_lowercase(),
      tree_type.class_name,
      base_name
    )?;
  }
  writeln!(file, "}}")?;

  Ok(())
}
