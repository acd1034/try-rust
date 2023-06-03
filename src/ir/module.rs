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

  pub fn get_function(&self, fun_id: FunctionId) -> &Function {
    self.functions.get(fun_id).unwrap()
  }

  // pub fn get_function_mut(&mut self, fun_id: FunctionId) -> &mut Function {
  //   self.functions.get_mut(fun_id).unwrap()
  // }

  pub fn function_ids(&self) -> Vec<FunctionId> {
    self.functions.iter().map(|(id, _fun)| id).collect()
  }

  // ----- function -----

  pub fn get_function_by_name(&self, name: &str) -> Option<FunctionId> {
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
