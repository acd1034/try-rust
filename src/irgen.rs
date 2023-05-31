use crate::common::{self, Expected};
use crate::err;
use crate::ir::{
  block::*, builder::*, builder_trait::*, function::*, inst::*, memory::*, module::*,
};
use crate::parse::{Stmt, TopLevel, AST};
use crate::ty::Type;
type Scope = common::Scope<MemoryId>;

// ----- irgen -----

pub struct IRGen {
  module: Module,
}

impl IRGen {
  pub fn new(name: String) -> IRGen {
    IRGen {
      module: Module::new(name),
    }
  }

  pub fn irgen(mut self, funs: Vec<TopLevel>) -> Expected<Module> {
    for fun in funs {
      self.gen_toplevel(fun)?;
    }
    Ok(self.module)
  }

  fn gen_toplevel(&mut self, fun: TopLevel) -> Expected<()> {
    match fun {
      TopLevel::FunDecl(ret_ty, name, param_tys) => {
        self.gen_fun_decl(ret_ty, name, param_tys)?;
        Ok(())
      }
      TopLevel::FunDef(ret_ty, name, param_tys, param_names, body) => {
        // Check consistency with forward declaration
        let fun_id = self.gen_fun_decl(ret_ty, name, param_tys)?;
        let new_fun = self.module.functions_get(fun_id).clone();
        let new_fun = GenFun::new(&mut self.module, new_fun).gen_fun(param_names, body)?;
        self.module.replace_function(fun_id, new_fun);
        Ok(())
      }
      TopLevel::VarDef(..) => todo!(),
      TopLevel::StructDef(..) => todo!(),
    }
  }

  fn gen_fun_decl(
    &mut self,
    ret_ty: Type,
    name: String,
    param_tys: Vec<Type>,
  ) -> Expected<FunctionId> {
    if let Some(fun_id) = self.module.get_function(&name) {
      let previous_ret_ty = self.module.functions_get(fun_id).ret_ty();
      let previous_param_tys = self.module.functions_get(fun_id).param_tys();
      if &ret_ty == previous_ret_ty && &param_tys == previous_param_tys {
        Ok(fun_id)
      } else {
        err!("function type differs from the previous declaration")
      }
    } else {
      let fun = Function::new(name, ret_ty, param_tys);
      let fun_id = self.module.add_function(fun);
      Ok(fun_id)
    }
  }
}

struct GenFun<'a> {
  module: &'a mut Module,
  builder: Builder,
  scope: Scope,
  break_label: Vec<BlockId>,
  cont_label: Vec<BlockId>,
}

impl<'a> GenFun<'a> {
  fn new(module: &mut Module, function: Function) -> GenFun {
    GenFun {
      module,
      builder: Builder::new(function),
      scope: Scope::new(),
      break_label: Vec::new(),
      cont_label: Vec::new(),
    }
  }

  // ----- gen_fun -----

  fn gen_fun(mut self, param_names: Vec<String>, body: Vec<Stmt>) -> Expected<Function> {
    // Check function is not defined
    if !self.builder.function().is_declaration() {
      return err!("function already exists");
    }

    // Create entry block
    let bb = self.builder.append_basic_block();
    self.builder.position_at_end(bb);

    // Push first scope
    self.scope.push();

    // Add function parameters
    let param_tys: Vec<_> = self
      .builder
      .function()
      .param_tys()
      .iter()
      .cloned()
      .collect();
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

    Ok(self.builder.retrieve_function())
  }

  // ----- gen_stmt -----

