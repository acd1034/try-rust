use crate::parse::{Function, Stmt, AST};
use crate::tokenize::Expected;
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use std::collections::HashMap;

use inkwell::values::*;
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

  fn check_prototype(&self, name: String, params: &Vec<String>) -> Expected<FunctionValue<'ctx>> {
    if let Some(fn_value) = self.module.get_function(&name) {
      if params.len() == fn_value.count_params().try_into().unwrap() {
        Ok(fn_value)
      } else {
        Err("number of arguments differs from the previous declaration")
      }
    } else {
      let i64_type = self.context.i64_type();
      let param_types: Vec<_> = params.iter().map(|_| i64_type.into()).collect();
      let fn_type = i64_type.fn_type(&param_types[..], false);
      Ok(self.module.add_function(&name, fn_type, None))
    }
  }

  fn gen_function(&self, function: Function) -> Expected<()> {
    match function {
      Function::Function(name, params, body) => {
        let fn_value = self.check_prototype(name, &params)?;
        if fn_value.count_basic_blocks() == 0 {
          let entry_block = self.context.append_basic_block(fn_value, "entry");
          self.builder.position_at_end(entry_block);
          let mut vars: HashMap<String, PointerValue<'ctx>> = HashMap::new();
          for (name, var) in std::iter::zip(params, fn_value.get_param_iter()) {
            if vars.get(&name).is_none() {
              let alloca = self.create_entry_block_alloca(name, &mut vars);
              self.builder.build_store(alloca, var.into_int_value());
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
            // TODO: 前方宣言後の定義ならば定義のみ消す
            // 前方宣言なしの定義ならば宣言ごと消す
            unsafe {
              fn_value.delete();
            }
            Err("gen_function: failed to verify function")
          }
        } else {
          Err("function already defined")
        }
      }
      Function::Prototype(name, params) => {
        self.check_prototype(name, &params)?;
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
        let i64_value = self.gen_expr_into_int_value(expr, vars)?;
        self.builder.build_return(Some(&i64_value));
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

  // Creates a new stack allocation instruction in the entry block of the function.
  fn create_entry_block_alloca(
    &self,
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

    let i64_type = self.context.i64_type();
    let alloca = builder.build_alloca(i64_type, &name);
    vars.insert(name, alloca);
    alloca
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

  fn gen_addr(
    &self,
    expr: AST,
    vars: &mut HashMap<String, PointerValue<'ctx>>,
  ) -> Expected<PointerValue<'ctx>> {
    match expr {
      AST::Assign(n, m) => {
        let rhs = self.gen_expr_into_int_value(*m, vars)?;
        let lhs = if let AST::Ident(name) = *n {
          if let Some(&var) = vars.get(&name) {
            var
          } else {
            self.create_entry_block_alloca(name, vars)
          }
        } else {
          self.gen_addr(*n, vars)?
        };
        self.builder.build_store(lhs, rhs);
        Ok(lhs)
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
        None => Err("variable not defined"),
      },
      _ => Err("cannot obtain the address of lvalue"),
    }
  }

  fn gen_expr(
    &self,
    expr: AST,
    vars: &mut HashMap<String, PointerValue<'ctx>>,
  ) -> Expected<BasicValueEnum<'ctx>> {
    let i64_type = self.context.i64_type();
    match expr {
      AST::Assign(n, m) => {
        let var = self.gen_addr(AST::Assign(n, m), vars)?;
        let res = self.builder.build_load(var, "tmpload");
        Ok(res)
      }
      AST::Eq(n, m) => {
        let lhs = self.gen_expr_into_int_value(*n, vars)?;
        let rhs = self.gen_expr_into_int_value(*m, vars)?;
        let cmp = self
          .builder
          .build_int_compare(IntPredicate::EQ, lhs, rhs, "tmpcmp");
        let b = self.builder.build_int_cast(cmp, i64_type, "tmpcast");
        Ok(self.builder.build_int_neg(b, "").as_basic_value_enum())
      }
      AST::Ne(n, m) => {
        let lhs = self.gen_expr_into_int_value(*n, vars)?;
        let rhs = self.gen_expr_into_int_value(*m, vars)?;
        let cmp = self
          .builder
          .build_int_compare(IntPredicate::NE, lhs, rhs, "tmpcmp");
        let b = self.builder.build_int_cast(cmp, i64_type, "tmpcast");
        Ok(self.builder.build_int_neg(b, "").as_basic_value_enum())
      }
      AST::Lt(n, m) => {
        let lhs = self.gen_expr_into_int_value(*n, vars)?;
        let rhs = self.gen_expr_into_int_value(*m, vars)?;
        let cmp = self
          .builder
          .build_int_compare(IntPredicate::ULT, lhs, rhs, "tmpcmp");
        let b = self.builder.build_int_cast(cmp, i64_type, "tmpcast");
        Ok(self.builder.build_int_neg(b, "").as_basic_value_enum())
      }
      AST::Le(n, m) => {
        let lhs = self.gen_expr_into_int_value(*n, vars)?;
        let rhs = self.gen_expr_into_int_value(*m, vars)?;
        let cmp = self
          .builder
          .build_int_compare(IntPredicate::ULE, lhs, rhs, "tmpcmp");
        let b = self.builder.build_int_cast(cmp, i64_type, "tmpcast");
        Ok(self.builder.build_int_neg(b, "").as_basic_value_enum())
      }
      AST::Add(n, m) => {
        let lhs = self.gen_expr_into_int_value(*n, vars)?;
        let rhs = self.gen_expr_into_int_value(*m, vars)?;
        let res = self
          .builder
          .build_int_add(lhs, rhs, "tmpadd")
          .as_basic_value_enum();
        Ok(res)
      }
      AST::Sub(n, m) => {
        let lhs = self.gen_expr_into_int_value(*n, vars)?;
        let rhs = self.gen_expr_into_int_value(*m, vars)?;
        let res = self
          .builder
          .build_int_sub(lhs, rhs, "tmpsub")
          .as_basic_value_enum();
        Ok(res)
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
      AST::Deref(n) => {
        let var = self.gen_addr(AST::Deref(n), vars)?;
        let res = self.builder.build_load(var, "tmpload");
        Ok(res)
      }
      AST::Call(name, args) => {
        if let Some(callee) = self.module.get_function(&name) {
          let args = args
            .into_iter()
            .map(|expr| self.gen_expr_into_int_value(expr, vars).map(|x| x.into()))
            .collect::<Result<Vec<BasicMetadataValueEnum>, _>>()?;
          let res = self
            .builder
            .build_call(callee, &args[..], "tmpcall")
            .try_as_basic_value()
            .unwrap_left();
          Ok(res)
        } else {
          Err("function not defined")
        }
      }
      AST::Ident(name) => {
        let var = self.gen_addr(AST::Ident(name), vars)?;
        let res = self.builder.build_load(var, "tmpload");
        Ok(res)
      }
      AST::Num(n) => Ok(i64_type.const_int(n, false).as_basic_value_enum()),
    }
  }
}
