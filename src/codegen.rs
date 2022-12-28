use crate::parse::{Function, Stmt, AST};
use crate::tokenize::Expected;
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use std::collections::HashMap;

use inkwell::values::{BasicValueEnum, IntValue, PointerValue};
use inkwell::IntPredicate;

pub struct CodeGen<'ctx> {
  pub context: &'ctx Context,
  pub module: Module<'ctx>,
  pub builder: Builder<'ctx>,
  // variables: HashMap<String, PointerValue<'ctx>>,
}

impl<'ctx> CodeGen<'ctx> {
  pub fn codegen(&self, functions: Vec<Function>) -> Expected<String> {
    for function in functions {
      self.gen_function(function)?;
    }
    Ok(self.module.to_string())
  }

  fn gen_function(&self, function: Function) -> Expected<()> {
    let i64_type = self.context.i64_type();
    let fn_type = i64_type.fn_type(&[], false);
    let fn_value = self.module.add_function(&function.name, fn_type, None);
    let mut vars: HashMap<String, PointerValue<'ctx>> = HashMap::new();

    let entry_block = self.context.append_basic_block(fn_value, "entry");
    self.builder.position_at_end(entry_block);

    for stmt in function.body {
      self.gen_statement(stmt, &mut vars)?;
    }

    if fn_value.verify(true) {
      Ok(())
    } else {
      unsafe {
        fn_value.delete();
      }
      Err("postprocess: failed to verify function")
    }
  }

  fn gen_statement(
    &self,
    stmt: Stmt,
    vars: &mut HashMap<String, PointerValue<'ctx>>,
  ) -> Expected<()> {
    match stmt {
      Stmt::Return(expr) => {
        let i64_value = self.gen_expr_into_int_value(expr, vars)?;
        self.builder.build_return(Some(&i64_value));
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
    let i64_type = self.context.i64_type();
    let alloca = self.builder.build_alloca(i64_type, &name);
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
            Some(_var) => Err("variable already defined"),
            None => {
              let alloca = self.create_entry_block_alloca(name, vars);
              self.builder.build_store(alloca, rhs);
              Ok(BasicValueEnum::PointerValue(alloca))
            }
          },
          _ => {
            let lhs = self.gen_expr(*n, vars)?;
            if lhs.is_pointer_value() {
              let i64_ptr_value = lhs.into_pointer_value();
              self.builder.build_store(i64_ptr_value, rhs);
              Ok(BasicValueEnum::PointerValue(i64_ptr_value))
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
        let b = self.builder.build_int_cast(cmp, i64_type, "tmpbool");
        Ok(BasicValueEnum::IntValue(self.builder.build_int_neg(b, "")))
      }
      AST::Ne(n, m) => {
        let lhs = self.gen_expr_into_int_value(*n, vars)?;
        let rhs = self.gen_expr_into_int_value(*m, vars)?;
        let cmp = self
          .builder
          .build_int_compare(IntPredicate::NE, lhs, rhs, "tmpcmp");
        let b = self.builder.build_int_cast(cmp, i64_type, "tmpbool");
        Ok(BasicValueEnum::IntValue(self.builder.build_int_neg(b, "")))
      }
      AST::Lt(n, m) => {
        let lhs = self.gen_expr_into_int_value(*n, vars)?;
        let rhs = self.gen_expr_into_int_value(*m, vars)?;
        let cmp = self
          .builder
          .build_int_compare(IntPredicate::ULT, lhs, rhs, "tmpcmp");
        let b = self.builder.build_int_cast(cmp, i64_type, "tmpbool");
        Ok(BasicValueEnum::IntValue(self.builder.build_int_neg(b, "")))
      }
      AST::Le(n, m) => {
        let lhs = self.gen_expr_into_int_value(*n, vars)?;
        let rhs = self.gen_expr_into_int_value(*m, vars)?;
        let cmp = self
          .builder
          .build_int_compare(IntPredicate::ULE, lhs, rhs, "tmpcmp");
        let b = self.builder.build_int_cast(cmp, i64_type, "tmpbool");
        Ok(BasicValueEnum::IntValue(self.builder.build_int_neg(b, "")))
      }
      AST::Add(n, m) => {
        let lhs = self.gen_expr_into_int_value(*n, vars)?;
        let rhs = self.gen_expr_into_int_value(*m, vars)?;
        Ok(BasicValueEnum::IntValue(
          self.builder.build_int_add(lhs, rhs, "tmpadd"),
        ))
      }
      AST::Sub(n, m) => {
        let lhs = self.gen_expr_into_int_value(*n, vars)?;
        let rhs = self.gen_expr_into_int_value(*m, vars)?;
        Ok(BasicValueEnum::IntValue(
          self.builder.build_int_sub(lhs, rhs, "tmpsub"),
        ))
      }
      AST::Mul(n, m) => {
        let lhs = self.gen_expr_into_int_value(*n, vars)?;
        let rhs = self.gen_expr_into_int_value(*m, vars)?;
        Ok(BasicValueEnum::IntValue(
          self.builder.build_int_mul(lhs, rhs, "tmpmul"),
        ))
      }
      AST::Div(n, m) => {
        let lhs = self.gen_expr_into_int_value(*n, vars)?;
        let rhs = self.gen_expr_into_int_value(*m, vars)?;
        Ok(BasicValueEnum::IntValue(
          self.builder.build_int_signed_div(lhs, rhs, "tmpdiv"),
        ))
      }
      AST::Ident(name) => match vars.get(&name) {
        Some(var) => Ok(BasicValueEnum::PointerValue(*var)),
        None => Err("variable not defined"),
      },
      AST::Num(n) => Ok(BasicValueEnum::IntValue(i64_type.const_int(n, false))),
    }
  }
}
