use crate::parse::{Function, Stmt, Type, AST};
use crate::tokenize::Expected;
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use std::collections::HashMap;

use inkwell::basic_block::BasicBlock;
use inkwell::types::*;
use inkwell::values::*;
use inkwell::AddressSpace;
use inkwell::IntPredicate;

// Module ⊇ Function ⊇ BasicBlock ⊇ Instruction
pub struct CodeGen<'ctx> {
  context: &'ctx Context,
}

impl<'ctx> CodeGen<'ctx> {
  pub fn new(context: &'ctx Context) -> CodeGen<'ctx> {
    CodeGen { context }
  }

  pub fn codegen(&self, functions: Vec<Function>) -> Expected<String> {
    let module = self.context.create_module("mod");
    for function in functions {
      GenFunction::new(self.context, &module).gen_function(function)?;
    }
    Ok(module.to_string())
  }
}

struct GenFunction<'a, 'ctx> {
  context: &'ctx Context,
  module: &'a Module<'ctx>,
  builder: Builder<'ctx>,
}

impl<'a, 'ctx> GenFunction<'a, 'ctx> {
  fn new(context: &'ctx Context, module: &'a Module<'ctx>) -> GenFunction<'a, 'ctx> {
    let builder = context.create_builder();
    GenFunction {
      context,
      module,
      builder,
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

  fn get_current_function(&self) -> FunctionValue<'ctx> {
    self.get_current_basic_block().get_parent().unwrap()
  }

  // Creates a new stack allocation instruction in the entry block of the function
  fn create_entry_block_alloca(
    &self,
    var_type: BasicTypeEnum<'ctx>,
    name: String,
    vars: &mut HashMap<String, PointerValue<'ctx>>,
  ) -> PointerValue<'ctx> {
    let fn_value = self.get_current_function();
    let entry_block = fn_value.get_first_basic_block().unwrap();
    let builder = self.context.create_builder();
    match entry_block.get_first_instruction() {
      Some(inst) => builder.position_before(&inst),
      None => builder.position_at_end(entry_block),
    }

    let alloca = builder.build_alloca(var_type, &name);
    vars.insert(name, alloca);
    alloca
  }

  fn check_prototype(
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
        Err("function type differs from the previous declaration")
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

  fn gen_function(&self, function: Function) -> Expected<FunctionValue<'ctx>> {
    match function {
      Function::Function(ret_ty, name, param_tys, param_names, body) => {
        assert_eq!(param_tys.len(), param_names.len());
        let fn_value = self.check_prototype(ret_ty, &name, param_tys)?;
        if fn_value.count_basic_blocks() == 0 {
          let entry_block = self.context.append_basic_block(fn_value, "entry");
          self.builder.position_at_end(entry_block);
          let mut vars: HashMap<String, PointerValue<'ctx>> = HashMap::new();
          for (name, param) in std::iter::zip(param_names, fn_value.get_param_iter()) {
            if vars.get(&name).is_none() {
              let alloca = self.create_entry_block_alloca(param.get_type(), name, &mut vars);
              self.builder.build_store(alloca, param);
            } else {
              return Err("function parameter already defined");
            }
          }

          let has_terminator = self.gen_statement(body, &mut vars)?;
          if !has_terminator {
            return Err("gen_function: no terminator in function");
          }

          if fn_value.verify(true) {
            Ok(fn_value)
          } else {
            // TODO: 前方宣言後の定義ならば定義のみ消す。前方宣言なしの定義ならば宣言ごと消す
            unsafe {
              fn_value.delete();
            }
            Err("gen_function: failed to verify function")
          }
        } else {
          Err("function already defined")
        }
      }
      Function::Prototype(ret_ty, name, param_tys) => {
        self.check_prototype(ret_ty, &name, param_tys)
      }
    }
  }

