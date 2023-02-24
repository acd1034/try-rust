use crate::parse::{self, Stmt, Type, AST};
use crate::{common::Expected, err};
use std::fmt;

pub struct Mod {
  name: String,
  funs: Vec<Fun>,
}

impl fmt::Display for Mod {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "; ModuleName = '{}'", self.name)?;
    for fun in &self.funs {
      write!(f, "\n{}", fun)?;
    }
    Ok(())
  }
}

pub struct Fun {
  name: String,
  bbs: Vec<BBId>,
  bb_arena: Vec<BB>,
  inst_arena: Vec<Inst>,
  reg_arena: Vec<Reg>,
}

impl fmt::Display for Fun {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{}()", self.name)?;
    for &bb in &self.bbs {
      write!(f, "\nbb{}:", bb)?;
      for &inst in &self.bb_arena[bb].insts {
        write!(f, "\n  {}", self.inst_arena[inst])?;
      }
    }
    Ok(())
  }
}

pub struct BB {
  insts: Vec<InstId>,
  pred: Vec<BBId>,
  succ: Vec<BBId>,
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
  Eq(Val, Val, Val),
  Ne(Val, Val, Val),
  Lt(Val, Val, Val),
  Le(Val, Val, Val),
  Add(Val, Val, Val),
  Sub(Val, Val, Val),
  Mul(Val, Val, Val),
  Div(Val, Val, Val),
  Ret(Val),
}

pub type InstId = usize;

impl fmt::Display for Inst {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      Inst::Eq(v0, v1, v2) => write!(f, "{} = eq {}, {}", v0, v1, v2),
      Inst::Ne(v0, v1, v2) => write!(f, "{} = ne {}, {}", v0, v1, v2),
      Inst::Lt(v0, v1, v2) => write!(f, "{} = lt {}, {}", v0, v1, v2),
      Inst::Le(v0, v1, v2) => write!(f, "{} = le {}, {}", v0, v1, v2),
      Inst::Add(v0, v1, v2) => write!(f, "{} = add {}, {}", v0, v1, v2),
      Inst::Sub(v0, v1, v2) => write!(f, "{} = sub {}, {}", v0, v1, v2),
      Inst::Mul(v0, v1, v2) => write!(f, "{} = mul {}, {}", v0, v1, v2),
      Inst::Div(v0, v1, v2) => write!(f, "{} = div {}, {}", v0, v1, v2),
      Inst::Ret(v1) => write!(f, "ret {}", v1),
    }
  }
}

#[derive(Clone)]
pub enum Val {
  Reg(RegId),
  Imm(u64),
}

impl fmt::Display for Val {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      Val::Reg(id) => write!(f, "r{}", id),
      Val::Imm(n) => write!(f, "{}", n),
    }
  }
}

pub struct Reg {
  def: InstId,
  use_: Vec<InstId>,
}

pub type RegId = usize;

pub fn irgen(funs: Vec<parse::Fun>) -> Expected<Mod> {
  let name = "mod".to_string();
  let funs = funs
    .into_iter()
    .map(|fun| GenFun::new().gen_fun(fun))
    .collect::<Result<Vec<_>, _>>()?;
  Ok(Mod { name, funs })
}

struct GenFun {
  bbs: Vec<BBId>,
  bb_arena: Vec<BB>,
  inst_arena: Vec<Inst>,
  reg_arena: Vec<Reg>,
}

impl GenFun {
  pub fn new() -> GenFun {
    GenFun {
      bbs: Vec::new(),
      bb_arena: Vec::new(),
      inst_arena: Vec::new(),
      reg_arena: Vec::new(),
    }
  }

  pub fn new_bb(&mut self) -> BBId {
    let bb_id = self.bb_arena.len();
    self.bb_arena.push(BB::new());
    self.bbs.push(bb_id);
    bb_id
  }

