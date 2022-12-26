use crate::parse::AST;
use crate::tokenize::Expected;
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::AddressSpace;
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
  pub fn codegen(&self, ast: Vec<AST>) -> Expected<String> {
    let i64_type = self.context.i64_type();
    let fn_type = i64_type.fn_type(&[], false);
    let function = self.module.add_function("main", fn_type, None);
    let mut vars: HashMap<String, PointerValue<'ctx>> = HashMap::new();

    let basic_block = self.context.append_basic_block(function, "entry");
    self.builder.position_at_end(basic_block);

    let _i64_value = self.gen_statements(ast, &mut vars)?;

    if function.verify(true) {
      Ok(function.print_to_string().to_string())
    } else {
      // unsafe { function.delete(); }
      Err("postprocess: failed to verify function")
    }
  }

  fn gen_statements(
    &self,
    stmts: Vec<AST>,
    vars: &mut HashMap<String, PointerValue<'ctx>>,
  ) -> Expected<IntValue> {
    let mut ex: Expected<_> = Err("empty statements");
    for assign in stmts {
      ex = Ok(self.gen_expr(assign, vars)?);
    }
    ex
  }

  fn gen_expr(
    &self,
    assign: AST,
    vars: &mut HashMap<String, PointerValue<'ctx>>,
  ) -> Expected<IntValue> {
    let i64_type = self.context.i64_type();
    match assign {
      AST::Return(n) => {
        let i64_value = self.gen_expr(*n, vars)?;
        self.builder.build_return(Some(&i64_value));
        Ok(i64_type.const_int(0, false))
      }
      AST::Assign(n, m) => {
        let rhs = self.gen_expr(*m, vars)?;
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
          _ => {
            let lhs = self.gen_expr(*n, vars)?;
            if lhs.is_const() {
              Err("unexpected rvalue, expecting lvalue")
            } else {
              // let name_str = lhs.get_name().to_str().map_err(|_| "failed to read cstr")?;
              // let alloca = vars.get(name_str).ok_or("XXX: failed to find var")?;
              // self.builder.build_store(*alloca, rhs);
              // let ret = self.builder.build_load(*alloca, name_str).into_int_value();
              // Ok(ret)
              Err("unimplemented!")
            }
          }
        }
      }
      AST::Eq(n, m) => {
        let lhs = self.gen_expr(*n, vars)?;
        let rhs = self.gen_expr(*m, vars)?;
        let cmp = self
          .builder
          .build_int_compare(IntPredicate::EQ, lhs, rhs, "");
        let cmp = self.builder.build_int_cast(cmp, i64_type, "");
        Ok(self.builder.build_int_neg(cmp, ""))
      }
      AST::Ne(n, m) => {
        let lhs = self.gen_expr(*n, vars)?;
        let rhs = self.gen_expr(*m, vars)?;
        let cmp = self
          .builder
          .build_int_compare(IntPredicate::NE, lhs, rhs, "");
        let cmp = self.builder.build_int_cast(cmp, i64_type, "");
        Ok(self.builder.build_int_neg(cmp, ""))
      }
      AST::Lt(n, m) => {
        let lhs = self.gen_expr(*n, vars)?;
        let rhs = self.gen_expr(*m, vars)?;
        let cmp = self
          .builder
          .build_int_compare(IntPredicate::ULT, lhs, rhs, "");
        let cmp = self.builder.build_int_cast(cmp, i64_type, "");
        Ok(self.builder.build_int_neg(cmp, ""))
      }
      AST::Le(n, m) => {
        let lhs = self.gen_expr(*n, vars)?;
        let rhs = self.gen_expr(*m, vars)?;
        let cmp = self
          .builder
          .build_int_compare(IntPredicate::ULE, lhs, rhs, "");
        let cmp = self.builder.build_int_cast(cmp, i64_type, "");
        Ok(self.builder.build_int_neg(cmp, ""))
      }
      AST::Add(n, m) => {
        let lhs = self.gen_expr(*n, vars)?;
        let rhs = self.gen_expr(*m, vars)?;
        Ok(self.builder.build_int_add(lhs, rhs, ""))
      }
      AST::Sub(n, m) => {
        let lhs = self.gen_expr(*n, vars)?;
        let rhs = self.gen_expr(*m, vars)?;
        Ok(self.builder.build_int_sub(lhs, rhs, ""))
      }
      AST::Mul(n, m) => {
        let lhs = self.gen_expr(*n, vars)?;
        let rhs = self.gen_expr(*m, vars)?;
        Ok(self.builder.build_int_mul(lhs, rhs, ""))
      }
      AST::Div(n, m) => {
        let lhs = self.gen_expr(*n, vars)?;
        let rhs = self.gen_expr(*m, vars)?;
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
}
