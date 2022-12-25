use crate::parse::AST;
use crate::tokenize::Expected;
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use std::collections::HashMap;

use inkwell::values::{AnyValue, IntValue, PointerValue};
use inkwell::IntPredicate;

pub struct CodeGen<'ctx> {
  pub context: &'ctx Context,
  pub module: Module<'ctx>,
  pub builder: Builder<'ctx>,
  // variables: HashMap<String, PointerValue<'ctx>>,
}

impl<'ctx> CodeGen<'ctx> {
  fn gen_assign(
    &self,
    ast: AST,
    vars: &mut HashMap<String, PointerValue<'ctx>>,
  ) -> Expected<IntValue> {
    let i64_type = self.context.i64_type();
    match ast {
      AST::Assign(n, m) => {
        let rhs = self.gen_assign(*m, vars)?;
        match *n {
          AST::Ident(name) => match vars.get(name.as_str()) {
            Some(_var) => Err("variable already defined"),
            None => {
              let alloca = self.builder.build_alloca(i64_type, name.as_str());
              self.builder.build_store(alloca, rhs);
              let ret = self
                .builder
                .build_load(alloca, name.as_str())
                .into_int_value();
              vars.insert(name, alloca);
              Ok(ret)
            }
          },
          _ => Err("unexpected rvalue, expecting lvalue"),
        }
      }
      AST::Eq(n, m) => {
        let lhs = self.gen_assign(*n, vars)?;
        let rhs = self.gen_assign(*m, vars)?;
        let cmp = self
          .builder
          .build_int_compare(IntPredicate::EQ, lhs, rhs, "");
        let cmp = self.builder.build_int_cast(cmp, i64_type, "");
        Ok(self.builder.build_int_neg(cmp, ""))
      }
      AST::Ne(n, m) => {
        let lhs = self.gen_assign(*n, vars)?;
        let rhs = self.gen_assign(*m, vars)?;
        let cmp = self
          .builder
          .build_int_compare(IntPredicate::NE, lhs, rhs, "");
        let cmp = self.builder.build_int_cast(cmp, i64_type, "");
        Ok(self.builder.build_int_neg(cmp, ""))
      }
      AST::Lt(n, m) => {
        let lhs = self.gen_assign(*n, vars)?;
        let rhs = self.gen_assign(*m, vars)?;
        let cmp = self
          .builder
          .build_int_compare(IntPredicate::ULT, lhs, rhs, "");
        let cmp = self.builder.build_int_cast(cmp, i64_type, "");
        Ok(self.builder.build_int_neg(cmp, ""))
      }
      AST::Le(n, m) => {
        let lhs = self.gen_assign(*n, vars)?;
        let rhs = self.gen_assign(*m, vars)?;
        let cmp = self
          .builder
          .build_int_compare(IntPredicate::ULE, lhs, rhs, "");
        let cmp = self.builder.build_int_cast(cmp, i64_type, "");
        Ok(self.builder.build_int_neg(cmp, ""))
      }
      AST::Add(n, m) => {
        let lhs = self.gen_assign(*n, vars)?;
        let rhs = self.gen_assign(*m, vars)?;
        Ok(self.builder.build_int_add(lhs, rhs, ""))
      }
      AST::Sub(n, m) => {
        let lhs = self.gen_assign(*n, vars)?;
        let rhs = self.gen_assign(*m, vars)?;
        Ok(self.builder.build_int_sub(lhs, rhs, ""))
      }
      AST::Mul(n, m) => {
        let lhs = self.gen_assign(*n, vars)?;
        let rhs = self.gen_assign(*m, vars)?;
        Ok(self.builder.build_int_mul(lhs, rhs, ""))
      }
      AST::Div(n, m) => {
        let lhs = self.gen_assign(*n, vars)?;
        let rhs = self.gen_assign(*m, vars)?;
        Ok(self.builder.build_int_signed_div(lhs, rhs, ""))
      }
      AST::Ident(name) => match vars.get(name.as_str()) {
        Some(var) => Ok(
          self
            .builder
            .build_load(*var, name.as_str())
            .into_int_value(),
        ),
        None => Err("variable not defined"),
      },
      AST::Num(n) => Ok(i64_type.const_int(n, false)),
    }
  }

  pub fn codegen(&self, ast: AST) -> Expected<String> {
    let i64_type = self.context.i64_type();
    let fn_type = i64_type.fn_type(&[], false);
    let function = self.module.add_function("main", fn_type, None);
    let mut vars: HashMap<String, PointerValue<'ctx>> = HashMap::new();

    let basic_block = self.context.append_basic_block(function, "entry");
    self.builder.position_at_end(basic_block);

    let i64_value = self.gen_assign(ast, &mut vars)?;

    self.builder.build_return(Some(&i64_value));

    if function.verify(true) {
      Ok(function.print_to_string().to_string())
    } else {
      // unsafe { function.delete(); }
      Err("postprocess: failed to verify function")
    }
  }
}
