use crate::ir::block::*;
use crate::ir::function::*;
use crate::ir::inst::*;

pub trait BuilderTrait {
  fn function(&self) -> &Function;
  fn function_mut(&mut self) -> &mut Function;
  fn insert_block(&self) -> BlockId;
  fn position_at_end(&mut self, block_id: BlockId);

  // ----- block -----

  fn append_basic_block(&mut self) -> BlockId {
    self.function_mut().append_basic_block()
  }

  fn insert_basic_block_after(&mut self, block_id: BlockId) -> BlockId {
    self.function_mut().insert_basic_block_after(block_id)
  }

  // ----- build -----

  fn build_add(&mut self, v1: InstId, v2: InstId) -> InstId {
    let block_id = self.insert_block();
    self
      .function_mut()
      .append_inst(block_id, InstKind::Add(v1, v2))
  }

  fn build_const(&mut self, n: u64) -> InstId {
    let block_id = self.insert_block();
    self
      .function_mut()
      .append_inst(block_id, InstKind::Const(n))
  }
}
