use crate::common::JoinView;
use crate::parse::Type;
use std::fmt;

pub struct Mod {
  pub name: String,
  pub funs: Vec<Fun>,
}

impl Mod {
  pub fn new(name: String) -> Mod {
    Mod {
      name,
      funs: Vec::new(),
    }
  }

  pub fn get_function(&self, name: &str) -> Option<FunId> {
    self.funs.iter().position(|fun| fun.name == name)
  }

  pub fn add_function(&mut self, fun: Fun) -> FunId {
    let fun_id = self.funs.len();
    self.funs.push(fun);
    fun_id
  }
}

pub struct Fun {
  pub name: String,
  pub ret_ty: Type,
  pub param_tys: Vec<Type>,
  pub bbs: Vec<BBId>,
  pub bb_arena: Vec<BB>,
  pub inst_arena: Vec<Inst>,
  pub reg_arena: Vec<Reg>,
  pub mem_arena: Vec<Mem>,
  // modify
  current_bb: Option<BBId>,
}

pub type FunId = usize;

impl Fun {
  pub fn new(name: String, ret_ty: Type, param_tys: Vec<Type>) -> Fun {
    Fun {
      name,
      ret_ty,
      param_tys,
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
    let bb_id = self.new_bb();
    self.bbs.push(bb_id);
    bb_id
  }

  pub fn insert_basic_block_after(&mut self, bb: BBId) -> Option<BBId> {
    let index = self.find_bb(bb)?;
    let bb_id = self.new_bb();
    self.bbs.insert(index + 1, bb_id);
    Some(bb_id)
  }

  pub fn remove_basic_block(&mut self, bb: BBId) -> bool {
    if let Some(index) = self.find_bb(bb) {
      self.bbs.remove(index);
      true
    } else {
      false
    }
  }

  fn new_bb(&mut self) -> BBId {
    let bb_id = self.bb_arena.len();
    let bb = BB {
      insts: Vec::new(),
      pred: Vec::new(),
      succ: Vec::new(),
    };
    self.bb_arena.push(bb);
    bb_id
  }

  fn find_bb(&self, bb: BBId) -> Option<usize> {
    self.bbs.iter().position(|&x| x == bb)
  }

  // ----- current_bb -----

  pub fn get_insert_block(&self) -> Option<BBId> {
    self.current_bb
  }

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

  pub fn build_conditional_branch(&mut self, v1: Val, bb1: BBId, bb2: BBId) {
    let bb0 = self.current_bb.unwrap();
    self.bb_arena[bb0].succ.push(bb1);
    self.bb_arena[bb0].succ.push(bb2);
    self.bb_arena[bb1].pred.push(bb0);
    self.bb_arena[bb2].pred.push(bb0);
    let inst_id = self.inst_arena.len();
    self.push_inst(Inst::Br(v1, bb1, bb2), inst_id);
  }

  pub fn build_unconditional_branch(&mut self, bb1: BBId) {
    let bb0 = self.current_bb.unwrap();
    self.bb_arena[bb0].succ.push(bb1);
    self.bb_arena[bb1].pred.push(bb0);
    let inst_id = self.inst_arena.len();
    self.push_inst(Inst::Jmp(bb1), inst_id);
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

pub enum Inst {
  Eq(RegId, Val, Val),
  Ne(RegId, Val, Val),
  Lt(RegId, Val, Val),
  Le(RegId, Val, Val),
  Add(RegId, Val, Val),
  Sub(RegId, Val, Val),
  Mul(RegId, Val, Val),
  Div(RegId, Val, Val),
  Br(Val, BBId, BBId),
  Jmp(BBId),
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
    let alloca = JoinView::new(0..self.mem_arena.len(), ",");
    write!(f, "\n  ; alloca={}", alloca)?;
    for &bb in &self.bbs {
      let bb_label = format!("bb{}:", bb);
      let pred = JoinView::new(self.bb_arena[bb].pred.iter(), ",");
      let succ = JoinView::new(self.bb_arena[bb].succ.iter(), ",");
      write!(f, "\n{:<40}; pred={} succ={}", bb_label, pred, succ)?;
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
      Inst::Br(v1, bb1, bb2) => write!(f, "br {}, bb{}, bb{}", v1, bb1, bb2),
      Inst::Jmp(bb1) => write!(f, "jmp bb{}", bb1),
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
