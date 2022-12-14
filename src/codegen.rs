use crate::parse::{Function, Stmt, AST};
use crate::tokenize::Expected;
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use std::collections::HashMap;

use inkwell::values::*;
use inkwell::IntPredicate;

pub struct CodeGen<'ctx> {
  pub context: &'ctx Context,
  pub module: Module<'ctx>,
}

impl<'ctx> CodeGen<'ctx> {
  pub fn codegen(&self, functions: Vec<Function>) -> Expected<String> {
    for function in functions {
      let i64_type = self.context.i64_type();
      let fn_type = i64_type.fn_type(&[], false);
      let fn_value = self.module.add_function(&function.name, fn_type, None);
      let entry_block = self.context.append_basic_block(fn_value, "entry");
      let builder = self.context.create_builder();
      builder.position_at_end(entry_block);

      GenFunction {
        context: self.context,
        fn_value,
        builder,
      }
      .gen_function(function.body)?;
    }
    Ok(self.module.to_string())
  }
}

pub struct GenFunction<'ctx> {
  pub context: &'ctx Context,
  pub fn_value: FunctionValue<'ctx>,
  pub builder: Builder<'ctx>,
}

impl<'ctx> GenFunction<'ctx> {
  fn gen_function(&self, stmts: Vec<Stmt>) -> Expected<()> {
    let mut vars: HashMap<String, PointerValue<'ctx>> = HashMap::new();

    for stmt in stmts {
      self.gen_statement(stmt, &mut vars)?;
    }

    if self.fn_value.verify(true) {
      Ok(())
    } else {
      unsafe {
        self.fn_value.delete();
      }
      Err("postprocess: failed to verify function")
    }
  }

  fn gen_statement(
    &self,
    stmt: Stmt,
    vars: &mut HashMap<String, PointerValue<'ctx>>,
  ) -> Expected<()> {
    let i64_type = self.context.i64_type();
    match stmt {
      Stmt::IfElse(cond, then_stmt, else_stmt) => {
        let cond = self.gen_expr_into_int_value(cond, vars)?;
        let zero = i64_type.const_int(0, false);
        let cond = self
          .builder
          .build_int_compare(IntPredicate::NE, cond, zero, "cond");

        let then_block = self.context.append_basic_block(self.fn_value, "then");
        let else_block = self.context.append_basic_block(self.fn_value, "else");
        self
          .builder
          .build_conditional_branch(cond, then_block, else_block);

        if let Some(else_stmt) = else_stmt {
          self.builder.position_at_end(then_block);
          self.gen_statement(*then_stmt, vars)?;
          let cont_block = if then_block.get_terminator().is_none() {
            let cont_block = self.context.append_basic_block(self.fn_value, "cont");
            self.builder.build_unconditional_branch(cont_block);
            Some(cont_block)
          } else {
            None
          };

          self.builder.position_at_end(else_block);
          self.gen_statement(*else_stmt, vars)?;
          let cont_block = if else_block.get_terminator().is_none() {
            let cont_block =
              cont_block.unwrap_or_else(|| self.context.append_basic_block(self.fn_value, "cont"));
            self.builder.build_unconditional_branch(cont_block);
            cont_block
          } else {
            else_block
          };
          self.builder.position_at_end(cont_block);
          Ok(())
        } else {
          self.builder.position_at_end(then_block);
          self.gen_statement(*then_stmt, vars)?;
          if then_block.get_terminator().is_none() {
            self.builder.build_unconditional_branch(else_block);
          }

          self.builder.position_at_end(else_block);
          Ok(())
        }
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

  // TODO: Creates a new stack allocation instruction in the entry block of the function.
  fn create_entry_block_alloca(
    &self,
    name: String,
    vars: &mut HashMap<String, PointerValue<'ctx>>,
  ) -> PointerValue<'ctx> {
    let builder = self.context.create_builder();
    let entry_block = self.fn_value.get_first_basic_block().unwrap();
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
  ) -> Expected<IntValue> {
    let value = self.gen_expr(expr, vars)?;
    if value.is_pointer_value() {
      Ok(
        self
          .builder
          .build_load(value.into_pointer_value(), "tmpload")
          .into_int_value(),
      )
    } else {
      Ok(value.into_int_value())
    }
  }

  fn gen_expr(
    &self,
    expr: AST,
    vars: &mut HashMap<String, PointerValue<'ctx>>,
  ) -> Expected<BasicValueEnum> {
    let i64_type = self.context.i64_type();
    match expr {
      AST::Assign(n, m) => {
        let rhs = self.gen_expr_into_int_value(*m, vars)?;
        match *n {
          AST::Ident(name) => match vars.get(&name) {
            Some(&var) => {
              self.builder.build_store(var, rhs);
              Ok(var.as_basic_value_enum())
            }
            None => {
              let alloca = self.create_entry_block_alloca(name, vars);
              self.builder.build_store(alloca, rhs);
              Ok(alloca.as_basic_value_enum())
            }
          },
          _ => {
            let lhs = self.gen_expr(*n, vars)?;
            if lhs.is_pointer_value() {
              let i64_ptr_value = lhs.into_pointer_value();
              self.builder.build_store(i64_ptr_value, rhs);
              Ok(i64_ptr_value.as_basic_value_enum())
            } else {
              Err("unexpected rvalue, expecting lvalue")
            }
          }
        }
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
        Ok(
          self
            .builder
            .build_int_add(lhs, rhs, "tmpadd")
            .as_basic_value_enum(),
        )
      }
      AST::Sub(n, m) => {
        let lhs = self.gen_expr_into_int_value(*n, vars)?;
        let rhs = self.gen_expr_into_int_value(*m, vars)?;
        Ok(
          self
            .builder
            .build_int_sub(lhs, rhs, "tmpsub")
            .as_basic_value_enum(),
        )
      }
      AST::Mul(n, m) => {
        let lhs = self.gen_expr_into_int_value(*n, vars)?;
        let rhs = self.gen_expr_into_int_value(*m, vars)?;
        Ok(
          self
            .builder
            .build_int_mul(lhs, rhs, "tmpmul")
            .as_basic_value_enum(),
        )
      }
      AST::Div(n, m) => {
        let lhs = self.gen_expr_into_int_value(*n, vars)?;
        let rhs = self.gen_expr_into_int_value(*m, vars)?;
        Ok(
          self
            .builder
            .build_int_signed_div(lhs, rhs, "tmpdiv")
            .as_basic_value_enum(),
        )
      }
      AST::Ident(name) => match vars.get(&name) {
        Some(var) => Ok(var.as_basic_value_enum()),
        None => Err("variable not defined"),
      },
      AST::Num(n) => Ok(i64_type.const_int(n, false).as_basic_value_enum()),
    }
  }
}
