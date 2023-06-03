use crate::ir::block::BlockId;
use crate::ir::function::FunctionId;
use crate::ir::memory::MemoryId;
use id_arena::Id;
use std::collections::{HashMap, HashSet};

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

  pub fn use_(&self) -> &HashSet<InstId> {
    &self.use_
  }

  // ----- use -----

  pub fn append_use(&mut self, inst_id: InstId) {
    self.use_.insert(inst_id);
  }

  pub fn remove_use(&mut self, inst_id: InstId) {
    self.use_.remove(&inst_id);
  }
}

// ----- inst predicates -----

pub fn has_side_effect(inst: &Inst) -> bool {
  use InstKind::*;
  matches!(inst.kind(), Br(..) | Jmp(..) | Store(..) | Ret(..))
}

pub fn is_dead(inst: &Inst, deadness: &mut HashMap<InstId, bool>) -> bool {
  if let Some(&dead) = deadness.get(&inst.id()) {
    dead
  } else if has_side_effect(inst) {
    deadness.insert(inst.id(), false);
    false
  } else {
    let dead = inst
      .use_()
      .iter()
      .all(|user| deadness.get(user).copied().unwrap_or(true));
    deadness.insert(inst.id(), dead);
    dead
  }
}
