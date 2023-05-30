use crate::ir::block::BlockId;
use crate::ir::function::FunctionId;
use crate::ir::memory::MemoryId;
use id_arena::Id;
use std::collections::HashSet;

pub enum InstKind {
  Eq(InstId, InstId),
  Ne(InstId, InstId),
  Lt(InstId, InstId),
  Le(InstId, InstId),
  Add(InstId, InstId),
  Sub(InstId, InstId),
  Mul(InstId, InstId),
  Div(InstId, InstId),
  Br(InstId, BlockId, BlockId),
  Jmp(BlockId),
  Store(MemoryId, InstId),
  Load(MemoryId),
  Call(FunctionId, Vec<InstId>),
  Const(u64),
  Ret(InstId),
}

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
