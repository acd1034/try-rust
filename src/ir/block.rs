use crate::ir::inst::*;
use id_arena::Id;
use std::collections::HashSet;

#[derive(Debug, Clone)]
pub struct Block {
  insts: Vec<InstId>,
  pred: HashSet<BlockId>,
  succ: HashSet<BlockId>,
}

pub type BlockId = Id<Block>;

impl Block {
  pub fn new() -> Block {
    Block {
      insts: Vec::new(),
      pred: HashSet::new(),
      succ: HashSet::new(),
    }
  }

  pub fn insts(&self) -> &[InstId] {
    self.insts.as_slice()
  }

  // ----- inst -----

  pub fn append_inst(&mut self, inst_id: InstId) {
    self.insts.push(inst_id);
  }
}
