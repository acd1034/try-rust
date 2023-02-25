use crate::parse::{self, Stmt, Type, AST};
use crate::sema;
use crate::{common::Expected, err};
use std::fmt;
type Scope = sema::Scope<MemId>;

pub struct Mod {
  pub name: String,
  pub funs: Vec<Fun>,
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
  pub name: String,
  pub bbs: Vec<BBId>,
  pub bb_arena: Vec<BB>,
  pub inst_arena: Vec<Inst>,
  pub reg_arena: Vec<Reg>,
  pub mem_arena: Vec<Mem>,
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
  pub insts: Vec<InstId>,
  pub pred: Vec<BBId>,
  pub succ: Vec<BBId>,
}

pub type BBId = usize;

impl BB {
  fn new() -> BB {
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
  Alloca(MemId),
  Store(MemId, Val),
  Load(Val, MemId),
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
      Inst::Alloca(m0) => write!(f, "m{} = alloca", m0),
      Inst::Store(m1, v2) => write!(f, "store m{}, {}", m1, v2),
      Inst::Load(v1, m2) => write!(f, "load {}, m{}", v1, m2),
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
  pub def: InstId,
  pub use_: Vec<InstId>,
}

pub type RegId = usize;

pub struct Mem {
  pub def: InstId,
  pub use_: Vec<InstId>,
}

pub type MemId = usize;

// ----- irgen -----

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
  current_bb: Option<BBId>,
  // arena
  bb_arena: Vec<BB>,
  inst_arena: Vec<Inst>,
  reg_arena: Vec<Reg>,
  mem_arena: Vec<Mem>,
  // sema
  scope: Scope,
}

impl GenFun {
  fn new() -> GenFun {
    GenFun {
      bbs: Vec::new(),
      current_bb: None,
      bb_arena: Vec::new(),
      inst_arena: Vec::new(),
      reg_arena: Vec::new(),
      mem_arena: Vec::new(),
      scope: Scope::new(),
    }
  }

  fn new_bb(&mut self) -> BBId {
    let bb_id = self.bb_arena.len();
    self.bb_arena.push(BB::new());
    self.bbs.push(bb_id);
    bb_id
  }

  fn position_at_end(&mut self, bb: BBId) {
    self.current_bb = Some(bb);
  }

  fn push_inst(&mut self, inst: Inst) {
    let inst_id = self.inst_arena.len();
    self.inst_arena.push(inst);
    self.bb_arena[self.current_bb.unwrap()].insts.push(inst_id);
  }

  fn new_reg(&mut self) -> Val {
    let reg_id = self.reg_arena.len();
    let inst_id = self.inst_arena.len();
    let reg = Reg {
      def: inst_id,
      use_: Vec::new(),
    };
    self.reg_arena.push(reg);
    Val::Reg(reg_id)
  }

  // ----- gen_fun -----

  fn gen_fun(mut self, fun: parse::Fun) -> Expected<Fun> {
    match fun {
      parse::Fun::FunDecl(_ret_ty, _name, _param_tys) => todo!(),
      parse::Fun::FunDef(_ret_ty, name, _param_tys, _param_names, body) => {
        // Create entry block
        let bb = self.new_bb();
        self.position_at_end(bb);

        // Push first scope
        self.scope.push();

        // Generate function body
        let mut has_terminator = false;
        for stmt in body {
          has_terminator = self.gen_stmt(stmt)?;
          if has_terminator {
            break;
          }
        }

        // Pop first scope
        self.scope.pop();

        // Check terminator
        if !has_terminator {
          return err!("no terminator in function");
        }

        Ok(Fun {
          name,
          bbs: self.bbs,
          bb_arena: self.bb_arena,
          inst_arena: self.inst_arena,
          reg_arena: self.reg_arena,
          mem_arena: self.mem_arena,
        })
      }
    }
  }

  // ----- gen_stmt -----

  fn gen_stmt(&mut self, stmt: Stmt) -> Expected<bool> {
    match stmt {
      Stmt::VarDef(ty, name, init) => {
        if self.scope.get(&name).is_some() {
          return err!("variable already exists");
        }

        let mem = self.create_entry_block_alloca(ty, name);
        if let Some(expr) = init {
          let rhs = self.gen_expr(expr)?;
          self.gen_assign_impl(mem, rhs)?;
        }
        Ok(false)
      }
      Stmt::Return(expr) => {
        let v1 = self.gen_expr(expr)?;
        self.push_inst(Inst::Ret(v1));
        Ok(true)
      }
      Stmt::Block(stmts) => {
        self.scope.push();
        let mut has_terminator = false;
        for stmt in stmts {
          has_terminator = self.gen_stmt(stmt)?;
          if has_terminator {
            break;
          }
        }
        self.scope.pop();
        Ok(has_terminator)
      }
      Stmt::Expr(expr) => {
        self.gen_expr(expr)?;
        Ok(false)
      }
      _ => todo!(),
    }
  }