  // Returns if the last basic block has a terminator
  fn gen_statement(
    &self,
    stmt: Stmt,
    vars: &mut HashMap<String, PointerValue<'ctx>>,
  ) -> Expected<bool> {
    let i64_type = self.context.i64_type();
    match stmt {
      Stmt::Decl(ty, name, init) => {
        if vars.get(&name).is_none() {
          let var_type = self.into_inkwell_type(ty);
          let alloca = self.create_entry_block_alloca(var_type, name, vars);
          if let Some(expr) = init {
            let rhs = self.gen_expr(expr, vars)?;
            self.gen_assign_impl(alloca, rhs)?;
          }
          Ok(false)
        } else {
          Err("variable already defined")
        }
      }
      Stmt::IfElse(cond, then_stmt, else_stmt) => {
        /* `if (A) B else C`
         *   A != 0 ? goto then : goto else;
         * then:
         *   B;
         *   goto cont;
         * else:
         *   C;
         * cont:
         */
        let current_block = self.get_current_basic_block();
        let then_block = self.context.insert_basic_block_after(current_block, "then");
        let else_block = self.context.insert_basic_block_after(then_block, "else");
        let cont_block = if else_stmt.is_some() {
          self.context.insert_basic_block_after(else_block, "cont")
        } else {
          else_block
        };

        let cond = self.gen_expr_into_int_value(cond, vars)?;
        let zero = i64_type.const_int(0, false);
        let cond = self
          .builder
          .build_int_compare(IntPredicate::NE, cond, zero, "cond");
        self
          .builder
          .build_conditional_branch(cond, then_block, else_block);

        // then:
        self.builder.position_at_end(then_block);
        let has_terminator_in_then = self.gen_statement(*then_stmt, vars)?;
        if !has_terminator_in_then {
          self.builder.build_unconditional_branch(cont_block);
        }

        // else:
        let has_terminator_in_else = if let Some(else_stmt) = else_stmt {
          self.builder.position_at_end(else_block);
          let has_terminator = self.gen_statement(*else_stmt, vars)?;
          if !has_terminator {
            self.builder.build_unconditional_branch(cont_block);
          }
          has_terminator
        } else {
          false
        };

        // cont:
        self.builder.position_at_end(cont_block);
        if has_terminator_in_then && has_terminator_in_else {
          self.builder.build_unreachable();
          Ok(true)
        } else {
          Ok(false)
        }
      }
      Stmt::For(init, cond, inc, body) => {
        /* `for (A; B; C) D`
         *   A;
         *   goto begin;
         * begin:
         *   B != 0 ? goto body : goto end;
         * body:
         *   D;
         *   C;
         *   goto begin;
         * end:
         */
        let current_block = self.get_current_basic_block();
        let begin_block = self
          .context
          .insert_basic_block_after(current_block, "begin");
        let body_block = self.context.insert_basic_block_after(begin_block, "body");
        let end_block = self.context.insert_basic_block_after(body_block, "end");

        if let Some(expr) = init {
          self.gen_expr(expr, vars)?;
        }
        self.builder.build_unconditional_branch(begin_block);

        // begin:
        self.builder.position_at_end(begin_block);
        let has_no_branch_to_end = cond.is_none();
        if let Some(expr) = cond {
          let cond = self.gen_expr_into_int_value(expr, vars)?;
          let zero = i64_type.const_int(0, false);
          let cond = self
            .builder
            .build_int_compare(IntPredicate::NE, cond, zero, "cond");
          self
            .builder
            .build_conditional_branch(cond, body_block, end_block);
        } else {
          self.builder.build_unconditional_branch(body_block);
        }

        // body:
        self.builder.position_at_end(body_block);
        let has_terminator_in_body = self.gen_statement(*body, vars)?;
        if let Some(expr) = inc {
          self.gen_expr(expr, vars)?;
        }
        if !has_terminator_in_body {
          self.builder.build_unconditional_branch(begin_block);
        }

        // end:
        self.builder.position_at_end(end_block);
        if has_no_branch_to_end {
          self.builder.build_unreachable();
        }
        Ok(has_no_branch_to_end)
      }
      Stmt::Return(expr) => {
        let return_value = self.gen_expr(expr, vars)?;
        self.builder.build_return(Some(&return_value));
        Ok(true)
      }
      Stmt::Block(stmts) => {
        let mut has_terminator = false;
        for stmt in stmts {
          has_terminator = self.gen_statement(stmt, vars)?;
          if has_terminator {
            break;
          }
        }
        Ok(has_terminator)
      }
      Stmt::Expr(expr) => {
        self.gen_expr(expr, vars)?;
        Ok(false)
      }
    }
  }

  fn gen_expr_into_int_value(
    &self,
    expr: AST,
    vars: &mut HashMap<String, PointerValue<'ctx>>,
  ) -> Expected<IntValue<'ctx>> {
    let value = self.gen_expr(expr, vars)?;
    if value.is_int_value() {
      Ok(value.into_int_value())
    } else {
      Err("unexpected type in expression, expecting int type")
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
      Err("mismatched types between lhs and rhs of assignment")
    }
  }

