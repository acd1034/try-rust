use crate::ir::inst::InstId;
use id_arena::Id;
use std::collections::HashSet;

#[derive(Debug, Clone)]
pub struct Memory {
  store: HashSet<InstId>,
  load: HashSet<InstId>,
}

pub type MemoryId = Id<Memory>;

impl Memory {
  pub fn new() -> Memory {
    Memory {
      store: HashSet::new(),
      load: HashSet::new(),
    }
  }

  // ----- store -----

  pub fn append_store(&mut self, inst_id: InstId) {
    self.store.insert(inst_id);
  }

  pub fn remove_store(&mut self, inst_id: InstId) {
    self.store.remove(&inst_id);
  }

  // ----- load -----

  pub fn append_load(&mut self, inst_id: InstId) {
    self.load.insert(inst_id);
  }

  pub fn remove_load(&mut self, inst_id: InstId) {
    self.load.remove(&inst_id);
  }
}
