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
      "Call     : Rc<Expr> callee, Token bracket, Rc<Vec<Rc<Expr>>> arguments",
      "Grouping : Rc<Expr> expression",
      "Literal  : Option<Literal> value",
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
      "Block      : Rc<Vec<Rc<Stmt>>> statements",
      "Break      : Token token",
      "Expression : Rc<Expr> expression",
      "Function   : Token name, Vec<Token> params, Rc<Vec<Rc<Stmt>>> body",
      "If         : Rc<Expr> condition, Rc<Stmt> then_branch, Option<Rc<Stmt>> else_branch",
      "Print      : Rc<Expr> expression",
      "Return     : Token keyword, Option<Rc<Expr>> value",
      "Var        : Token name, Option<Rc<Expr>> initialiser",
      "While      : Rc<Expr> condition, Rc<Stmt> body",
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

  writeln!(file, "use std::hash::Hash;")?;
  writeln!(file, "use std::hash::Hasher;")?;
  writeln!(file, "use std::mem::discriminant;")?;
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
  writeln!(file, "#[derive(Debug)]")?;
  writeln!(file, "pub enum {base_name} {{")?;
  for tree_type in tree_types.iter() {
    writeln!(file, "  {0}(Rc<{0}{1}>),", tree_type.class_name, base_name)?;
  }
  writeln!(file, "}}")?;

  writeln!(file)?;
  writeln!(file, "impl {base_name} {{")?;
  writeln!(
    file,
    "  pub fn accept<T>(&self, wrapper: &Rc<{0}>, {1}_visitor: &dyn {0}Visitor<T>) -> Result<T, LoxError> {{",
    base_name,
    base_name.to_lowercase(),
  )?;
  writeln!(file, "    match self {{")?;
  for tree_type in tree_types.iter() {
    writeln!(
      file,
      "      {0}::{1}({2}) => {2}_visitor.visit_{3}_{2}(wrapper, {2}),",
      base_name,
      tree_type.class_name,
      base_name.to_lowercase(),
      tree_type.class_name.to_lowercase(),
    )?;
  }
  writeln!(file, "    }}")?;
  writeln!(file, "  }}")?;
  writeln!(file, "}}")?;

  writeln!(file)?;
  writeln!(file, "impl PartialEq for {base_name} {{")?;
  writeln!(file, "  fn eq(&self, other: &{base_name}) -> bool {{")?;
  writeln!(file, "    match (self, other) {{")?;
  for tree_type in tree_types.iter() {
    writeln!(
      file,
      "      ({0}::{1}(l0), {0}::{1}(r0)) => Rc::ptr_eq(l0, r0),",
      base_name, tree_type.class_name
    )?;
  }
  writeln!(file, "      _ => false,")?;
  writeln!(file, "    }}")?;
  writeln!(file, "  }}")?;
  writeln!(file, "}}")?;

  writeln!(file)?;
  writeln!(file, "impl Eq for {base_name} {{}}")?;

  writeln!(file)?;
  writeln!(file, "impl Hash for {base_name} {{")?;
  writeln!(file, "  fn hash<H: Hasher>(&self, state: &mut H) {{")?;
  // writeln!(file, "    match self {{")?;
  // for tree_type in tree_types.iter() {
  //   writeln!(
  //     file,
  //     "       {0}::{1}({2}) => state.write_usize(Rc::as_ptr({2}) as usize),",
  //     base_name,
  //     tree_type.class_name,
  //     base_name.chars().next().unwrap().to_lowercase()
  //   )?;
  // }
  // writeln!(file, "    }}")?;
  writeln!(file, "    discriminant(self).hash(state);")?;
  writeln!(file, "  }}")?;
  writeln!(file, "}}")?;

  for tree_type in tree_types.iter() {
    writeln!(file)?;
    writeln!(file, "#[derive(Debug)]")?;
    writeln!(file, "pub struct {}{} {{", tree_type.class_name, base_name)?;
    for field in tree_type.fields.iter() {
      writeln!(file, "  pub {field},")?;
    }
    writeln!(file, "}}")?;
  }

  writeln!(file)?;
  writeln!(file, "pub trait {base_name}Visitor<T> {{")?;
  for tree_type in tree_types.iter() {
    writeln!(
      file,
      "  fn visit_{0}_{1}(&self, wrapper: &Rc<{2}>, {1}: &{3}{2}) -> Result<T, LoxError>;",
      tree_type.class_name.to_lowercase(),
      base_name.to_lowercase(),
      base_name,
      tree_type.class_name,
    )?;
  }
  writeln!(file, "}}")?;

  Ok(())
}
