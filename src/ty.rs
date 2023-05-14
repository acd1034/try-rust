use std::fmt;

#[derive(Clone, Debug, PartialEq)]
pub enum Type {
  Int,
  Char,
  Pointer(Box<Type>),
  Array(Box<Type>, u32),
  FunTy(Box<Type>, Vec<Type>, Vec<String>),
}

impl fmt::Display for Type {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      Type::Int => write!(f, "int"),
      Type::Char => write!(f, "char"),
      _ => todo!(),
    }
  }
}
