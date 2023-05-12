use crate::common::{self, Expected};
use crate::err;
use crate::parse::{Stmt, TopLevel, AST};
use crate::ty::Type;
use inkwell::basic_block::BasicBlock;
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::{Linkage, Module};
use inkwell::types::*;
use inkwell::values::*;
use inkwell::AddressSpace;
use inkwell::IntPredicate;
type Scope<'a> = common::Scope<PointerValue<'a>>;

// Module ⊇ Function ⊇ BasicBlock ⊇ Instruction
pub struct CodeGen<'ctx> {
  context: &'ctx Context,
}

impl<'ctx> CodeGen<'ctx> {
  pub fn new(context: &'ctx Context) -> CodeGen<'ctx> {
    CodeGen { context }
  }

  pub fn codegen(self, toplevels: Vec<TopLevel>) -> Expected<String> {
    let module = self.context.create_module("mod");
    let mut scope = Scope::new();
    scope.push();
    for toplevel in toplevels {
      GenTopLevel::new(self.context, &module, &mut scope).gen_toplevel(toplevel)?;
    }
    scope.pop();
    Ok(module.to_string())
  }
}

enum StmtKind<'ctx> {
  Terminator,
  NoTerminator,
  Expr(BasicValueEnum<'ctx>),
}

impl<'ctx> StmtKind<'ctx> {
  fn is_terminator(&self) -> bool {
    if let StmtKind::Terminator = self {
      true
    } else {
      false
    }
  }
}

struct GenTopLevel<'a, 'ctx> {
  context: &'ctx Context,
  module: &'a Module<'ctx>,
  builder: Builder<'ctx>,
  scope: &'a mut Scope<'ctx>,
  break_label: Vec<BasicBlock<'ctx>>,
  cont_label: Vec<BasicBlock<'ctx>>,
}

impl<'a, 'ctx> GenTopLevel<'a, 'ctx> {
  fn new(
    context: &'ctx Context,
    module: &'a Module<'ctx>,
    scope: &'a mut Scope<'ctx>,
  ) -> GenTopLevel<'a, 'ctx> {
    let builder = context.create_builder();
    let break_label = Vec::new();
    let cont_label = Vec::new();
    GenTopLevel {
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
      Type::Char => self.context.i8_type().as_basic_type_enum(),
      Type::Pointer(ty) => self
        .into_inkwell_type(*ty)
        .ptr_type(AddressSpace::default())
        .as_basic_type_enum(),
      Type::Array(ty, size) => self
        .into_inkwell_type(*ty)
        .array_type(size)
        .as_basic_type_enum(),
      Type::FunTy(_ret_ty, _param_tys) => {
        todo!();
        // let return_type = self.into_inkwell_type(*_ret_ty);
        // let param_types: Vec<_> = _param_tys
        //   .into_iter()
        //   .map(|(ty, _name)| self.into_inkwell_type(ty).into())
        //   .collect();
        // return_type.fn_type(param_types.as_slice(), false)
      }
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

  // ----- gen_toplevel -----

  fn gen_toplevel(mut self, toplevel: TopLevel) -> Expected<AnyValueEnum<'ctx>> {
    match toplevel {
      TopLevel::FunDecl(ret_ty, name, param_tys) => self
        .gen_fun_decl(ret_ty, &name, param_tys)
        .map(|fun| fun.as_any_value_enum()),
      TopLevel::FunDef(ret_ty, name, param_tys, param_names, body) => self
        .gen_fun_def(ret_ty, &name, param_tys, param_names, body)
        .map(|fun| fun.as_any_value_enum()),
      TopLevel::VarDef(ty, name, init) => {
        if self.scope.get(&name).is_some() {
          return err!("global variable already exists");
        }

        let var_type = self.into_inkwell_type(ty);
        let var = self.module.add_global(var_type.clone(), None, &name);

        let value = if let Some(expr) = init {
          let rhs = self.gen_expr(expr)?;
          if var_type != rhs.get_type() {
            return err!("inconsistent types in operands of global assignment");
          }
          rhs
        } else {
          match var_type {
            BasicTypeEnum::IntType(int_type) => int_type.const_zero().as_basic_value_enum(),
            BasicTypeEnum::PointerType(ptr_type) => ptr_type.const_null().as_basic_value_enum(),
            BasicTypeEnum::ArrayType(array_type) => array_type.const_zero().as_basic_value_enum(),
            _ => todo!(),
          }
        };
        var.set_initializer(&value);
        self.scope.insert(name, var.as_pointer_value());
        Ok(var.as_any_value_enum())
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
    let mut stmt_kind = StmtKind::NoTerminator;
    for stmt in body {
      stmt_kind = self.gen_stmt(stmt)?;
      if stmt_kind.is_terminator() {
        break;
      }
    }
    // Remove first scope
    self.scope.pop();

    // Check terminator
    if !stmt_kind.is_terminator() {
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
  fn gen_stmt(&mut self, stmt: Stmt) -> Expected<StmtKind<'ctx>> {
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
        Ok(StmtKind::NoTerminator)
      }
      Stmt::IfElse(cond, then, else_) => self.gen_if_else(cond, then, else_),
      Stmt::For(init, cond, inc, body) => self.gen_for(init, cond, inc, *body),
      Stmt::Break => {
        self
          .builder
          .build_unconditional_branch(*self.break_label.last().unwrap());
        Ok(StmtKind::Terminator)
      }
      Stmt::Cont => {
        self
          .builder
          .build_unconditional_branch(*self.cont_label.last().unwrap());
        Ok(StmtKind::Terminator)
      }
      Stmt::Return(expr) => {
        let ret = self.gen_expr(expr)?;
        if Some(ret.get_type()) == self.get_current_fun().get_type().get_return_type() {
          self.builder.build_return(Some(&ret));
          Ok(StmtKind::Terminator)
        } else {
          err!("return type differs from the declaration")
        }
      }
      Stmt::Block(stmts) => self.gen_block(stmts),
      Stmt::Expr(expr) => {
        let value = self.gen_expr(expr)?;
        Ok(StmtKind::Expr(value))
      }
    }
  }

  fn gen_if_else(
    &mut self,
    cond: AST,
    then: Box<Stmt>,
    else_: Option<Box<Stmt>>,
  ) -> Expected<StmtKind<'ctx>> {
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
    let stmt_kind_in_then = self.gen_stmt(*then)?;
    if !stmt_kind_in_then.is_terminator() {
      self.builder.build_unconditional_branch(merge_block);
    }

    // else:
    let stmt_kind_in_else = if let Some(else_) = else_ {
      self.builder.position_at_end(else_block);
      let stmt_kind = self.gen_stmt(*else_)?;
      if !stmt_kind.is_terminator() {
        self.builder.build_unconditional_branch(merge_block);
      }
      stmt_kind
    } else {
      StmtKind::NoTerminator
    };

    // merge:
    self.builder.position_at_end(merge_block);
    if stmt_kind_in_then.is_terminator() && stmt_kind_in_else.is_terminator() {
      self.builder.build_unreachable();
      Ok(StmtKind::Terminator)
    } else {
      Ok(StmtKind::NoTerminator)
    }
  }

  fn gen_for(
    &mut self,
    init: Option<AST>,
    cond: Option<AST>,
    inc: Option<AST>,
    body: Stmt,
  ) -> Expected<StmtKind<'ctx>> {
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
    let stmt_kind_in_body = self.gen_stmt(body)?;
    if !stmt_kind_in_body.is_terminator() {
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
    let end_block_is_unreachable = end_block.get_first_use().is_none();
    if end_block_is_unreachable {
      self.builder.build_unreachable();
    }

    self.break_label.pop();
    self.cont_label.pop();
    if end_block_is_unreachable {
      Ok(StmtKind::Terminator)
    } else {
      Ok(StmtKind::NoTerminator)
    }
  }

  fn gen_block(&mut self, stmts: Vec<Stmt>) -> Expected<StmtKind<'ctx>> {
    self.scope.push();
    let mut stmt_kind = StmtKind::NoTerminator;
    for stmt in stmts {
      stmt_kind = self.gen_stmt(stmt)?;
      if stmt_kind.is_terminator() {
        break;
      }
    }
    self.scope.pop();
    Ok(stmt_kind)
  }

