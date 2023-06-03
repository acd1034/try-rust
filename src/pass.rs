use crate::ir::{function::*, module::*, visitor::*, visitor_trait::*};

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

  pub fn run_on_function(&self, fun: &Function) {
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
