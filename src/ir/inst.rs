use crate::ir::block::BlockId;
use crate::ir::function::FunctionId;
use crate::ir::memory::MemoryId;
use id_arena::Id;
use std::collections::HashSet;

#[derive(Debug, Clone)]
pub enum InstKind {
  // Value
  Eq(InstId, InstId),
  Ne(InstId, InstId),
  Lt(InstId, InstId),
  Le(InstId, InstId),
  Add(InstId, InstId),
  Sub(InstId, InstId),
  Mul(InstId, InstId),
  Div(InstId, InstId),
  Load(MemoryId),
  Call(FunctionId, Vec<InstId>),
  Const(u64),
  // Effect
  Br(InstId, BlockId, BlockId),
  Jmp(BlockId),
  Store(MemoryId, InstId),
  Ret(InstId),
}

#[derive(Debug, Clone)]
pub struct Inst {
  kind: InstKind,
  id: InstId,
  use_: HashSet<InstId>,
}

pub type InstId = Id<Inst>;

impl Inst {
  pub fn new(kind: InstKind, id: InstId) -> Inst {
    Inst {
      kind,
      id,
      use_: HashSet::new(),
    }
  }

  pub fn kind(&self) -> &InstKind {
    &self.kind
  }

  pub fn id(&self) -> InstId {
    self.id
  }
}
