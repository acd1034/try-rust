pub mod c;
use crate::ir::module::Module;
use std::fmt;

pub enum Target {
  C(Module),
}

impl fmt::Display for Target {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      Target::C(module) => c::codegen(f, module),
    }
  }
}
