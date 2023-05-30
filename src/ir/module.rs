use crate::ir::function::*;
use id_arena::*;

pub struct Module {
  name: String,
  functions: Arena<Function>,
}

impl Module {
  pub fn new(name: String) -> Module {
    Module {
      name,
      functions: Arena::new(),
    }
  }

  pub fn name(&self) -> &str {
    &self.name
  }

  pub fn functions(&self) -> &Arena<Function> {
    &self.functions
  }

  // ----- function -----

  pub fn get_function(&self, name: &str) -> Option<FunctionId> {
    self
      .functions
      .iter()
      .find(|(_id, fun)| fun.name() == name)
      .map(|(id, _fun)| id)
  }

  pub fn add_function(&mut self, fun: Function) -> FunctionId {
    self.functions.alloc(fun)
  }
}
