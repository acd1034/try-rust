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
  fun: Fun,
  scope: Scope,
}

impl GenFun {
  fn new() -> GenFun {
    GenFun {
      fun: Fun::new(),
      scope: Scope::new(),
    }
  }

  // ----- gen_fun -----

  fn gen_fun(mut self, fun: parse::Fun) -> Expected<Fun> {
    match fun {
      parse::Fun::FunDecl(_ret_ty, _name, _param_tys) => todo!(),
      parse::Fun::FunDef(_ret_ty, name, _param_tys, _param_names, body) => {
        // Add name
        self.fun.name = name;

        // Create entry block
        let bb = self.fun.append_basic_block();
        self.fun.position_at_end(bb);

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

        Ok(self.fun)
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
        self.fun.build_ret(v1);
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
    let mem_id = self.fun.build_alloca();
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
        let r0 = self.fun.build_inst(InstArgs::Eq(v1, v2));
        Ok(Val::Reg(r0))
      }
      AST::Ne(n, m) => {
        let v1 = self.gen_expr(*n)?;
        let v2 = self.gen_expr(*m)?;
        let r0 = self.fun.build_inst(InstArgs::Ne(v1, v2));
        Ok(Val::Reg(r0))
      }
      AST::Lt(n, m) => {
        let v1 = self.gen_expr(*n)?;
        let v2 = self.gen_expr(*m)?;
        let r0 = self.fun.build_inst(InstArgs::Lt(v1, v2));
        Ok(Val::Reg(r0))
      }
      AST::Le(n, m) => {
        let v1 = self.gen_expr(*n)?;
        let v2 = self.gen_expr(*m)?;
        let r0 = self.fun.build_inst(InstArgs::Le(v1, v2));
        Ok(Val::Reg(r0))
      }
      AST::Add(n, m) => {
        let v1 = self.gen_expr(*n)?;
        let v2 = self.gen_expr(*m)?;
        let r0 = self.fun.build_inst(InstArgs::Add(v1, v2));
        Ok(Val::Reg(r0))
      }
      AST::Sub(n, m) => {
        let v1 = self.gen_expr(*n)?;
        let v2 = self.gen_expr(*m)?;
        let r0 = self.fun.build_inst(InstArgs::Sub(v1, v2));
        Ok(Val::Reg(r0))
      }
      AST::Mul(n, m) => {
        let v1 = self.gen_expr(*n)?;
        let v2 = self.gen_expr(*m)?;
        let r0 = self.fun.build_inst(InstArgs::Mul(v1, v2));
        Ok(Val::Reg(r0))
      }
      AST::Div(n, m) => {
        let v1 = self.gen_expr(*n)?;
        let v2 = self.gen_expr(*m)?;
        let r0 = self.fun.build_inst(InstArgs::Div(v1, v2));
        Ok(Val::Reg(r0))
      }
      AST::Num(n) => Ok(Val::Imm(n)),
      AST::Assign(..) | AST::Ident(..) => {
        let mem = self.gen_addr(expr)?;
        // TODO: check if mem.get_type().get_element_type().is_array_type()
        if false {
          todo!()
          // Ok(self.gen_array_addr_impl(mem))
        } else {
          let r0 = self.fun.build_inst(InstArgs::Load(mem));
          Ok(Val::Reg(r0))
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
      self.fun.build_store(mem, rhs);
      Ok(mem)
    } else {
      err!("inconsistent types in operands of assignment")
    }
  }
}
