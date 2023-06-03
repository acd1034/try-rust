use crate::ir::{
  builder::*, builder_trait::*, function::*, inst::is_dead, module::*, visitor::*, visitor_trait::*,
};
use std::collections::HashMap;

// ----- OpCountPrinter -----

pub struct OpCountPrinter<'a> {
  module: &'a Module,
}

#[allow(dead_code)]
impl<'a> OpCountPrinter<'a> {
  pub fn new(module: &Module) -> OpCountPrinter {
    OpCountPrinter { module }
  }

  pub fn run(&self) {
    for (_id, fun) in self.module.functions() {
      self.run_on_function(fun);
    }
  }

  fn run_on_function(&self, fun: &Function) {
    let mut count = 0;
    let mut vis = Visitor::new(fun);
    while let Some(..) = vis.next_block() {
      while let Some(..) = vis.next_inst() {
        count += 1;
      }
    }
    eprintln!("Name of function: {}", vis.function().name());
    eprintln!("# of ops: {}", count);
  }
}

// ----- DeadCodeElimination -----

pub struct DeadCodeElimination {
  module: Module,
}

#[allow(dead_code)]
impl DeadCodeElimination {
  pub fn new(module: Module) -> DeadCodeElimination {
    DeadCodeElimination { module }
  }

  pub fn run(mut self) -> Module {
    let fun_ids: Vec<_> = self
      .module
      .functions()
      .iter()
      .map(|(id, _fun)| id)
      .collect();
    for fun_id in fun_ids {
      let fun = self.run_on_function(self.module.functions_get(fun_id).clone());
      self.module.replace_function(fun_id, fun);
    }
    self.module
  }

  fn run_on_function(&mut self, fun: Function) -> Function {
    let mut deadness = HashMap::new();
    let mut builder = Builder::new(fun);
    while let Some(..) = builder.prev_block() {
      while let Some(inst_id) = builder.prev_inst() {
        if is_dead(builder.function().get(inst_id), &mut deadness) {
          builder.remove_inst();
        }
      }
    }
    builder.retrieve_function()
  }
}
