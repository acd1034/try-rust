use crate::ir::block::*;
use crate::ir::function::*;
use crate::ir::inst::*;

#[derive(Clone, Copy)]
pub enum InsertPoint {
  Before,
  At(InstId),
  After,
  Nowhere,
}

pub trait VisitorTrait {
  fn function(&self) -> &Function;
  fn get_insert_block(&self) -> Option<BlockId>;
  fn get_insert_point(&self) -> InsertPoint;
  fn clear_position(&mut self);
  fn position_before(&mut self, block_id: BlockId);
  fn position_at(&mut self, block_id: BlockId, inst_id: InstId);
  fn position_at_end(&mut self, block_id: BlockId);

  fn get_insert_index(&self) -> (BlockId, usize) {
    let block_id = self.get_insert_block().unwrap();
    let index = match self.get_insert_point() {
      InsertPoint::At(inst_id) => self
        .function()
        .get(block_id)
        .inst_position(inst_id)
        .unwrap(),
      InsertPoint::After => self.function().get(block_id).insts().len(),
      _ => panic!("invalid insert point"),
    };
    (block_id, index)
  }

  fn position_at_index(&mut self, block_id: BlockId, index: usize) {
    let inst_id = self.function().get(block_id).insts()[index];
    self.position_at(block_id, inst_id);
  }

  fn next_block(&mut self) -> Option<BlockId> {
    let next_block = if let Some(block_id) = self.get_insert_block() {
      let pos = self.function().block_position(block_id).unwrap();
      self.function().blocks().get(pos + 1).copied()
    } else {
      self.function().blocks().first().copied()
    };

    if let Some(block_id) = next_block {
      self.position_before(block_id);
    } else {
      self.clear_position();
    }

    next_block
  }

  fn prev_block(&mut self) -> Option<BlockId> {
    let prev_block = if let Some(block_id) = self.get_insert_block() {
      let pos = self.function().block_position(block_id).unwrap();
      if pos == 0 {
        None
      } else {
        Some(self.function().blocks()[pos - 1])
      }
    } else {
      self.function().blocks().last().copied()
    };

    if let Some(block_id) = prev_block {
      self.position_at_end(block_id);
    } else {
      self.clear_position();
    }

    prev_block
  }

  fn next_inst(&mut self) -> Option<InstId> {
    let block_id = self.get_insert_block()?;
    let point = self.get_insert_point();
    match point {
      InsertPoint::Before => {
        if self.function().get(block_id).insts().is_empty() {
          self.position_at_end(block_id);
          None
        } else {
          self.position_at_index(block_id, 0);
          Some(self.function().get(block_id).insts()[0])
        }
      }
      InsertPoint::At(inst_id) => {
        let index = self
          .function()
          .get(block_id)
          .inst_position(inst_id)
          .unwrap();
        if index + 1 == self.function().get(block_id).insts().len() {
          self.position_at_end(block_id);
          None
        } else {
          self.position_at_index(block_id, index + 1);
          Some(self.function().get(block_id).insts()[index + 1])
        }
      }
      _ => None,
    }
  }

  fn prev_inst(&mut self) -> Option<InstId> {
    let block_id = self.get_insert_block()?;
    let point = self.get_insert_point();
    match point {
      InsertPoint::After => {
        if self.function().get(block_id).insts().is_empty() {
          self.position_before(block_id);
          None
        } else {
          let index = self.function().get(block_id).insts().len() - 1;
          self.position_at_index(block_id, index);
          Some(self.function().get(block_id).insts()[index])
        }
      }
      InsertPoint::At(inst_id) => {
        let index = self
          .function()
          .get(block_id)
          .inst_position(inst_id)
          .unwrap();
        if index == 0 {
          self.position_before(block_id);
          None
        } else {
          self.position_at_index(block_id, index - 1);
          Some(self.function().get(block_id).insts()[index - 1])
        }
      }
      _ => None,
    }
  }
}
