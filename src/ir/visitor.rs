use crate::ir::block::*;
use crate::ir::function::*;
use crate::ir::inst::*;
use crate::ir::visitor_trait::*;

pub struct Visitor<'a> {
  function: &'a Function,
  insert_block: Option<BlockId>,
  insert_point: InsertPoint,
}

impl<'a> Visitor<'a> {
  pub fn new(function: &Function) -> Visitor {
    Visitor {
      function,
      insert_block: None,
      insert_point: InsertPoint::Nowhere,
    }
  }
}

impl<'a> VisitorTrait for Visitor<'a> {
  fn function(&self) -> &Function {
    &self.function
  }

  fn get_insert_block(&self) -> Option<BlockId> {
    self.insert_block
  }

  fn get_insert_point(&self) -> InsertPoint {
    self.insert_point
  }

  fn clear_position(&mut self) {
    self.insert_block = None;
    self.insert_point = InsertPoint::Nowhere;
  }

  fn position_before(&mut self, block_id: BlockId) {
    self.insert_block = Some(block_id);
    self.insert_point = InsertPoint::Before;
  }

  fn position_at(&mut self, block_id: BlockId, inst_id: InstId) {
    self.insert_block = Some(block_id);
    self.insert_point = InsertPoint::At(inst_id);
  }

  fn position_at_end(&mut self, block_id: BlockId) {
    self.insert_block = Some(block_id);
    self.insert_point = InsertPoint::After;
  }
}
