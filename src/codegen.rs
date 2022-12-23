use crate::parse::AST;
use crate::tokenize::Expected;
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;

use inkwell::values::{AnyValue, IntValue};
use inkwell::IntPredicate;

pub struct CodeGen<'ctx> {
  pub context: &'ctx Context,
  pub module: Module<'ctx>,
  pub builder: Builder<'ctx>,
}

impl<'ctx> CodeGen<'ctx> {
  fn gen<'a>(&self, ast: AST<'a>) -> Expected<IntValue> {
    let i64_type = self.context.i64_type();
    match ast {
      AST::Assign(n, m) => {
        let lhs = self.gen(*n)?;
        let rhs = self.gen(*m)?;
        // let cmp = self
        //   .builder
        //   .build_int_compare(IntPredicate::EQ, lhs, rhs, "");
        // let cmp = self.builder.build_int_cast(cmp, i64_type, "");
        // Ok(self.builder.build_int_neg(cmp, ""))
        Err("unimplemented!")
      }
      AST::Eq(n, m) => {
        let lhs = self.gen(*n)?;
        let rhs = self.gen(*m)?;
        let cmp = self
          .builder
          .build_int_compare(IntPredicate::EQ, lhs, rhs, "");
        let cmp = self.builder.build_int_cast(cmp, i64_type, "");
        Ok(self.builder.build_int_neg(cmp, ""))
      }
      AST::Ne(n, m) => {
        let lhs = self.gen(*n)?;
        let rhs = self.gen(*m)?;
        let cmp = self
          .builder
          .build_int_compare(IntPredicate::NE, lhs, rhs, "");
        let cmp = self.builder.build_int_cast(cmp, i64_type, "");
        Ok(self.builder.build_int_neg(cmp, ""))
      }
      AST::Lt(n, m) => {
        let lhs = self.gen(*n)?;
        let rhs = self.gen(*m)?;
        let cmp = self
          .builder
          .build_int_compare(IntPredicate::ULT, lhs, rhs, "");
        let cmp = self.builder.build_int_cast(cmp, i64_type, "");
        Ok(self.builder.build_int_neg(cmp, ""))
      }
      AST::Le(n, m) => {
        let lhs = self.gen(*n)?;
        let rhs = self.gen(*m)?;
        let cmp = self
          .builder
          .build_int_compare(IntPredicate::ULE, lhs, rhs, "");
        let cmp = self.builder.build_int_cast(cmp, i64_type, "");
        Ok(self.builder.build_int_neg(cmp, ""))
      }
      AST::Add(n, m) => {
        let lhs = self.gen(*n)?;
        let rhs = self.gen(*m)?;
        Ok(self.builder.build_int_add(lhs, rhs, ""))
      }
      AST::Sub(n, m) => {
        let lhs = self.gen(*n)?;
        let rhs = self.gen(*m)?;
        Ok(self.builder.build_int_sub(lhs, rhs, ""))
      }
      AST::Mul(n, m) => {
        let lhs = self.gen(*n)?;
        let rhs = self.gen(*m)?;
        Ok(self.builder.build_int_mul(lhs, rhs, ""))
      }
      AST::Div(n, m) => {
        let lhs = self.gen(*n)?;
        let rhs = self.gen(*m)?;
        Ok(self.builder.build_int_signed_div(lhs, rhs, ""))
      }
      AST::Ident(name) => Err("unimplemented!"),
      AST::Num(n) => Ok(i64_type.const_int(n, false)),
    }
  }

  pub fn codegen(&self, ast: AST) -> Expected<String> {
    let i64_type = self.context.i64_type();
    let fn_type = i64_type.fn_type(&[], false);
    let function = self.module.add_function("main", fn_type, None);
    let basic_block = self.context.append_basic_block(function, "entry");

    self.builder.position_at_end(basic_block);

    let i64_value = self.gen(ast)?;

    self.builder.build_return(Some(&i64_value));

    if function.verify(true) {
      Ok(function.print_to_string().to_string())
    } else {
      // unsafe { function.delete(); }
      Err("postprocess: failed to verify function")
    }
  }
}
