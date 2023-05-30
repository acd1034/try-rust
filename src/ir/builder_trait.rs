use crate::ir::block::*;
use crate::ir::function::*;
use crate::ir::inst::*;
use crate::ir::memory::*;

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

  fn build_eq(&mut self, v1: InstId, v2: InstId) -> InstId {
    let block_id = self.insert_block();
    self
      .function_mut()
      .append_inst(block_id, InstKind::Eq(v1, v2))
  }

  fn build_ne(&mut self, v1: InstId, v2: InstId) -> InstId {
    let block_id = self.insert_block();
    self
      .function_mut()
      .append_inst(block_id, InstKind::Ne(v1, v2))
  }

  fn build_lt(&mut self, v1: InstId, v2: InstId) -> InstId {
    let block_id = self.insert_block();
    self
      .function_mut()
      .append_inst(block_id, InstKind::Lt(v1, v2))
  }

  fn build_le(&mut self, v1: InstId, v2: InstId) -> InstId {
    let block_id = self.insert_block();
    self
      .function_mut()
      .append_inst(block_id, InstKind::Le(v1, v2))
  }

  fn build_add(&mut self, v1: InstId, v2: InstId) -> InstId {
    let block_id = self.insert_block();
    self
      .function_mut()
      .append_inst(block_id, InstKind::Add(v1, v2))
  }

  fn build_sub(&mut self, v1: InstId, v2: InstId) -> InstId {
    let block_id = self.insert_block();
    self
      .function_mut()
      .append_inst(block_id, InstKind::Sub(v1, v2))
  }

  fn build_mul(&mut self, v1: InstId, v2: InstId) -> InstId {
    let block_id = self.insert_block();
    self
      .function_mut()
      .append_inst(block_id, InstKind::Mul(v1, v2))
  }

  fn build_div(&mut self, v1: InstId, v2: InstId) -> InstId {
    let block_id = self.insert_block();
    self
      .function_mut()
      .append_inst(block_id, InstKind::Div(v1, v2))
  }

  fn build_call(&mut self, fun_id: FunctionId, args: Vec<InstId>) -> InstId {
    let block_id = self.insert_block();
    self
      .function_mut()
      .append_inst(block_id, InstKind::Call(fun_id, args))
  }

  fn build_const(&mut self, n: u64) -> InstId {
    let block_id = self.insert_block();
    self
      .function_mut()
      .append_inst(block_id, InstKind::Const(n))
  }

  fn build_alloca(&mut self) -> MemoryId {
    self.function_mut().append_memory()
  }

  fn build_ret(&mut self, v1: InstId) -> InstId {
    let block_id = self.insert_block();
    self.function_mut().append_inst(block_id, InstKind::Ret(v1))
  }
}
