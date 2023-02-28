pub mod c;
pub mod ll;
use crate::ir::Mod;
use std::fmt;

pub enum Target {
  C(Mod),
}

impl fmt::Display for Target {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      Target::C(module) => c::codegen(f, module),
    }
  }
}
