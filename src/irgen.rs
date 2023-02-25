use crate::ir::*;
use crate::parse::{self, Stmt, Type, AST};
use crate::sema;
use crate::{common::Expected, err};
type Scope = sema::Scope<MemId>;

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
