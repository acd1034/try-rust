use id_arena::Id;
use std::collections::HashSet;

#[derive(Debug, Clone)]
pub struct Memory {
  store: HashSet<MemoryId>,
  load: HashSet<MemoryId>,
}

pub type MemoryId = Id<Memory>;

impl Memory {
  pub fn new() -> Memory {
    Memory {
      store: HashSet::new(),
      load: HashSet::new(),
    }
  }
}
