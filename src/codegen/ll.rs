use crate::parse::{Fun, Stmt, Type, AST};
use crate::sema;
use crate::{common::Expected, err};
use inkwell::basic_block::BasicBlock;
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::types::*;
use inkwell::values::*;
use inkwell::AddressSpace;
use inkwell::IntPredicate;
type Scope<'a> = sema::Scope<PointerValue<'a>>;

// Module ⊇ Function ⊇ BasicBlock ⊇ Instruction
pub struct CodeGen<'ctx> {
  context: &'ctx Context,
}

impl<'ctx> CodeGen<'ctx> {
  pub fn new(context: &'ctx Context) -> CodeGen<'ctx> {
    CodeGen { context }
  }

  pub fn codegen(self, funs: Vec<Fun>) -> Expected<String> {
    let module = self.context.create_module("mod");
    for fun in funs {
      GenFun::new(self.context, &module).gen_fun(fun)?;
    }
    Ok(module.to_string())
  }
}

struct GenFun<'a, 'ctx> {
  context: &'ctx Context,
  module: &'a Module<'ctx>,
  builder: Builder<'ctx>,
  scope: Scope<'ctx>,
  break_label: Vec<BasicBlock<'ctx>>,
  cont_label: Vec<BasicBlock<'ctx>>,
}

impl<'a, 'ctx> GenFun<'a, 'ctx> {
  fn new(context: &'ctx Context, module: &'a Module<'ctx>) -> GenFun<'a, 'ctx> {
    let builder = context.create_builder();
    let scope = Scope::new();
    let break_label = Vec::new();
    let cont_label = Vec::new();
    GenFun {
      context,
      module,
      builder,
      scope,
      break_label,
      cont_label,
    }
  }

  fn into_inkwell_type(&self, ty: Type) -> BasicTypeEnum<'ctx> {
    match ty {
      Type::Int => self.context.i64_type().as_basic_type_enum(),
      Type::Pointer(ty) => self
        .into_inkwell_type(*ty)
        .ptr_type(AddressSpace::default())
        .as_basic_type_enum(),
      Type::Array(ty, size) => self
        .into_inkwell_type(*ty)
        .array_type(size)
        .as_basic_type_enum(),
    }
  }

