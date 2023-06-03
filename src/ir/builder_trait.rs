use crate::ir::block::*;
use crate::ir::function::*;
use crate::ir::inst::*;
use crate::ir::memory::*;
use crate::ir::visitor_trait::*;

pub trait BuilderTrait: VisitorTrait {
  fn function_mut(&mut self) -> &mut Function;

  // ----- block -----

  fn append_basic_block(&mut self) -> BlockId {
    self.function_mut().append_basic_block()
  }

  fn insert_basic_block_after(&mut self, block_id: BlockId) -> BlockId {
    self.function_mut().insert_basic_block_after(block_id)
  }

  fn remove_basic_block(&mut self, block_id: BlockId) {
    self.function_mut().remove_basic_block(block_id);
  }

  // ----- inst -----

  fn build_inst_with_id<F>(&mut self, f: F) -> InstId
  where
    F: FnOnce(InstId) -> Inst,
  {
    let (block_id, index) = self.get_insert_index();
    self.function_mut().insert_inst_with_id(block_id, index, f)
  }

  fn build_inst(&mut self, inst: Inst) -> InstId {
    let (block_id, index) = self.get_insert_index();
    self.function_mut().insert_inst(block_id, index, inst)
  }

  fn remove_inst(&mut self) {
    let (block_id, index) = self.get_insert_index();
    self.function_mut().remove_inst(block_id, index)
  }

  // ----- inst -> value -----

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

  fn build_load(&mut self, m1: MemoryId) -> InstId {
    self.build_inst_with_id(|v0| Inst::new(InstKind::Load(v0, m1)))
  }

  fn build_call(&mut self, fun_id: FunctionId, args: Vec<InstId>) -> InstId {
    self.build_inst_with_id(|v0| Inst::new(InstKind::Call(v0, fun_id, args)))
  }

  fn build_const(&mut self, n: u64) -> InstId {
    self.build_inst_with_id(|v0| Inst::new(InstKind::Const(v0, n)))
  }

  // ----- inst -> effect -----

  fn build_conditional_branch(&mut self, v1: InstId, block1: BlockId, block2: BlockId) -> InstId {
    let block0 = self.get_insert_block().unwrap();
    self.function_mut().get_mut(block0).append_succ(block1);
    self.function_mut().get_mut(block0).append_succ(block2);
    self.function_mut().get_mut(block1).append_pred(block0);
    self.function_mut().get_mut(block2).append_pred(block0);
    self.build_inst(Inst::new(InstKind::Br(v1, block1, block2)))
  }

  fn build_unconditional_branch(&mut self, block1: BlockId) -> InstId {
    let block0 = self.get_insert_block().unwrap();
    self.function_mut().get_mut(block0).append_succ(block1);
    self.function_mut().get_mut(block1).append_pred(block0);
    self.build_inst(Inst::new(InstKind::Jmp(block1)))
  }

  fn build_store(&mut self, m1: MemoryId, v1: InstId) -> InstId {
    self.build_inst(Inst::new(InstKind::Store(m1, v1)))
  }

  fn build_return(&mut self, v1: InstId) -> InstId {
    self.build_inst(Inst::new(InstKind::Ret(v1)))
  }

  // ----- memory -----

  fn build_alloca(&mut self) -> MemoryId {
    self.function_mut().append_memory()
  }
}