  fn gen_addr(
    &self,
    lvalue: AST,
    vars: &mut HashMap<String, PointerValue<'ctx>>,
  ) -> Expected<PointerValue<'ctx>> {
    match lvalue {
      AST::Assign(n, m) => {
        let rhs = self.gen_expr(*m, vars)?;
        let lhs = self.gen_addr(*n, vars)?;
        self.gen_assign_impl(lhs, rhs)
      }
      AST::Deref(n) => {
        let ptr = self.gen_expr(*n, vars)?;
        if ptr.is_pointer_value() {
          Ok(ptr.into_pointer_value())
        } else {
          Err("cannot dereference int value")
        }
      }
      AST::Ident(name) => match vars.get(&name) {
        Some(&var) => Ok(var),
        None => Err("variable shoube declared before its first use"),
      },
      _ => Err("cannot obtain address of rvalue"),
    }
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

  fn gen_expr(
    &self,
    expr: AST,
    vars: &mut HashMap<String, PointerValue<'ctx>>,
  ) -> Expected<BasicValueEnum<'ctx>> {
    let i64_type = self.context.i64_type();
    match expr {
      AST::Eq(n, m) => {
        let lhs = self.gen_expr_into_int_value(*n, vars)?;
        let rhs = self.gen_expr_into_int_value(*m, vars)?;
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
        let lhs = self.gen_expr_into_int_value(*n, vars)?;
        let rhs = self.gen_expr_into_int_value(*m, vars)?;
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
        let lhs = self.gen_expr_into_int_value(*n, vars)?;
        let rhs = self.gen_expr_into_int_value(*m, vars)?;
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
        let lhs = self.gen_expr_into_int_value(*n, vars)?;
        let rhs = self.gen_expr_into_int_value(*m, vars)?;
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
        let lhs = self.gen_expr(*n, vars)?;
        let rhs = self.gen_expr(*m, vars)?;
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
          _ => Err("types of lhs and rhs of addition are not consistent")
        }
      }
      AST::Sub(n, m) => {
        let lhs = self.gen_expr(*n, vars)?;
        let rhs = self.gen_expr(*m, vars)?;
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
            if lhs.get_type() == rhs.get_type() {
              let res = self
                .builder
                .build_ptr_diff(lhs, rhs, "tmp_ptr_diff")
                .as_basic_value_enum();
              Ok(res)
            } else {
              Err("types of lhs and rhs of pointer-pointer subtraction are not consistent")
            }
          }
          _ => Err("types of lhs and rhs of subtraction are not consistent")
        }
      }
      AST::Mul(n, m) => {
        let lhs = self.gen_expr_into_int_value(*n, vars)?;
        let rhs = self.gen_expr_into_int_value(*m, vars)?;
        let res = self
          .builder
          .build_int_mul(lhs, rhs, "tmpmul")
          .as_basic_value_enum();
        Ok(res)
      }
      AST::Div(n, m) => {
        let lhs = self.gen_expr_into_int_value(*n, vars)?;
        let rhs = self.gen_expr_into_int_value(*m, vars)?;
        let res = self
          .builder
          .build_int_signed_div(lhs, rhs, "tmpdiv")
          .as_basic_value_enum();
        Ok(res)
      }
      AST::Addr(n) => {
        let var = self.gen_addr(*n, vars)?;
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
            .map(|expr| self.gen_expr(expr, vars))
            .collect::<Result<Vec<_>, _>>()?;
          let arg_types: Vec<_> = args.iter().map(|arg| arg.get_type()).collect();
          if arg_types == stored_param_types {
            let args: Vec<_> = args
              .into_iter()
              .map(|arg| arg.into())
              .collect();
            let res = self
              .builder
              .build_call(callee, args.as_slice(), "tmpcall")
              .try_as_basic_value()
              .unwrap_left();
            Ok(res)
          } else {
            Err("argument types mismatch function parameter types")
          }
        } else {
          Err("function not defined")
        }
      }
      AST::Num(n) => Ok(i64_type.const_int(n, false).as_basic_value_enum()),
      lvalue /* AST::Assign, AST::Deref, AST::Ident */ => {
        let var = self.gen_addr(lvalue, vars)?;
        if var.get_type().get_element_type().is_array_type() {
          Ok(self.gen_array_addr_impl(var))
        } else {
          let res = self.builder.build_load(var, "tmpload");
          Ok(res)
        }
      }
    }
  }
}
