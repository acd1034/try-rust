use crate::ir::block::*;
use crate::ir::builder_trait::*;
use crate::ir::function::*;

pub struct Builder<'a> {
  function: &'a mut Function,
  insert_block: Option<BlockId>,
}

impl<'a> Builder<'a> {
  pub fn new(function: &'a mut Function) -> Builder<'a> {
    Builder {
      function,
      insert_block: None,
    }
  }
}

impl<'a> BuilderTrait for Builder<'a> {
  fn function(&self) -> &Function {
    &self.function
  }

  fn function_mut(&mut self) -> &mut Function {
    &mut self.function
  }

  fn insert_block(&self) -> BlockId {
    self.insert_block.unwrap()
  }

  fn position_at_end(&mut self, block_id: BlockId) {
    self.insert_block = Some(block_id);
  }
}

#[test]
fn test_ir_builder() {
  use crate::ir::function;
  use crate::ir::inst;
  use crate::ir::module;
  use crate::ty::Type;

  let mut module = module::Module::new("module".to_string());
  let mut fun = function::Function::new("fun".to_string(), Type::Int, Vec::new());
  let v3 = {
    let mut builder = Builder::new(&mut fun);
    let entry_block = builder.append_basic_block();
    builder.position_at_end(entry_block);
    let v1 = builder.build_const(42);
    let v2 = builder.build_const(1);
    builder.build_add(v1, v2)
  };
  assert!(matches!(fun.get(v3).kind(), inst::InstKind::Add(..)));
  module.add_function(fun);
  eprintln!("{}", module);
}