  fn get_current_basic_block(&self) -> BasicBlock<'ctx> {
    self.builder.get_insert_block().unwrap()
  }

  fn get_current_fun(&self) -> FunctionValue<'ctx> {
    self.get_current_basic_block().get_parent().unwrap()
  }

  // Creates a new stack allocation instruction in the entry block of the function
  fn create_entry_block_alloca(
    &mut self,
    var_type: BasicTypeEnum<'ctx>,
    name: String,
  ) -> PointerValue<'ctx> {
    let fn_value = self.get_current_fun();
    let entry_block = fn_value.get_first_basic_block().unwrap();
    let builder = self.context.create_builder();
    match entry_block.get_first_instruction() {
      Some(inst) => builder.position_before(&inst),
      None => builder.position_at_end(entry_block),
    }

    let alloca = builder.build_alloca(var_type, &name);
    self.scope.insert(name, alloca);
    alloca
  }

  // ----- gen_fun -----

  fn gen_fun(mut self, fun: Fun) -> Expected<FunctionValue<'ctx>> {
    match fun {
      Fun::FunDecl(ret_ty, name, param_tys) => self.gen_fun_decl(ret_ty, &name, param_tys),
      Fun::FunDef(ret_ty, name, param_tys, param_names, body) => {
        self.gen_fun_def(ret_ty, &name, param_tys, param_names, body)
      }
    }
  }

  fn gen_fun_decl(
    &self,
    ret_ty: Type,
    name: &str,
    param_tys: Vec<Type>,
  ) -> Expected<FunctionValue<'ctx>> {
    if let Some(fn_value) = self.module.get_function(name) {
      let stored_fn_type = fn_value.get_type();
      let return_type = self.into_inkwell_type(ret_ty);
      let param_types: Vec<_> = param_tys
        .into_iter()
        .map(|ty| self.into_inkwell_type(ty))
        .collect();
      if return_type == stored_fn_type.get_return_type().unwrap()
        && param_types == stored_fn_type.get_param_types()
      {
        Ok(fn_value)
      } else {
        err!("function type differs from the previous declaration")
      }
    } else {
      let return_type = self.into_inkwell_type(ret_ty);
      let param_types: Vec<_> = param_tys
        .into_iter()
        .map(|ty| self.into_inkwell_type(ty).into())
        .collect();
      let fn_type = return_type.fn_type(param_types.as_slice(), false);
      Ok(self.module.add_function(name, fn_type, None))
    }
  }

  fn gen_fun_def(
    &mut self,
    ret_ty: Type,
    name: &str,
    param_tys: Vec<Type>,
    param_names: Vec<String>,
    body: Vec<Stmt>,
  ) -> Expected<FunctionValue<'ctx>> {
    assert_eq!(param_tys.len(), param_names.len());
    // Check consistency with forward declaration
    let fn_value = self.gen_fun_decl(ret_ty, &name, param_tys)?;
    // Check function does not exist
    if fn_value.count_basic_blocks() != 0 {
      return err!("function already exists");
    }

    // Create first basic block
    let entry_block = self.context.append_basic_block(fn_value, "entry");
    self.builder.position_at_end(entry_block);
    // Create first variable scope
    self.scope.push();
    // Allocate function parameters
    for (name, param) in std::iter::zip(param_names, fn_value.get_param_iter()) {
      if self.scope.get(&name).is_none() {
        let alloca = self.create_entry_block_alloca(param.get_type(), name);
        self.builder.build_store(alloca, param);
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
    // Remove first scope
    self.scope.pop();

    // Check terminator
    if !has_terminator {
      return err!("no terminator in function");
    }

    if fn_value.verify(true) {
      Ok(fn_value)
    } else {
      // TODO: 前方宣言後の定義ならば定義のみ消す。前方宣言なしの定義ならば宣言ごと消す
      // unsafe { fn_value.delete(); }
      err!("failed to verify function")
    }
  }

  // ----- gen_stmt -----

  // Returns if the last basic block has a terminator
  fn gen_stmt(&mut self, stmt: Stmt) -> Expected<bool> {
    match stmt {
      Stmt::VarDef(ty, name, init) => {
        if self.scope.get(&name).is_some() {
          return err!("variable already exists");
        }

        let var_type = self.into_inkwell_type(ty);
        let alloca = self.create_entry_block_alloca(var_type, name);
        if let Some(expr) = init {
          let rhs = self.gen_expr(expr)?;
          self.gen_assign_impl(alloca, rhs)?;
        }
        Ok(false)
      }
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
        let ret = self.gen_expr(expr)?;
        self.builder.build_return(Some(&ret));
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

  fn gen_if_else(
    &mut self,
    cond: AST,
    then: Box<Stmt>,
    else_: Option<Box<Stmt>>,
  ) -> Expected<bool> {
    /* `if (A) B else C`
     *   A != 0 ? goto then : goto else;
     * then:
     *   B;
     *   goto merge;
     * else:
     *   C;
     *   goto merge;
     * merge:
     */
    let current_block = self.get_current_basic_block();
    let then_block = self.context.insert_basic_block_after(current_block, "then");
    let else_block = self.context.insert_basic_block_after(then_block, "else");
    let merge_block = if else_.is_some() {
      self.context.insert_basic_block_after(else_block, "cont")
    } else {
      else_block
    };

    // cond:
    let lhs = self.gen_expr_into_int_value(cond)?;
    let zero = lhs.get_type().const_int(0, false);
    let comp = self
      .builder
      .build_int_compare(IntPredicate::NE, lhs, zero, "cond");
    self
      .builder
      .build_conditional_branch(comp, then_block, else_block);

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
    self.builder.position_at_end(merge_block);
    if has_terminator_in_then && has_terminator_in_else {
      self.builder.build_unreachable();
      Ok(true)
    } else {
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
    /* `for (A; B; C) D`
     *   A;
     *   goto cond;
     * cond:
     *   B != 0 ? goto body : goto end;
     * body:
     *   D;
     *   goto inc;
     * inc:
     *   C;
     *   goto cond;
     * end:
     */
    let current_block = self.get_current_basic_block();
    let cond_block = self.context.insert_basic_block_after(current_block, "cond");
    let body_block = self.context.insert_basic_block_after(cond_block, "body");
    let inc_block = self.context.insert_basic_block_after(body_block, "inc");
    let end_block = self.context.insert_basic_block_after(inc_block, "end");
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
      let lhs = self.gen_expr_into_int_value(expr)?;
      let zero = lhs.get_type().const_int(0, false);
      let comp = self
        .builder
        .build_int_compare(IntPredicate::NE, lhs, zero, "cond");
      self
        .builder
        .build_conditional_branch(comp, body_block, end_block);
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
    self.builder.position_at_end(end_block);
    let has_no_branch_to_end = end_block.get_first_use().is_none();
    if has_no_branch_to_end {
      self.builder.build_unreachable();
    }

    self.break_label.pop();
    self.cont_label.pop();
    Ok(has_no_branch_to_end)
  }

  // ----- gen_expr -----

  fn gen_expr_into_int_value(&self, expr: AST) -> Expected<IntValue<'ctx>> {
    let value = self.gen_expr(expr)?;
    if value.is_int_value() {
      Ok(value.into_int_value())
    } else {
      err!("unexpected type in expression, expecting int type")
    }
  }

  fn gen_expr(&self, expr: AST) -> Expected<BasicValueEnum<'ctx>> {
    let i64_type = self.context.i64_type();
    match expr {
      AST::Ternary(cond, then, else_) => self.gen_ternary(*cond, *then, *else_),
      AST::Eq(n, m) => {
        let lhs = self.gen_expr_into_int_value(*n)?;
        let rhs = self.gen_expr_into_int_value(*m)?;
        let cmp = self
          .builder
          .build_int_compare(IntPredicate::EQ, lhs, rhs, "tmpcmp");
        let zext = self
          .builder
          .build_int_z_extend(cmp, i64_type, "tmpzext")
          .as_basic_value_enum();
        Ok(zext)
      }
      AST::Ne(n, m) => {
        let lhs = self.gen_expr_into_int_value(*n)?;
        let rhs = self.gen_expr_into_int_value(*m)?;
        let cmp = self
          .builder
          .build_int_compare(IntPredicate::NE, lhs, rhs, "tmpcmp");
        let zext = self
          .builder
          .build_int_z_extend(cmp, i64_type, "tmpzext")
          .as_basic_value_enum();
        Ok(zext)
      }
      AST::Lt(n, m) => {
        let lhs = self.gen_expr_into_int_value(*n)?;
        let rhs = self.gen_expr_into_int_value(*m)?;
        let cmp = self
          .builder
          .build_int_compare(IntPredicate::SLT, lhs, rhs, "tmpcmp");
        let zext = self
          .builder
          .build_int_z_extend(cmp, i64_type, "tmpzext")
          .as_basic_value_enum();
        Ok(zext)
      }
      AST::Le(n, m) => {
        let lhs = self.gen_expr_into_int_value(*n)?;
        let rhs = self.gen_expr_into_int_value(*m)?;
        let cmp = self
          .builder
          .build_int_compare(IntPredicate::SLE, lhs, rhs, "tmpcmp");
        let zext = self
          .builder
          .build_int_z_extend(cmp, i64_type, "tmpzext")
          .as_basic_value_enum();
        Ok(zext)
      }
      AST::Add(n, m) => {
        let lhs = self.gen_expr(*n)?;
        let rhs = self.gen_expr(*m)?;
        match (lhs, rhs) {
          (BasicValueEnum::IntValue(lhs), BasicValueEnum::IntValue(rhs)) => {
            let res = self
              .builder
              .build_int_add(lhs, rhs, "tmpadd")
              .as_basic_value_enum();
            Ok(res)
          }
          (BasicValueEnum::PointerValue(ptr), BasicValueEnum::IntValue(idx)) => {
            Ok(self.gen_pointer_add_impl(ptr, idx))
          }
          (BasicValueEnum::IntValue(idx), BasicValueEnum::PointerValue(ptr)) => {
            Ok(self.gen_pointer_add_impl(ptr, idx))
          }
          _ => err!("inconsistent types in operands of addition"),
        }
      }
      AST::Sub(n, m) => {
        let lhs = self.gen_expr(*n)?;
        let rhs = self.gen_expr(*m)?;
        match (lhs, rhs) {
          (BasicValueEnum::IntValue(lhs), BasicValueEnum::IntValue(rhs)) => {
            let res = self
              .builder
              .build_int_sub(lhs, rhs, "tmpadd")
              .as_basic_value_enum();
            Ok(res)
          }
          (BasicValueEnum::PointerValue(ptr), BasicValueEnum::IntValue(idx)) => {
            let idx = self.builder.build_int_neg(idx, "tmpneg");
            Ok(self.gen_pointer_add_impl(ptr, idx))
          }
          (BasicValueEnum::PointerValue(lhs), BasicValueEnum::PointerValue(rhs)) => {
            if lhs.get_type() != rhs.get_type() {
              return err!("inconsistent types in operands of pointer difference");
            }
            let res = self
              .builder
              .build_ptr_diff(lhs, rhs, "tmp_ptr_diff")
              .as_basic_value_enum();
            Ok(res)
          }
          _ => err!("inconsistent types in operands of subtraction"),
        }
      }
      AST::Mul(n, m) => {
        let lhs = self.gen_expr_into_int_value(*n)?;
        let rhs = self.gen_expr_into_int_value(*m)?;
        let res = self
          .builder
          .build_int_mul(lhs, rhs, "tmpmul")
          .as_basic_value_enum();
        Ok(res)
      }
      AST::Div(n, m) => {
        let lhs = self.gen_expr_into_int_value(*n)?;
        let rhs = self.gen_expr_into_int_value(*m)?;
        let res = self
          .builder
          .build_int_signed_div(lhs, rhs, "tmpdiv")
          .as_basic_value_enum();
        Ok(res)
      }
      AST::Addr(n) => {
        let var = self.gen_addr(*n)?;
        if var.get_type().get_element_type().is_array_type() {
          Ok(self.gen_array_addr_impl(var))
        } else {
          Ok(var.as_basic_value_enum())
        }
      }
      AST::Call(name, args) => {
        if let Some(callee) = self.module.get_function(&name) {
          let stored_param_types = callee.get_type().get_param_types();
          let args = args
            .into_iter()
            .map(|expr| self.gen_expr(expr))
            .collect::<Result<Vec<_>, _>>()?;
          let arg_types: Vec<_> = args.iter().map(|arg| arg.get_type()).collect();
          if arg_types == stored_param_types {
            let args: Vec<_> = args.into_iter().map(|arg| arg.into()).collect();
            let res = self
              .builder
              .build_call(callee, args.as_slice(), "tmpcall")
              .try_as_basic_value()
              .unwrap_left();
            Ok(res)
          } else {
            err!("argument types mismatch function parameter types")
          }
        } else {
          err!("function does not exist")
        }
      }
      AST::Num(n) => Ok(i64_type.const_int(n, false).as_basic_value_enum()),
      AST::Assign(..) | AST::Deref(..) | AST::Ident(..) => {
        let var = self.gen_addr(expr)?;
        if var.get_type().get_element_type().is_array_type() {
          Ok(self.gen_array_addr_impl(var))
        } else {
          let res = self.builder.build_load(var, "tmpload");
          Ok(res)
        }
      }
    }
  }

  fn gen_ternary(&self, cond: AST, then: AST, else_: AST) -> Expected<BasicValueEnum<'ctx>> {
    let current_block = self.get_current_basic_block();
    let then_block = self.context.insert_basic_block_after(current_block, "then");
    let else_block = self.context.insert_basic_block_after(then_block, "else");
    let merge_block = self.context.insert_basic_block_after(else_block, "merge");

    // cond:
    let lhs = self.gen_expr_into_int_value(cond)?;
    let zero = lhs.get_type().const_int(0, false);
    let comp = self
      .builder
      .build_int_compare(IntPredicate::NE, lhs, zero, "cond");
    self
      .builder
      .build_conditional_branch(comp, then_block, else_block);

    // then:
    self.builder.position_at_end(then_block);
    let then_value = self.gen_expr(then)?;
    self.builder.build_unconditional_branch(merge_block);

    // else:
    self.builder.position_at_end(else_block);
    let else_value = self.gen_expr(else_)?;
    self.builder.build_unconditional_branch(merge_block);

    // merge:
    if then_value.get_type() != else_value.get_type() {
      return err!("inconsistent types in operands of ternary operator");
    }
    self.builder.position_at_end(merge_block);
    let phi = self.builder.build_phi(then_value.get_type(), "tmpphi");
    phi.add_incoming(&[(&then_value, then_block), (&else_value, else_block)]);
    Ok(phi.as_basic_value())
  }

  fn gen_pointer_add_impl(
    &self,
    ptr: PointerValue<'ctx>,
    idx: IntValue<'ctx>,
  ) -> BasicValueEnum<'ctx> {
    unsafe {
      self
        .builder
        .build_in_bounds_gep(ptr, &[idx], "tmpgep")
        .as_basic_value_enum()
    }
  }

  fn gen_array_addr_impl(&self, ptr: PointerValue<'ctx>) -> BasicValueEnum<'ctx> {
    let zero = self.context.i64_type().const_int(0, false);
    unsafe {
      self
        .builder
        .build_in_bounds_gep(ptr, &[zero, zero], "tmpgep")
        .as_basic_value_enum()
    }
  }

  // ----- gen_addr -----

  fn gen_addr(&self, expr: AST) -> Expected<PointerValue<'ctx>> {
    match expr {
      AST::Assign(n, m) => {
        let rhs = self.gen_expr(*m)?;
        let lhs = self.gen_addr(*n)?;
        self.gen_assign_impl(lhs, rhs)
      }
      AST::Deref(n) => {
        let ptr = self.gen_expr(*n)?;
        if ptr.is_pointer_value() {
          Ok(ptr.into_pointer_value())
        } else {
          err!("cannot dereference int value")
        }
      }
      AST::Ident(name) => match self.scope.get_all(&name) {
        Some(&var) => Ok(var),
        None => err!("variable should be declared before its first use"),
      },
      _ => err!("cannot obtain address of rvalue"),
    }
  }

  fn gen_assign_impl(
    &self,
    lhs: PointerValue<'ctx>,
    rhs: BasicValueEnum<'ctx>,
  ) -> Expected<PointerValue<'ctx>> {
    if lhs.get_type().get_element_type() == rhs.get_type().as_any_type_enum() {
      self.builder.build_store(lhs, rhs);
      Ok(lhs)
    } else {
      err!("inconsistent types in operands of assignment")
    }
  }
}
