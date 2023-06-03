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

  fn remove_inst(&mut self) {
    let (block_id, index) = self.get_insert_index();
    self.function_mut().remove_inst(block_id, index);
    self.position_at_index(block_id, index);
  }

  // ----- inst -> value -----

  fn build_eq(&mut self, v1: InstId, v2: InstId) -> InstId {
    let v0 = self.build_inst_with_id(|id| Inst::new(InstKind::Eq(v1, v2), id));
    self.function_mut().get_mut(v1).append_use(v0);
    self.function_mut().get_mut(v2).append_use(v0);
    v0
  }

  fn build_ne(&mut self, v1: InstId, v2: InstId) -> InstId {
    let v0 = self.build_inst_with_id(|id| Inst::new(InstKind::Ne(v1, v2), id));
    self.function_mut().get_mut(v1).append_use(v0);
    self.function_mut().get_mut(v2).append_use(v0);
    v0
  }

  fn build_lt(&mut self, v1: InstId, v2: InstId) -> InstId {
    let v0 = self.build_inst_with_id(|id| Inst::new(InstKind::Lt(v1, v2), id));
    self.function_mut().get_mut(v1).append_use(v0);
    self.function_mut().get_mut(v2).append_use(v0);
    v0
  }

  fn build_le(&mut self, v1: InstId, v2: InstId) -> InstId {
    let v0 = self.build_inst_with_id(|id| Inst::new(InstKind::Le(v1, v2), id));
    self.function_mut().get_mut(v1).append_use(v0);
    self.function_mut().get_mut(v2).append_use(v0);
    v0
  }

  fn build_add(&mut self, v1: InstId, v2: InstId) -> InstId {
    let v0 = self.build_inst_with_id(|id| Inst::new(InstKind::Add(v1, v2), id));
    self.function_mut().get_mut(v1).append_use(v0);
    self.function_mut().get_mut(v2).append_use(v0);
    v0
  }

  fn build_sub(&mut self, v1: InstId, v2: InstId) -> InstId {
    let v0 = self.build_inst_with_id(|id| Inst::new(InstKind::Sub(v1, v2), id));
    self.function_mut().get_mut(v1).append_use(v0);
    self.function_mut().get_mut(v2).append_use(v0);
    v0
  }

  fn build_mul(&mut self, v1: InstId, v2: InstId) -> InstId {
    let v0 = self.build_inst_with_id(|id| Inst::new(InstKind::Mul(v1, v2), id));
    self.function_mut().get_mut(v1).append_use(v0);
    self.function_mut().get_mut(v2).append_use(v0);
    v0
  }

  fn build_div(&mut self, v1: InstId, v2: InstId) -> InstId {
    let v0 = self.build_inst_with_id(|id| Inst::new(InstKind::Div(v1, v2), id));
    self.function_mut().get_mut(v1).append_use(v0);
    self.function_mut().get_mut(v2).append_use(v0);
    v0
  }

  fn build_load(&mut self, m1: MemoryId) -> InstId {
    let v0 = self.build_inst_with_id(|id| Inst::new(InstKind::Load(m1), id));
    self.function_mut().get_mut(m1).append_load(v0);
    v0
  }

  fn build_call(&mut self, fun_id: FunctionId, args: Vec<InstId>) -> InstId {
    let v0 = self.build_inst_with_id(|id| Inst::new(InstKind::Call(fun_id, args.clone()), id));
    for inst_id in args {
      self.function_mut().get_mut(inst_id).append_use(v0);
    }
    v0
  }

  fn build_const(&mut self, n: u64) -> InstId {
    let v0 = self.build_inst_with_id(|id| Inst::new(InstKind::Const(n), id));
    v0
  }

  // ----- inst -> effect -----

  fn build_conditional_branch(&mut self, v1: InstId, block1: BlockId, block2: BlockId) -> InstId {
    let block0 = self.get_insert_block().unwrap();
    self.function_mut().get_mut(block0).append_succ(block1);
    self.function_mut().get_mut(block0).append_succ(block2);
    self.function_mut().get_mut(block1).append_pred(block0);
    self.function_mut().get_mut(block2).append_pred(block0);

    let v0 = self.build_inst_with_id(|id| Inst::new(InstKind::Br(v1, block1, block2), id));
    self.function_mut().get_mut(v1).append_use(v0);
    v0
  }

  fn build_unconditional_branch(&mut self, block1: BlockId) -> InstId {
    let block0 = self.get_insert_block().unwrap();
    self.function_mut().get_mut(block0).append_succ(block1);
    self.function_mut().get_mut(block1).append_pred(block0);

    let v0 = self.build_inst_with_id(|id| Inst::new(InstKind::Jmp(block1), id));
    v0
  }

  fn build_store(&mut self, m1: MemoryId, v1: InstId) -> InstId {
    let v0 = self.build_inst_with_id(|id| Inst::new(InstKind::Store(m1, v1), id));
    self.function_mut().get_mut(m1).append_store(v0);
    self.function_mut().get_mut(v1).append_use(v0);
    v0
  }

  fn build_return(&mut self, v1: InstId) -> InstId {
    let v0 = self.build_inst_with_id(|id| Inst::new(InstKind::Ret(v1), id));
    self.function_mut().get_mut(v1).append_use(v0);
    v0
  }

  // ----- memory -----

  fn build_alloca(&mut self) -> MemoryId {
    self.function_mut().append_memory()
  }
}
