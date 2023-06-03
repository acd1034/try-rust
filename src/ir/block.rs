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

  pub fn pred(&self) -> &HashSet<BlockId> {
    &self.pred
  }

  pub fn succ(&self) -> &HashSet<BlockId> {
    &self.succ
  }

  // ----- inst -----

  pub fn inst_position(&self, inst_id: InstId) -> Option<usize> {
    self.insts.iter().position(|&x| x == inst_id)
  }

  pub fn insert_inst(&mut self, index: usize, inst_id: InstId) {
    self.insts.insert(index, inst_id);
  }

  pub fn remove_inst(&mut self, index: usize) {
    self.insts.remove(index);
  }

  // ----- pred -----

  pub fn append_pred(&mut self, block_id: BlockId) {
    self.pred.insert(block_id);
  }

  // ----- succ -----

  pub fn append_succ(&mut self, block_id: BlockId) {
    self.succ.insert(block_id);
  }
}
