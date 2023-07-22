use crate::ir::{function::*, module::*, visitor::*, visitor_trait::*};

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

pub fn count_ops(fun: &Function) -> usize {
  let mut count = 0;
  let mut vis = Visitor::new(fun);
  while let Some(block_id) = vis.next_block() {
    count += vis.function().get(block_id).insts().len();
  }
  count
}

#[test]
fn test_count_ops() {
  use crate::ir::builder;
  use crate::ir::builder_trait::BuilderTrait;
  use crate::ir::function;
  use parser::ty::Type;

  let fun = function::Function::new("fun".to_string(), Type::Int, Vec::new());
  let mut builder = builder::Builder::new(fun);
  let entry_block = builder.append_basic_block();
  let next_block = builder.append_basic_block();

  builder.position_at_end(entry_block);
  let v1 = builder.build_const(42);
  let v2 = builder.build_const(1);
  let v3 = builder.build_add(v1, v2);
  builder.build_unconditional_branch(next_block);

  builder.position_at_end(next_block);
  let v4 = builder.build_add(v3, v2);
  builder.build_return(v4);

  let fun = builder.retrieve_function();
  assert_eq!(6, count_ops(&fun));
}
