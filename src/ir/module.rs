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

  pub fn functions_get(&self, fun_id: FunctionId) -> &Function {
    self.functions.get(fun_id).unwrap()
  }

  pub fn functions_get_mut(&mut self, fun_id: FunctionId) -> &mut Function {
    self.functions.get_mut(fun_id).unwrap()
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

  pub fn replace_function(&mut self, fun_id: FunctionId, fun: Function) {
    *self.functions.get_mut(fun_id).unwrap() = fun;
  }
}
