use std::fmt;

pub struct Mod {
  pub name: String,
  pub funs: Vec<Fun>,
}

pub struct Fun {
  pub name: String,
  pub bbs: Vec<BBId>,
  pub bb_arena: Vec<BB>,
  pub inst_arena: Vec<Inst>,
  pub reg_arena: Vec<Reg>,
  pub mem_arena: Vec<Mem>,
  // modify
  current_bb: Option<BBId>,
}

impl Fun {
  pub fn new() -> Fun {
    Fun {
      name: String::new(),
      bbs: Vec::new(),
      bb_arena: Vec::new(),
      inst_arena: Vec::new(),
      reg_arena: Vec::new(),
      mem_arena: Vec::new(),
      current_bb: None,
    }
  }

  // ----- bb -----

  pub fn append_basic_block(&mut self) -> BBId {
    let bb_id = self.bb_arena.len();
    self.bb_arena.push(BB::new());
    self.bbs.push(bb_id);
    bb_id
  }

  // ----- current_bb -----

  pub fn position_at_end(&mut self, bb: BBId) {
    self.current_bb = Some(bb);
  }

  // ----- inst -----

  pub fn build_inst(&mut self, inst_args: InstArgs) -> RegId {
    let inst_id = self.inst_arena.len();
    let r0 = self.new_reg(inst_id);
    let inst = match inst_args {
      InstArgs::Eq(v1, v2) => Inst::Eq(r0.clone(), v1, v2),
      InstArgs::Ne(v1, v2) => Inst::Ne(r0.clone(), v1, v2),
      InstArgs::Lt(v1, v2) => Inst::Lt(r0.clone(), v1, v2),
      InstArgs::Le(v1, v2) => Inst::Le(r0.clone(), v1, v2),
      InstArgs::Add(v1, v2) => Inst::Add(r0.clone(), v1, v2),
      InstArgs::Sub(v1, v2) => Inst::Sub(r0.clone(), v1, v2),
      InstArgs::Mul(v1, v2) => Inst::Mul(r0.clone(), v1, v2),
      InstArgs::Div(v1, v2) => Inst::Div(r0.clone(), v1, v2),
      InstArgs::Load(m1) => Inst::Load(r0.clone(), m1),
    };
    self.push_inst(inst, inst_id);
    r0
  }

  pub fn build_alloca(&mut self) -> MemId {
    self.new_mem()
  }

  pub fn build_store(&mut self, m1: MemId, v2: Val) {
    let inst_id = self.inst_arena.len();
    self.push_inst(Inst::Store(m1, v2), inst_id);
  }

  pub fn build_ret(&mut self, v1: Val) {
    let inst_id = self.inst_arena.len();
    self.push_inst(Inst::Ret(v1), inst_id);
  }

  fn push_inst(&mut self, inst: Inst, inst_id: InstId) {
    self.inst_arena.push(inst);
    self.bb_arena[self.current_bb.unwrap()].insts.push(inst_id);
  }

  // ----- reg, mem -----

  fn new_reg(&mut self, inst_id: InstId) -> RegId {
    let reg_id = self.reg_arena.len();
    let reg = Reg {
      def: inst_id,
      use_: Vec::new(),
    };
    self.reg_arena.push(reg);
    reg_id
  }

  fn new_mem(&mut self) -> MemId {
    let mem_id = self.mem_arena.len();
    let mem = Mem {
      store: Vec::new(),
      load: Vec::new(),
    };
    self.mem_arena.push(mem);
    mem_id
  }
}

pub struct BB {
  pub insts: Vec<InstId>,
  pub pred: Vec<BBId>,
  pub succ: Vec<BBId>,
}

pub type BBId = usize;

impl BB {
  pub fn new() -> BB {
    BB {
      insts: Vec::new(),
      pred: Vec::new(),
      succ: Vec::new(),
    }
  }
}

pub enum Inst {
  Eq(RegId, Val, Val),
  Ne(RegId, Val, Val),
  Lt(RegId, Val, Val),
  Le(RegId, Val, Val),
  Add(RegId, Val, Val),
  Sub(RegId, Val, Val),
  Mul(RegId, Val, Val),
  Div(RegId, Val, Val),
  Store(MemId, Val),
  Load(RegId, MemId),
  Ret(Val),
}

pub type InstId = usize;

pub enum InstArgs {
  Eq(Val, Val),
  Ne(Val, Val),
  Lt(Val, Val),
  Le(Val, Val),
  Add(Val, Val),
  Sub(Val, Val),
  Mul(Val, Val),
  Div(Val, Val),
  Load(MemId),
}

#[derive(Clone)]
pub enum Val {
  Reg(RegId),
  Imm(u64),
}

pub struct Reg {
  pub def: InstId,
  pub use_: Vec<InstId>,
}

pub type RegId = usize;

pub struct Mem {
  pub store: Vec<InstId>,
  pub load: Vec<InstId>,
}

pub type MemId = usize;

// ----- fmt::Display -----

impl fmt::Display for Mod {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "; ModuleName = '{}'", self.name)?;
    for fun in &self.funs {
      write!(f, "\n{}", fun)?;
    }
    Ok(())
  }
}

impl fmt::Display for Fun {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{}()", self.name)?;
    for mem_id in 0..self.mem_arena.len() {
      write!(f, "\n  m{} = alloca", mem_id)?;
    }
    for &bb in &self.bbs {
      write!(f, "\nbb{}:", bb)?;
      for &inst in &self.bb_arena[bb].insts {
        write!(f, "\n  {}", self.inst_arena[inst])?;
      }
    }
    Ok(())
  }
}

impl fmt::Display for Inst {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      Inst::Eq(r0, v1, v2) => write!(f, "r{} = eq {}, {}", r0, v1, v2),
      Inst::Ne(r0, v1, v2) => write!(f, "r{} = ne {}, {}", r0, v1, v2),
      Inst::Lt(r0, v1, v2) => write!(f, "r{} = lt {}, {}", r0, v1, v2),
      Inst::Le(r0, v1, v2) => write!(f, "r{} = le {}, {}", r0, v1, v2),
      Inst::Add(r0, v1, v2) => write!(f, "r{} = add {}, {}", r0, v1, v2),
      Inst::Sub(r0, v1, v2) => write!(f, "r{} = sub {}, {}", r0, v1, v2),
      Inst::Mul(r0, v1, v2) => write!(f, "r{} = mul {}, {}", r0, v1, v2),
      Inst::Div(r0, v1, v2) => write!(f, "r{} = div {}, {}", r0, v1, v2),
      Inst::Store(m1, v2) => write!(f, "store m{}, {}", m1, v2),
      Inst::Load(r0, m1) => write!(f, "r{} = load m{}", r0, m1),
      Inst::Ret(v1) => write!(f, "ret {}", v1),
    }
  }
}

impl fmt::Display for Val {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      Val::Reg(id) => write!(f, "r{}", id),
      Val::Imm(n) => write!(f, "{}", n),
    }
  }
}
