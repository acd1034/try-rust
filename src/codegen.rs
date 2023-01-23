use crate::parse::{Function, Stmt, Type, AST};
use crate::tokenize::Expected;
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use std::collections::HashMap;

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
    }
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

  // Creates a new stack allocation instruction in the entry block of the function.
  fn create_entry_block_alloca(
    &self,
    ty: BasicTypeEnum<'ctx>,
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

    let alloca = builder.build_alloca(ty, &name);
    vars.insert(name, alloca);
    alloca
  }

  fn gen_function(&self, function: Function) -> Expected<()> {
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

          for stmt in body {
            self.gen_statement(stmt, &mut vars)?;
          }

          if fn_value.verify(true) {
            Ok(())
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
        self.check_prototype(ret_ty, &name, param_tys)?;
        Ok(())
      }
    }
  }

  fn get_current_function(&self) -> FunctionValue<'ctx> {
    self
      .builder
      .get_insert_block()
      .unwrap()
      .get_parent()
      .unwrap()
  }

  fn gen_statement(
    &self,
    stmt: Stmt,
    vars: &mut HashMap<String, PointerValue<'ctx>>,
  ) -> Expected<()> {
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
          Ok(())
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
        let fn_value = self.get_current_function();
        let then_block = self.context.append_basic_block(fn_value, "then");
        let else_block = self.context.append_basic_block(fn_value, "else");

        let cond = self.gen_expr_into_int_value(cond, vars)?;
        let zero = i64_type.const_int(0, false);
        let cond = self
          .builder
          .build_int_compare(IntPredicate::NE, cond, zero, "cond");
        self
          .builder
          .build_conditional_branch(cond, then_block, else_block);

        if let Some(else_stmt) = else_stmt {
          // then:
          self.builder.position_at_end(then_block);
          self.gen_statement(*then_stmt, vars)?;
          let cont_block = if then_block.get_terminator().is_none() {
            let cont_block = self.context.append_basic_block(fn_value, "cont");
            self.builder.build_unconditional_branch(cont_block);
            Some(cont_block)
          } else {
            None
          };

          // else:
          self.builder.position_at_end(else_block);
          self.gen_statement(*else_stmt, vars)?;
          let cont_block = if else_block.get_terminator().is_none() {
            let cont_block =
              cont_block.unwrap_or_else(|| self.context.append_basic_block(fn_value, "cont"));
            self.builder.build_unconditional_branch(cont_block);
            cont_block
          } else {
            else_block
          };

          // cont:
          self.builder.position_at_end(cont_block);
          Ok(())
        } else {
          // then:
          self.builder.position_at_end(then_block);
          self.gen_statement(*then_stmt, vars)?;
          if then_block.get_terminator().is_none() {
            self.builder.build_unconditional_branch(else_block);
          }

          // else:
          self.builder.position_at_end(else_block);
          Ok(())
        }
      }
      Stmt::For(init, cond, inc, stmt) => {
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
        let fn_value = self.get_current_function();
        let begin_block = self.context.append_basic_block(fn_value, "begin");
        let body_block = self.context.append_basic_block(fn_value, "body");
        let end_block = self.context.append_basic_block(fn_value, "end");

        if let Some(expr) = init {
          self.gen_expr(expr, vars)?;
        }
        self.builder.build_unconditional_branch(begin_block);

        // begin:
        self.builder.position_at_end(begin_block);
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
        self.gen_statement(*stmt, vars)?;
        if let Some(expr) = inc {
          self.gen_expr(expr, vars)?;
        }
        if body_block.get_terminator().is_none() {
          self.builder.build_unconditional_branch(begin_block);
        }

        // end:
        self.builder.position_at_end(end_block);
        Ok(())
      }
      Stmt::Return(expr) => {
        let return_value = self.gen_expr(expr, vars)?;
        self.builder.build_return(Some(&return_value));
        Ok(())
      }
      Stmt::Block(stmts) => {
        for stmt in stmts {
          self.gen_statement(stmt, vars)?;
        }
        Ok(())
      }
      Stmt::Expr(expr) => {
        self.gen_expr(expr, vars)?;
        Ok(())
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
    ptr: BasicValueEnum<'ctx>,
    idx: BasicValueEnum<'ctx>,
  ) -> BasicValueEnum<'ctx> {
    let ptr = ptr.into_pointer_value();
    let idx = idx.into_int_value();
    unsafe {
      self
        .builder
        .build_gep(ptr, &[idx], "tmpgep")
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
        let b = self.builder.build_int_cast(cmp, i64_type, "tmpcast");
        Ok(self.builder.build_int_neg(b, "tmpneg").as_basic_value_enum())
      }
      AST::Ne(n, m) => {
        let lhs = self.gen_expr_into_int_value(*n, vars)?;
        let rhs = self.gen_expr_into_int_value(*m, vars)?;
        let cmp = self
          .builder
          .build_int_compare(IntPredicate::NE, lhs, rhs, "tmpcmp");
        let b = self.builder.build_int_cast(cmp, i64_type, "tmpcast");
        Ok(self.builder.build_int_neg(b, "tmpneg").as_basic_value_enum())
      }
      AST::Lt(n, m) => {
        let lhs = self.gen_expr_into_int_value(*n, vars)?;
        let rhs = self.gen_expr_into_int_value(*m, vars)?;
        let cmp = self
          .builder
          .build_int_compare(IntPredicate::ULT, lhs, rhs, "tmpcmp");
        let b = self.builder.build_int_cast(cmp, i64_type, "tmpcast");
        Ok(self.builder.build_int_neg(b, "tmpneg").as_basic_value_enum())
      }
      AST::Le(n, m) => {
        let lhs = self.gen_expr_into_int_value(*n, vars)?;
        let rhs = self.gen_expr_into_int_value(*m, vars)?;
        let cmp = self
          .builder
          .build_int_compare(IntPredicate::ULE, lhs, rhs, "tmpcmp");
        let b = self.builder.build_int_cast(cmp, i64_type, "tmpcast");
        Ok(self.builder.build_int_neg(b, "tmpneg").as_basic_value_enum())
      }
      AST::Add(n, m) => {
        let lhs = self.gen_expr(*n, vars)?;
        let rhs = self.gen_expr(*m, vars)?;
        match (lhs.get_type(), rhs.get_type()) {
          (BasicTypeEnum::IntType(_), BasicTypeEnum::IntType(_)) => {
            let res = self
              .builder
              .build_int_add(lhs.into_int_value(), rhs.into_int_value(), "tmpadd")
              .as_basic_value_enum();
            Ok(res)
          }
          (BasicTypeEnum::PointerType(_), BasicTypeEnum::IntType(_)) => {
            Ok(self.gen_pointer_add_impl(lhs, rhs))
          }
          (BasicTypeEnum::IntType(_), BasicTypeEnum::PointerType(_)) => {
            Ok(self.gen_pointer_add_impl(rhs, lhs))
          }
          _ => Err("types of lhs and rhs of addition are not consistent")
        }
      }
      AST::Sub(n, m) => {
        let lhs = self.gen_expr(*n, vars)?;
        let rhs = self.gen_expr(*m, vars)?;
        match (lhs.get_type(), rhs.get_type()) {
          (BasicTypeEnum::IntType(_), BasicTypeEnum::IntType(_)) => {
            let res = self
              .builder
              .build_int_sub(lhs.into_int_value(), rhs.into_int_value(), "tmpsub")
              .as_basic_value_enum();
            Ok(res)
          }
          (BasicTypeEnum::PointerType(_), BasicTypeEnum::IntType(_)) => {
            let rhs = self.builder.build_int_neg(rhs.into_int_value(), "tmpneg").as_basic_value_enum();
            Ok(self.gen_pointer_add_impl(lhs, rhs))
          }
          _ => Err("types of lhs and rhs of addition are not consistent")
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
        Ok(var.as_basic_value_enum())
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
        let res = self.builder.build_load(var, "tmpload");
        Ok(res)
      }
    }
  }
}
