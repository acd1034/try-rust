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

#[test]
fn test_ir_visitor() {
  use crate::ir::builder;
  use crate::ir::builder_trait::BuilderTrait;
  use crate::ir::function;
  use parser::ty::Type;

  let fun = function::Function::new("fun".to_string(), Type::Int, Vec::new());
  let mut builder = builder::Builder::new(fun);
  let entry_block = builder.append_basic_block();
  let next_block = builder.append_basic_block();

  builder.position_at_end(entry_block);
  let v1 = builder.build_const(42);
  let v2 = builder.build_const(1);
  let v3 = builder.build_add(v1, v2);
  builder.build_unconditional_branch(next_block);

  builder.position_at_end(next_block);
  let v4 = builder.build_add(v3, v2);
  builder.build_return(v4);

  let fun = builder.retrieve_function();
  let mut vis = Visitor::new(&fun);

  // ----- next_block, next_inst -----

  let mut visited = Vec::new();
  for &block_id in vis.function().blocks() {
    let len = vis.function().get(block_id).insts().len();
    visited.push(vec![false; len]);
  }

  vis.clear_position();
  while let Some(block_id) = vis.next_block() {
    let block = vis.function().block_position(block_id).unwrap();
    while let Some(inst_id) = vis.next_inst() {
      let inst = vis.function().get(block_id).inst_position(inst_id).unwrap();
      visited[block][inst] = true;
    }
  }

  for visited_block in &visited {
    for visited_inst in visited_block {
      assert!(visited_inst);
    }
  }

  // ----- prev_block, prev_inst -----

  let mut visited = Vec::new();
  for &block_id in vis.function().blocks() {
    let len = vis.function().get(block_id).insts().len();
    visited.push(vec![false; len]);
  }

  vis.clear_position();
  while let Some(block_id) = vis.prev_block() {
    let block = vis.function().block_position(block_id).unwrap();
    while let Some(inst_id) = vis.prev_inst() {
      let inst = vis.function().get(block_id).inst_position(inst_id).unwrap();
      visited[block][inst] = true;
    }
  }

  for visited_block in &visited {
    for visited_inst in visited_block {
      assert!(visited_inst);
    }
  }
}