  // ----- gen_expr -----

  fn gen_expr_into_int_value(&mut self, expr: AST) -> Expected<IntValue<'ctx>> {
    let value = self.gen_expr(expr)?;
    if value.is_int_value() {
      Ok(value.into_int_value())
    } else {
      err!("unexpected type in expression, expecting int type")
    }
  }

  fn gen_expr(&mut self, expr: AST) -> Expected<BasicValueEnum<'ctx>> {
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
      AST::Cast(ty, n) => {
        let cast_type = self.into_inkwell_type(ty);
        let value = self.gen_expr(*n)?;
        match (cast_type, value) {
          (BasicTypeEnum::IntType(int_type), BasicValueEnum::IntValue(int_value)) => {
            if int_type.get_bit_width() > int_value.get_type().get_bit_width() {
              let res = self
                .builder
                .build_int_s_extend(int_value, int_type, "sext")
                .as_basic_value_enum();
              Ok(res)
            } else if int_type.get_bit_width() < int_value.get_type().get_bit_width() {
              let res = self
                .builder
                .build_int_truncate(int_value, int_type, "trunc")
                .as_basic_value_enum();
              Ok(res)
            } else {
              Ok(int_value.as_basic_value_enum())
            }
          }
          _ => todo!(),
        }
      }
      AST::Block(stmts) => {
        let stmt_kind = self.gen_block(stmts)?;
        if let StmtKind::Expr(value) = stmt_kind {
          Ok(value)
        } else {
          err!("GNU statement expression does not end with expression statement")
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
          if arg_types != stored_param_types {
            return err!("argument types mismatch function parameter types");
          }

          let args: Vec<_> = args.into_iter().map(|arg| arg.into()).collect();
          let res = self
            .builder
            .build_call(callee, args.as_slice(), "tmpcall")
            .try_as_basic_value()
            .unwrap_left();
          Ok(res)
        } else {
          err!("function does not exist")
        }
      }
      AST::Num(n) => Ok(i64_type.const_int(n, false).as_basic_value_enum()),
      AST::Str(s) => {
        let value = self.context.const_string(s.as_bytes(), true);
        let global = self.module.add_global(value.get_type(), None, ".str");
        global.set_initializer(&value);
        global.set_linkage(Linkage::Private);
        global.set_constant(true);
        let ptr = global.as_pointer_value();
        Ok(self.gen_array_addr_impl(ptr))
      }
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

  fn gen_ternary(&mut self, cond: AST, then: AST, else_: AST) -> Expected<BasicValueEnum<'ctx>> {
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

  fn gen_addr(&mut self, expr: AST) -> Expected<PointerValue<'ctx>> {
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
