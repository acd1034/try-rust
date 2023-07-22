use crate::ir::block::*;
use crate::ir::builder_trait::*;
use crate::ir::function::*;
use crate::ir::inst::*;
use crate::ir::visitor_trait::*;

pub struct Builder {
  function: Function,
  insert_block: Option<BlockId>,
  insert_point: InsertPoint,
}

impl Builder {
  pub fn new(function: Function) -> Builder {
    Builder {
      function,
      insert_block: None,
      insert_point: InsertPoint::Nowhere,
    }
  }

  pub fn retrieve_function(self) -> Function {
    self.function
  }
}

impl VisitorTrait for Builder {
  fn function(&self) -> &Function {
    &self.function
  }

  fn get_insert_block(&self) -> Option<BlockId> {
    self.insert_block
  }

  fn get_insert_point(&self) -> InsertPoint {
    self.insert_point
  }

  fn clear_position(&mut self) {
    self.insert_block = None;
    self.insert_point = InsertPoint::Nowhere;
  }

  fn position_before(&mut self, block_id: BlockId) {
    self.insert_block = Some(block_id);
    self.insert_point = InsertPoint::Before;
  }

  fn position_at(&mut self, block_id: BlockId, inst_id: InstId) {
    self.insert_block = Some(block_id);
    self.insert_point = InsertPoint::At(inst_id);
  }

  fn position_at_end(&mut self, block_id: BlockId) {
    self.insert_block = Some(block_id);
    self.insert_point = InsertPoint::After;
  }
}

impl BuilderTrait for Builder {
  fn function_mut(&mut self) -> &mut Function {
    &mut self.function
  }
}

#[test]
fn test_ir_builder() {
  use crate::ir::function;
  use crate::ir::inst;
  use parser::ty::Type;

  let fun = function::Function::new("fun".to_string(), Type::Int, Vec::new());
  let mut builder = Builder::new(fun);
  let entry_block = builder.append_basic_block();
  builder.position_at_end(entry_block);
  let v1 = builder.build_const(42);
  let v2 = builder.build_const(1);
  let v3 = builder.build_add(v1, v2);

  let fun = builder.retrieve_function();
  assert!(matches!(fun.get(v1).kind(), inst::InstKind::Const(..)));
  assert!(matches!(fun.get(v2).kind(), inst::InstKind::Const(..)));
  assert!(matches!(fun.get(v3).kind(), inst::InstKind::Add(..)));
}
