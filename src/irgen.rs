use crate::ir::*;
use crate::parse::{self, Stmt, Type, AST};
use crate::sema;
use crate::{common::Expected, err};
type Scope = sema::Scope<MemId>;

// ----- irgen -----

pub struct IRGen {
  module: Mod,
}

impl IRGen {
  pub fn new(name: String) -> IRGen {
    IRGen {
      module: Mod::new(name),
    }
  }

  pub fn irgen(mut self, funs: Vec<parse::Fun>) -> Expected<Mod> {
    for fun in funs {
      GenFun::new(&mut self.module).gen_fun(fun)?;
    }
    Ok(self.module)
  }
}

struct GenFun<'a> {
  module: &'a mut Mod,
  insert_fun: FunId,
  scope: Scope,
  break_label: Vec<BBId>,
  cont_label: Vec<BBId>,
}

macro_rules! insert_fun {
  ($x:expr) => {
    $x.module.funs[$x.insert_fun]
  };
}

impl<'a> GenFun<'a> {
  fn new(module: &'a mut Mod) -> GenFun<'a> {
    GenFun {
      module,
      insert_fun: 0,
      scope: Scope::new(),
      break_label: Vec::new(),
      cont_label: Vec::new(),
    }
  }

  fn set_insert_function(&mut self, fun: FunId) {
    self.insert_fun = fun;
  }

  // ----- gen_fun -----

  fn gen_fun(mut self, fun: parse::Fun) -> Expected<FunId> {
    match fun {
      parse::Fun::FunDecl(ret_ty, name, param_tys) => self.gen_fun_decl(ret_ty, name, param_tys),
      parse::Fun::FunDef(ret_ty, name, param_tys, param_names, body) => {
        // Check consistency with forward declaration
        let fn_id = self.gen_fun_decl(ret_ty, name, param_tys.clone())?;
        self.set_insert_function(fn_id);

        // Check function is not defined
        if insert_fun!(self).bbs.len() != 0 {
          return err!("function already exists");
        }

        // Create entry block
        let bb = insert_fun!(self).append_basic_block();
        insert_fun!(self).position_at_end(bb);

        // Push first scope
        self.scope.push();

        // Add function parameters
        for (ty, name) in std::iter::zip(param_tys, param_names) {
          if self.scope.get(&name).is_none() {
            self.create_entry_block_alloca(ty, name);
          } else {
            return err!("function parameter already exists");
          }
        }

        // Generate function body
        let mut has_terminator = false;
        for stmt in body {
          has_terminator = self.gen_stmt(stmt)?;
          if has_terminator {
            break;
          }
        }

        // Check terminator
        if !has_terminator {
          return err!("no terminator in function");
        }

        // Pop first scope
        self.scope.pop();

        Ok(self.insert_fun)
      }
    }
  }

  fn gen_fun_decl(&mut self, ret_ty: Type, name: String, param_tys: Vec<Type>) -> Expected<FunId> {
    if let Some(fn_id) = self.module.get_function(&name) {
      let previous_ret_ty = &self.module.funs[fn_id].ret_ty;
      let previous_param_tys = &self.module.funs[fn_id].param_tys;
      if &ret_ty == previous_ret_ty && &param_tys == previous_param_tys {
        Ok(fn_id)
      } else {
        err!("function type differs from the previous declaration")
      }
    } else {
      let fun = Fun::new(name, ret_ty, param_tys);
      let fun_id = self.module.add_function(fun);
      Ok(fun_id)
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
      Stmt::IfElse(cond, then, else_) => self.gen_if_else(cond, then, else_),
      Stmt::For(init, cond, inc, body) => self.gen_for(init, cond, inc, *body),
      Stmt::Break => {
        insert_fun!(self).build_unconditional_branch(*self.break_label.last().unwrap());
        Ok(true)
      }
      Stmt::Cont => {
        insert_fun!(self).build_unconditional_branch(*self.cont_label.last().unwrap());
        Ok(true)
      }
      Stmt::Return(expr) => {
        let v1 = self.gen_expr(expr)?;
        insert_fun!(self).build_ret(v1);
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
    }
  }

  fn create_entry_block_alloca(&mut self, ty: Type, name: String) -> MemId {
    // Push mem_arena
    let mem_id = insert_fun!(self).build_alloca();
    // Insert scope
    self.scope.insert(name, mem_id);
    mem_id
  }

  fn gen_if_else(
    &mut self,
    cond: AST,
    then: Box<Stmt>,
    else_: Option<Box<Stmt>>,
  ) -> Expected<bool> {
    let current_block = insert_fun!(self).get_insert_block();
    let then_block = insert_fun!(self)
      .insert_basic_block_after(current_block)
      .unwrap();
    let else_block = insert_fun!(self)
      .insert_basic_block_after(then_block)
      .unwrap();
    let merge_block = if else_.is_some() {
      insert_fun!(self)
        .insert_basic_block_after(else_block)
        .unwrap()
    } else {
      else_block
    };

    // cond:
    let expr = self.gen_expr(cond)?;
    insert_fun!(self).build_conditional_branch(expr, then_block, else_block);

    // then:
    insert_fun!(self).position_at_end(then_block);
    let has_terminator_in_then = self.gen_stmt(*then)?;
    if !has_terminator_in_then {
      insert_fun!(self).build_unconditional_branch(merge_block);
    }

    // else:
    let has_terminator_in_else = if let Some(else_) = else_ {
      insert_fun!(self).position_at_end(else_block);
      let has_terminator = self.gen_stmt(*else_)?;
      if !has_terminator {
        insert_fun!(self).build_unconditional_branch(merge_block);
      }
      has_terminator
    } else {
      false
    };

    // merge:
    if has_terminator_in_then && has_terminator_in_else {
      insert_fun!(self).remove_basic_block(merge_block);
      Ok(true)
    } else {
      insert_fun!(self).position_at_end(merge_block);
      Ok(false)
    }
  }

  fn gen_for(
    &mut self,
    init: Option<AST>,
    cond: Option<AST>,
    inc: Option<AST>,
    body: Stmt,
  ) -> Expected<bool> {
    let current_block = insert_fun!(self).get_insert_block();
    let cond_block = insert_fun!(self)
      .insert_basic_block_after(current_block)
      .unwrap();
    let body_block = insert_fun!(self)
      .insert_basic_block_after(cond_block)
      .unwrap();
    let inc_block = insert_fun!(self)
      .insert_basic_block_after(body_block)
      .unwrap();
    let end_block = insert_fun!(self)
      .insert_basic_block_after(inc_block)
      .unwrap();
    self.break_label.push(end_block.clone());
    self.cont_label.push(inc_block.clone());

    // init:
    if let Some(expr) = init {
      self.gen_expr(expr)?;
    }
    insert_fun!(self).build_unconditional_branch(cond_block);

    // cond:
    insert_fun!(self).position_at_end(cond_block);
    if let Some(expr) = cond {
      let expr = self.gen_expr(expr)?;
      insert_fun!(self).build_conditional_branch(expr, body_block, end_block);
    } else {
      insert_fun!(self).build_unconditional_branch(body_block);
    }

    // body:
    insert_fun!(self).position_at_end(body_block);
    let has_terminator_in_body = self.gen_stmt(body)?;
    if !has_terminator_in_body {
      insert_fun!(self).build_unconditional_branch(inc_block);
    }

    // inc:
    insert_fun!(self).position_at_end(inc_block);
    if let Some(expr) = inc {
      self.gen_expr(expr)?;
    }
    insert_fun!(self).build_unconditional_branch(cond_block);

    // end:
    let has_no_branch_to_end = insert_fun!(self).bb_arena[end_block].pred.is_empty();
    if has_no_branch_to_end {
      insert_fun!(self).remove_basic_block(end_block);
    } else {
      insert_fun!(self).position_at_end(end_block);
    }

    self.break_label.pop();
    self.cont_label.pop();
    Ok(has_no_branch_to_end)
  }

  // ----- gen_expr -----

  fn gen_expr(&mut self, expr: AST) -> Expected<Val> {
    match expr {
      AST::Eq(n, m) => {
        let v1 = self.gen_expr(*n)?;
        let v2 = self.gen_expr(*m)?;
        let r0 = insert_fun!(self).build_inst(InstArgs::Eq(v1, v2));
        Ok(Val::Reg(r0))
      }
      AST::Ne(n, m) => {
        let v1 = self.gen_expr(*n)?;
        let v2 = self.gen_expr(*m)?;
        let r0 = insert_fun!(self).build_inst(InstArgs::Ne(v1, v2));
        Ok(Val::Reg(r0))
      }
      AST::Lt(n, m) => {
        let v1 = self.gen_expr(*n)?;
        let v2 = self.gen_expr(*m)?;
        let r0 = insert_fun!(self).build_inst(InstArgs::Lt(v1, v2));
        Ok(Val::Reg(r0))
      }
      AST::Le(n, m) => {
        let v1 = self.gen_expr(*n)?;
        let v2 = self.gen_expr(*m)?;
        let r0 = insert_fun!(self).build_inst(InstArgs::Le(v1, v2));
        Ok(Val::Reg(r0))
      }
      AST::Add(n, m) => {
        let v1 = self.gen_expr(*n)?;
        let v2 = self.gen_expr(*m)?;
        let r0 = insert_fun!(self).build_inst(InstArgs::Add(v1, v2));
        Ok(Val::Reg(r0))
      }
      AST::Sub(n, m) => {
        let v1 = self.gen_expr(*n)?;
        let v2 = self.gen_expr(*m)?;
        let r0 = insert_fun!(self).build_inst(InstArgs::Sub(v1, v2));
        Ok(Val::Reg(r0))
      }
      AST::Mul(n, m) => {
        let v1 = self.gen_expr(*n)?;
        let v2 = self.gen_expr(*m)?;
        let r0 = insert_fun!(self).build_inst(InstArgs::Mul(v1, v2));
        Ok(Val::Reg(r0))
      }
      AST::Div(n, m) => {
        let v1 = self.gen_expr(*n)?;
        let v2 = self.gen_expr(*m)?;
        let r0 = insert_fun!(self).build_inst(InstArgs::Div(v1, v2));
        Ok(Val::Reg(r0))
      }
      AST::Call(name, args) => {
        if let Some(fun) = self.module.get_function(&name) {
          let args = args
            .into_iter()
            .map(|expr| self.gen_expr(expr))
            .collect::<Result<Vec<_>, _>>()?;
          // TODO: type check
          // let arg_types: Vec<_> = args.iter().map(|arg| arg.get_type()).collect();
          // let param_types = &self.module.funs[fun].param_tys;
          // if arg_types != param_types {
          //   return err!("argument types mismatch function parameter types");
          // }

          let r0 = insert_fun!(self).build_inst(InstArgs::Call(fun, args));
          Ok(Val::Reg(r0))
        } else {
          err!("function does not exist")
        }
      }
      AST::Num(n) => Ok(Val::Imm(n)),
      AST::Assign(..) | AST::Ident(..) => {
        let mem = self.gen_addr(expr)?;
        // TODO: check if mem.get_type().get_element_type().is_array_type()
        if false {
          todo!()
          // Ok(self.gen_array_addr_impl(mem))
        } else {
          let r0 = insert_fun!(self).build_inst(InstArgs::Load(mem));
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
      insert_fun!(self).build_store(mem, rhs);
      Ok(mem)
    } else {
      err!("inconsistent types in operands of assignment")
    }
  }
}