  fn create_entry_block_alloca(&mut self, ty: Type, name: String) -> MemId {
    // Push mem_arena
    let mem_id = self.mem_arena.len();
    let inst_id = self.inst_arena.len();
    let mem = Mem {
      def: inst_id,
      use_: Vec::new(),
    };
    self.mem_arena.push(mem);

    // Push Alloca
    let &entry_block = self.bbs.first().unwrap();
    let alloca = Inst::Alloca(mem_id);
    self.inst_arena.push(alloca);
    self.bb_arena[entry_block].insts.insert(0, inst_id);

    // Insert scope
    self.scope.insert(name, mem_id);

    mem_id
  }

  // ----- gen_expr -----

  fn gen_expr(&mut self, expr: AST) -> Expected<Val> {
    match expr {
      AST::Eq(n, m) => {
        let v1 = self.gen_expr(*n)?;
        let v2 = self.gen_expr(*m)?;
        let v0 = self.new_reg();
        self.push_inst(Inst::Eq(v0.clone(), v1, v2));
        Ok(v0)
      }
      AST::Ne(n, m) => {
        let v1 = self.gen_expr(*n)?;
        let v2 = self.gen_expr(*m)?;
        let v0 = self.new_reg();
        self.push_inst(Inst::Ne(v0.clone(), v1, v2));
        Ok(v0)
      }
      AST::Lt(n, m) => {
        let v1 = self.gen_expr(*n)?;
        let v2 = self.gen_expr(*m)?;
        let v0 = self.new_reg();
        self.push_inst(Inst::Lt(v0.clone(), v1, v2));
        Ok(v0)
      }
      AST::Le(n, m) => {
        let v1 = self.gen_expr(*n)?;
        let v2 = self.gen_expr(*m)?;
        let v0 = self.new_reg();
        self.push_inst(Inst::Le(v0.clone(), v1, v2));
        Ok(v0)
      }
      AST::Add(n, m) => {
        let v1 = self.gen_expr(*n)?;
        let v2 = self.gen_expr(*m)?;
        let v0 = self.new_reg();
        self.push_inst(Inst::Add(v0.clone(), v1, v2));
        Ok(v0)
      }
      AST::Sub(n, m) => {
        let v1 = self.gen_expr(*n)?;
        let v2 = self.gen_expr(*m)?;
        let v0 = self.new_reg();
        self.push_inst(Inst::Sub(v0.clone(), v1, v2));
        Ok(v0)
      }
      AST::Mul(n, m) => {
        let v1 = self.gen_expr(*n)?;
        let v2 = self.gen_expr(*m)?;
        let v0 = self.new_reg();
        self.push_inst(Inst::Mul(v0.clone(), v1, v2));
        Ok(v0)
      }
      AST::Div(n, m) => {
        let v1 = self.gen_expr(*n)?;
        let v2 = self.gen_expr(*m)?;
        let v0 = self.new_reg();
        self.push_inst(Inst::Div(v0.clone(), v1, v2));
        Ok(v0)
      }
      AST::Num(n) => Ok(Val::Imm(n)),
      AST::Assign(..) | AST::Ident(..) => {
        let mem = self.gen_addr(expr)?;
        // TODO: check if mem.get_type().get_element_type().is_array_type()
        if false {
          todo!()
          // Ok(self.gen_array_addr_impl(mem))
        } else {
          let v0 = self.new_reg();
          self.push_inst(Inst::Load(v0.clone(), mem));
          Ok(v0)
        }
      }
      _ => todo!(),
    }
  }

  // ----- gen_addr -----

  fn gen_addr(&mut self, expr: AST) -> Expected<MemId> {
    match expr {
      AST::Assign(n, m) => {
        let rhs = self.gen_expr(*m)?;
        let mem = self.gen_addr(*n)?;
        self.gen_assign_impl(mem, rhs)
      }
      // AST::Deref(n) => {
      //   let ptr = self.gen_expr(*n)?;
      //   if ptr.is_pointer_value() {
      //     Ok(ptr.into_pointer_value())
      //   } else {
      //     err!("cannot dereference int value")
      //   }
      // }
      AST::Ident(name) => match self.scope.get_all(&name) {
        Some(&mem) => Ok(mem),
        None => err!("variable should be declared before its first use"),
      },
      _ => err!("cannot obtain address of rvalue"),
    }
  }

  fn gen_assign_impl(&mut self, mem: MemId, rhs: Val) -> Expected<MemId> {
    // TODO: check if lhs.get_type().get_element_type() == rhs.get_type().as_any_type_enum()
    if true {
      self.push_inst(Inst::Store(mem, rhs));
      Ok(mem)
    } else {
      err!("inconsistent types in operands of assignment")
    }
  }
}
