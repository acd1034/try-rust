use crate::ir::block::BlockId;
use crate::ir::function::FunctionId;
use crate::ir::memory::MemoryId;
use id_arena::Id;
use std::collections::HashSet;

#[derive(Debug, Clone)]
pub enum InstKind {
  // Value
  Eq(InstId, InstId, InstId),
  Ne(InstId, InstId, InstId),
  Lt(InstId, InstId, InstId),
  Le(InstId, InstId, InstId),
  Add(InstId, InstId, InstId),
  Sub(InstId, InstId, InstId),
  Mul(InstId, InstId, InstId),
  Div(InstId, InstId, InstId),
  Load(InstId, MemoryId),
  Call(InstId, FunctionId, Vec<InstId>),
  Const(InstId, u64),
  // Effect
  Br(InstId, BlockId, BlockId),
  Jmp(BlockId),
  Store(MemoryId, InstId),
  Ret(InstId),
}

#[derive(Debug, Clone)]
pub struct Inst {
  kind: InstKind,
  use_: HashSet<InstId>,
}

pub type InstId = Id<Inst>;

impl Inst {
  pub fn new(kind: InstKind) -> Inst {
    Inst {
      kind,
      use_: HashSet::new(),
    }
  }

  pub fn kind(&self) -> &InstKind {
    &self.kind
  }
}
