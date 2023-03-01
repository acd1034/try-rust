use std::fmt;

#[derive(Clone, Debug, PartialEq)]
pub enum Type {
  Int,
  Pointer(Box<Type>),
  Array(Box<Type>, u32),
}

impl fmt::Display for Type {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      Type::Int => write!(f, "int"),
      _ => todo!(),
    }
  }
}