  pub fn gen_fun(mut self, fun: parse::Fun) -> Expected<Fun> {
    match fun {
      parse::Fun::FunDecl(ret_ty, name, param_tys) => todo!(),
      parse::Fun::FunDef(ret_ty, name, param_tys, param_names, body) => {
        let bb = self.new_bb();

        // Generate function body
        for stmt in body {
          self.gen_stmt(stmt, bb)?;
        }

        Ok(Fun {
          name,
          bbs: self.bbs,
          bb_arena: self.bb_arena,
          inst_arena: self.inst_arena,
          reg_arena: self.reg_arena,
        })
      }
    }
  }

  pub fn push_inst(&mut self, inst: Inst, bb: BBId) {
    let inst_id = self.inst_arena.len();
    self.inst_arena.push(inst);
    self.bb_arena[bb].insts.push(inst_id);
  }

  pub fn gen_stmt(&mut self, stmt: Stmt, bb: BBId) -> Expected<BBId> {
    match stmt {
      Stmt::Return(expr) => {
        let v1 = self.gen_expr(expr, bb)?;
        self.push_inst(Inst::Ret(v1), bb);
        Ok(bb)
      }
      _ => todo!(),
    }
  }

  pub fn new_reg(&mut self) -> Val {
    let reg_id = self.reg_arena.len();
    let inst_id = self.inst_arena.len();
    let reg = Reg {
      def: inst_id,
      use_: Vec::new(),
    };
    self.reg_arena.push(reg);
    Val::Reg(reg_id)
  }

  pub fn gen_expr(&mut self, expr: AST, bb: BBId) -> Expected<Val> {
    match expr {
      AST::Eq(n, m) => {
        let v1 = self.gen_expr(*n, bb)?;
        let v2 = self.gen_expr(*m, bb)?;
        let v0 = self.new_reg();
        self.push_inst(Inst::Eq(v0.clone(), v1, v2), bb);
        Ok(v0)
      }
      AST::Ne(n, m) => {
        let v1 = self.gen_expr(*n, bb)?;
        let v2 = self.gen_expr(*m, bb)?;
        let v0 = self.new_reg();
        self.push_inst(Inst::Ne(v0.clone(), v1, v2), bb);
        Ok(v0)
      }
      AST::Lt(n, m) => {
        let v1 = self.gen_expr(*n, bb)?;
        let v2 = self.gen_expr(*m, bb)?;
        let v0 = self.new_reg();
        self.push_inst(Inst::Lt(v0.clone(), v1, v2), bb);
        Ok(v0)
      }
      AST::Le(n, m) => {
        let v1 = self.gen_expr(*n, bb)?;
        let v2 = self.gen_expr(*m, bb)?;
        let v0 = self.new_reg();
        self.push_inst(Inst::Le(v0.clone(), v1, v2), bb);
        Ok(v0)
      }
      AST::Add(n, m) => {
        let v1 = self.gen_expr(*n, bb)?;
        let v2 = self.gen_expr(*m, bb)?;
        let v0 = self.new_reg();
        self.push_inst(Inst::Add(v0.clone(), v1, v2), bb);
        Ok(v0)
      }
      AST::Sub(n, m) => {
        let v1 = self.gen_expr(*n, bb)?;
        let v2 = self.gen_expr(*m, bb)?;
        let v0 = self.new_reg();
        self.push_inst(Inst::Sub(v0.clone(), v1, v2), bb);
        Ok(v0)
      }
      AST::Mul(n, m) => {
        let v1 = self.gen_expr(*n, bb)?;
        let v2 = self.gen_expr(*m, bb)?;
        let v0 = self.new_reg();
        self.push_inst(Inst::Mul(v0.clone(), v1, v2), bb);
        Ok(v0)
      }
      AST::Div(n, m) => {
        let v1 = self.gen_expr(*n, bb)?;
        let v2 = self.gen_expr(*m, bb)?;
        let v0 = self.new_reg();
        self.push_inst(Inst::Div(v0.clone(), v1, v2), bb);
        Ok(v0)
      }
      AST::Num(n) => Ok(Val::Imm(n)),
      _ => todo!(),
    }
  }
}
