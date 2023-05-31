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

  // ----- value -----

  fn build_inst_with_id<F>(&mut self, f: F) -> InstId
  where
    F: FnOnce(InstId) -> Inst,
  {
    let block_id = self.insert_block();
    self.function_mut().append_inst_with_id(block_id, f)
  }

  fn build_eq(&mut self, v1: InstId, v2: InstId) -> InstId {
    self.build_inst_with_id(|v0| Inst::new(InstKind::Eq(v0, v1, v2)))
  }

  fn build_ne(&mut self, v1: InstId, v2: InstId) -> InstId {
    self.build_inst_with_id(|v0| Inst::new(InstKind::Ne(v0, v1, v2)))
  }

  fn build_lt(&mut self, v1: InstId, v2: InstId) -> InstId {
    self.build_inst_with_id(|v0| Inst::new(InstKind::Lt(v0, v1, v2)))
  }

  fn build_le(&mut self, v1: InstId, v2: InstId) -> InstId {
    self.build_inst_with_id(|v0| Inst::new(InstKind::Le(v0, v1, v2)))
  }

  fn build_add(&mut self, v1: InstId, v2: InstId) -> InstId {
    self.build_inst_with_id(|v0| Inst::new(InstKind::Add(v0, v1, v2)))
  }

  fn build_sub(&mut self, v1: InstId, v2: InstId) -> InstId {
    self.build_inst_with_id(|v0| Inst::new(InstKind::Sub(v0, v1, v2)))
  }

  fn build_mul(&mut self, v1: InstId, v2: InstId) -> InstId {
    self.build_inst_with_id(|v0| Inst::new(InstKind::Mul(v0, v1, v2)))
  }

  fn build_div(&mut self, v1: InstId, v2: InstId) -> InstId {
    self.build_inst_with_id(|v0| Inst::new(InstKind::Div(v0, v1, v2)))
  }

  fn build_call(&mut self, fun_id: FunctionId, args: Vec<InstId>) -> InstId {
    self.build_inst_with_id(|v0| Inst::new(InstKind::Call(v0, fun_id, args)))
  }

  fn build_const(&mut self, n: u64) -> InstId {
    self.build_inst_with_id(|v0| Inst::new(InstKind::Const(v0, n)))
  }

  // ----- effect -----

  fn build_inst(&mut self, inst: Inst) -> InstId {
    let block_id = self.insert_block();
    self.function_mut().append_inst(block_id, inst)
  }

  fn build_ret(&mut self, v1: InstId) -> InstId {
    self.build_inst(Inst::new(InstKind::Ret(v1)))
  }

  // ----- memory -----

  fn build_alloca(&mut self) -> MemoryId {
    self.function_mut().append_memory()
  }
}