  fn gen_stmt(&mut self, stmt: Stmt) -> Expected<bool> {
    match stmt {
      Stmt::VarDef(var_defs) => {
        for (ty, name, init) in var_defs.into_iter() {
          if self.scope.get(&name).is_some() {
            return err!("variable already exists");
          }

          let mem = self.create_entry_block_alloca(ty, name);
          if let Some(expr) = init {
            let rhs = self.gen_expr(expr)?;
            self.gen_assign_impl(mem, rhs)?;
          }
        }
        Ok(false)
      }
      Stmt::StructDef(..) => todo!(),
      Stmt::IfElse(cond, then, else_) => self.gen_if_else(cond, then, else_),
      Stmt::For(init, cond, inc, body) => self.gen_for(init, cond, inc, *body),
      Stmt::Break => {
        self
          .builder
          .build_unconditional_branch(*self.break_label.last().unwrap());
        Ok(true)
      }
      Stmt::Cont => {
        self
          .builder
          .build_unconditional_branch(*self.cont_label.last().unwrap());
        Ok(true)
      }
      Stmt::Return(expr) => {
        let v1 = self.gen_expr(expr)?;
        self.builder.build_return(v1);
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

  fn create_entry_block_alloca(&mut self, _ty: Type, name: String) -> MemoryId {
    // Push mem_arena
    let mem_id = self.builder.build_alloca();
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
    let current_block = self.builder.get_insert_block();
    let then_block = self.builder.insert_basic_block_after(current_block);
    let else_block = self.builder.insert_basic_block_after(then_block);
    let merge_block = if else_.is_some() {
      self.builder.insert_basic_block_after(else_block)
    } else {
      else_block
    };

    // cond:
    let expr = self.gen_expr(cond)?;
    self
      .builder
      .build_conditional_branch(expr, then_block, else_block);

    // then:
    self.builder.position_at_end(then_block);
    let has_terminator_in_then = self.gen_stmt(*then)?;
    if !has_terminator_in_then {
      self.builder.build_unconditional_branch(merge_block);
    }

    // else:
    let has_terminator_in_else = if let Some(else_) = else_ {
      self.builder.position_at_end(else_block);
      let has_terminator = self.gen_stmt(*else_)?;
      if !has_terminator {
        self.builder.build_unconditional_branch(merge_block);
      }
      has_terminator
    } else {
      false
    };

    // merge:
    if has_terminator_in_then && has_terminator_in_else {
      self.builder.remove_basic_block(merge_block);
      Ok(true)
    } else {
      self.builder.position_at_end(merge_block);
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
    let current_block = self.builder.get_insert_block();
    let cond_block = self.builder.insert_basic_block_after(current_block);
    let body_block = self.builder.insert_basic_block_after(cond_block);
    let inc_block = self.builder.insert_basic_block_after(body_block);
    let end_block = self.builder.insert_basic_block_after(inc_block);
    self.break_label.push(end_block.clone());
    self.cont_label.push(inc_block.clone());

    // init:
    if let Some(expr) = init {
      self.gen_expr(expr)?;
    }
    self.builder.build_unconditional_branch(cond_block);

    // cond:
    self.builder.position_at_end(cond_block);
    if let Some(expr) = cond {
      let expr = self.gen_expr(expr)?;
      self
        .builder
        .build_conditional_branch(expr, body_block, end_block);
    } else {
      self.builder.build_unconditional_branch(body_block);
    }

    // body:
    self.builder.position_at_end(body_block);
    let has_terminator_in_body = self.gen_stmt(body)?;
    if !has_terminator_in_body {
      self.builder.build_unconditional_branch(inc_block);
    }

    // inc:
    self.builder.position_at_end(inc_block);
    if let Some(expr) = inc {
      self.gen_expr(expr)?;
    }
    self.builder.build_unconditional_branch(cond_block);

    // end:
    let has_no_branch_to_end = self.builder.function().get(end_block).pred().is_empty();
    if has_no_branch_to_end {
      self.builder.remove_basic_block(end_block);
    } else {
      self.builder.position_at_end(end_block);
    }

    self.break_label.pop();
    self.cont_label.pop();
    Ok(has_no_branch_to_end)
  }

  // ----- gen_expr -----

  fn gen_expr(&mut self, expr: AST) -> Expected<InstId> {
    match expr {
      AST::Eq(n, m) => {
        let v1 = self.gen_expr(*n)?;
        let v2 = self.gen_expr(*m)?;
        let v0 = self.builder.build_eq(v1, v2);
        Ok(v0)
      }
      AST::Ne(n, m) => {
        let v1 = self.gen_expr(*n)?;
        let v2 = self.gen_expr(*m)?;
        let v0 = self.builder.build_ne(v1, v2);
        Ok(v0)
      }
      AST::Lt(n, m) => {
        let v1 = self.gen_expr(*n)?;
        let v2 = self.gen_expr(*m)?;
        let v0 = self.builder.build_lt(v1, v2);
        Ok(v0)
      }
      AST::Le(n, m) => {
        let v1 = self.gen_expr(*n)?;
        let v2 = self.gen_expr(*m)?;
        let v0 = self.builder.build_le(v1, v2);
        Ok(v0)
      }
      AST::Add(n, m) => {
        let v1 = self.gen_expr(*n)?;
        let v2 = self.gen_expr(*m)?;
        let v0 = self.builder.build_add(v1, v2);
        Ok(v0)
      }
      AST::Sub(n, m) => {
        let v1 = self.gen_expr(*n)?;
        let v2 = self.gen_expr(*m)?;
        let v0 = self.builder.build_sub(v1, v2);
        Ok(v0)
      }
      AST::Mul(n, m) => {
        let v1 = self.gen_expr(*n)?;
        let v2 = self.gen_expr(*m)?;
        let v0 = self.builder.build_mul(v1, v2);
        Ok(v0)
      }
      AST::Div(n, m) => {
        let v1 = self.gen_expr(*n)?;
        let v2 = self.gen_expr(*m)?;
        let v0 = self.builder.build_div(v1, v2);
        Ok(v0)
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

          let v0 = self.builder.build_call(fun, args);
          Ok(v0)
        } else {
          err!("function does not exist")
        }
      }
      AST::Num(n) => Ok(self.builder.build_const(n)),
      AST::Assign(..) | AST::Ident(..) => {
        let mem = self.gen_addr(expr)?;
        // TODO: check if mem.get_type().get_element_type().is_array_type()
        if false {
          todo!()
          // Ok(self.gen_array_addr_impl(mem))
        } else {
          let v0 = self.builder.build_load(mem);
          Ok(v0)
        }
      }
      _ => todo!(),
    }
  }

  // ----- gen_addr -----

  fn gen_addr(&mut self, expr: AST) -> Expected<MemoryId> {
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

  fn gen_assign_impl(&mut self, mem: MemoryId, rhs: InstId) -> Expected<MemoryId> {
    // TODO: check if lhs.get_type().get_element_type() == rhs.get_type().as_any_type_enum()
    if true {
      self.builder.build_store(mem, rhs);
      Ok(mem)
    } else {
      err!("inconsistent types in operands of assignment")
    }
  }
}
