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
    let count = count_ops(fun);
    eprintln!("Name of function: {}", fun.name());
    eprintln!("  # of ops: {}", count);
  }
}

fn count_ops(fun: &Function) -> usize {
  let mut count = 0;
  let mut vis = Visitor::new(fun);
  while let Some(block_id) = vis.next_block() {
    count += vis.function().get(block_id).insts().len();
  }
  count
}

// ----- DeadCodeElimination -----

pub struct DeadCodeElimination {
  module: Module,
}

impl DeadCodeElimination {
  pub fn new(module: Module) -> DeadCodeElimination {
    DeadCodeElimination { module }
  }

  pub fn run(mut self) -> Module {
    let fun_ids = self.module.function_ids();
    for fun_id in fun_ids {
      let fun = self.run_on_function(self.module.get_function(fun_id).clone());
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

#[test]
fn test_dead_code_elimination() {
  use crate::irgen::IRGen;
  use crate::parse::parse;
  use crate::tokenize::Tokenizer;

  let input = r"
int main() {
  int x=0;
  x+1+2+3;
  return x;
}
  ";
  let it = Tokenizer::new(input);
  let funs = parse(it).unwrap();
  let module = IRGen::new("mod".to_string()).irgen(funs).unwrap();

  let fun_id = module.get_function_by_name("main").unwrap();
  let before = count_ops(module.get_function(fun_id));
  let module = DeadCodeElimination::new(module).run();
  let after = count_ops(module.get_function(fun_id));
  assert!(after < before);
}
