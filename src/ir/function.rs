use crate::ir::block::*;
use crate::ir::inst::*;
use crate::ir::memory::*;
use crate::ty::Type;
use id_arena::{Arena, Id};

#[derive(Debug, Clone)]
pub struct Function {
  name: String,
  ret_ty: Type,
  param_tys: Vec<Type>,
  blocks: Vec<BlockId>,
  block_arena: Arena<Block>,
  inst_arena: Arena<Inst>,
  memory_arena: Arena<Memory>,
}

pub type FunctionId = Id<Function>;

impl Function {
  pub fn new(name: String, ret_ty: Type, param_tys: Vec<Type>) -> Function {
    Function {
      name,
      ret_ty,
      param_tys,
      blocks: Vec::new(),
      block_arena: Arena::new(),
      inst_arena: Arena::new(),
      memory_arena: Arena::new(),
    }
  }

  pub fn is_declaration(&self) -> bool {
    self.blocks.is_empty()
  }

  // ----- accessor -----

  pub fn name(&self) -> &str {
    &self.name
  }

  pub fn ret_ty(&self) -> &Type {
    &self.ret_ty
  }

  pub fn param_tys(&self) -> &[Type] {
    self.param_tys.as_slice()
  }

  pub fn blocks(&self) -> &[BlockId] {
    self.blocks.as_slice()
  }

  pub fn block_arena(&self) -> &Arena<Block> {
    &self.block_arena
  }

  pub fn inst_arena(&self) -> &Arena<Inst> {
    &self.inst_arena
  }

  pub fn memory_arena(&self) -> &Arena<Memory> {
    &self.memory_arena
  }

  pub fn get<Id: AccessFunction>(&self, id: Id) -> &Id::Output {
    id.get(self)
  }

  pub fn get_mut<Id: AccessFunction>(&mut self, id: Id) -> &mut Id::Output {
    id.get_mut(self)
  }

  // ----- block -----

  fn new_block(&mut self) -> BlockId {
    self.block_arena.alloc(Block::new())
  }

  pub fn block_position(&self, block_id: BlockId) -> usize {
    self.blocks.iter().position(|&x| x == block_id).unwrap()
  }

  pub fn append_basic_block(&mut self) -> BlockId {
    let block_id = self.new_block();
    self.blocks.push(block_id);
    block_id
  }

  pub fn insert_basic_block_after(&mut self, block_id: BlockId) -> BlockId {
    let index = self.block_position(block_id);
    let block_id = self.new_block();
    self.blocks.insert(index + 1, block_id);
    block_id
  }

  pub fn remove_basic_block(&mut self, block_id: BlockId) {
    let index = self.block_position(block_id);
    self.blocks.remove(index);
  }

  // ----- inst -----

  pub fn insert_inst(&mut self, block_id: BlockId, index: usize, inst: Inst) -> InstId {
    let inst_id = self.inst_arena.alloc(inst);
    self
      .block_arena
      .get_mut(block_id)
      .unwrap()
      .insert_inst(index, inst_id);
    inst_id
  }

  pub fn insert_inst_with_id<F>(&mut self, block_id: BlockId, index: usize, f: F) -> InstId
  where
    F: FnOnce(InstId) -> Inst,
  {
    let inst_id = self.inst_arena.alloc_with_id(f);
    self
      .block_arena
      .get_mut(block_id)
      .unwrap()
      .insert_inst(index, inst_id);
    inst_id
  }

  pub fn remove_inst(&mut self, block_id: BlockId, index: usize) {
    self
      .block_arena
      .get_mut(block_id)
      .unwrap()
      .remove_inst(index);
  }

  // ----- memory -----

  pub fn append_memory(&mut self) -> MemoryId {
    self.memory_arena.alloc(Memory::new())
  }
}

pub trait AccessFunction {
  type Output;
  fn get<'a>(&self, fun: &'a Function) -> &'a Self::Output;
  fn get_mut<'a>(&self, fun: &'a mut Function) -> &'a mut Self::Output;
}

impl AccessFunction for BlockId {
  type Output = Block;
  fn get<'a>(&self, fun: &'a Function) -> &'a Self::Output {
    fun.block_arena.get(*self).unwrap()
  }
  fn get_mut<'a>(&self, fun: &'a mut Function) -> &'a mut Self::Output {
    fun.block_arena.get_mut(*self).unwrap()
  }
}

impl AccessFunction for InstId {
  type Output = Inst;
  fn get<'a>(&self, fun: &'a Function) -> &'a Self::Output {
    fun.inst_arena.get(*self).unwrap()
  }
  fn get_mut<'a>(&self, fun: &'a mut Function) -> &'a mut Self::Output {
    fun.inst_arena.get_mut(*self).unwrap()
  }
}

impl AccessFunction for MemoryId {
  type Output = Memory;
  fn get<'a>(&self, fun: &'a Function) -> &'a Self::Output {
    fun.memory_arena.get(*self).unwrap()
  }
  fn get_mut<'a>(&self, fun: &'a mut Function) -> &'a mut Self::Output {
    fun.memory_arena.get_mut(*self).unwrap()
  }
}
